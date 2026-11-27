"""Opt-in native-event latency budget; preserve both independent display clocks."""
import json
from bisect import bisect_left, bisect_right
from collections import Counter, defaultdict
from pathlib import Path
import g5_latency as common


def check_damage_allocations(log):
    """An allocation failure invalidates a test capture even if a later resize recovers."""
    with open(log) as records:
        for record in records:
            if record.startswith(("RUSTUI_DAMAGE_WARNING ", "RUSTUI_DAMAGE_CAPTURE_FAILED ")):
                raise RuntimeError(f"damage allocation failed during capture: {record.strip()}")


def load(path):
    data=common.records(path)
    data['L1_NATIVE ']=[json.loads(l[len('L1_NATIVE '):]) for l in Path(path).read_text().splitlines() if l.startswith('L1_NATIVE ')]
    return data


def decompose(handler,gpu,display,presented,deadline):
    """Seconds in one host timebase. Never clamp signed clock differences."""
    p=(gpu-handler)*1000;d=deadline*1000;wait=(display-gpu-deadline)*1000;s=(presented-display)*1000
    return dict(P=p,D_estimate=d,wait=wait,S=s,sck_total=(display-handler)*1000,
                presented_total=(presented-handler)*1000,
                sck_residual=p+d+wait-(display-handler)*1000,
                presented_residual=p+d+wait+s-(presented-handler)*1000)


def analyse(path):
    data=load(path);events=data['RUSTUI_PX4_LOG '];markers=data['RUSTUI_G0_LOG ']
    start=next(e for e in markers if e['event']=='start');end=next(e for e in markers if e['event']=='end')
    assert end['valid'],('final state',end)
    clock=next(e for e in events if e['event']=='clock');scale=clock['numer']/clock['denom']/1e9
    seconds=(end['mach']-start['mach'])*scale
    native={e['version']:e for e in data['L1_NATIVE '] if e['event']=='native' and 1<=e['version']<=end['ticks']}
    assert native, 'no tagged native events received'
    if all(e['kind']==0 for e in native.values()):
        assert len(native)==end['ticks'],('keyboard OS delivery',len(native),end['ticks'])
    assert all(e['consumed'] for e in native.values()),'native event was not consumed by GPUI'
    keyed=lambda event,key:{(e['window'],e[key]):e for e in events if e['event']==event}
    attempts=keyed('attempt','attempt');paints=keyed('painted','draw');draws=keyed('draw','draw')
    commits=keyed('committed','drawable');gpus=keyed('gpu_completed','drawable');acquires=keyed('acquired','drawable')
    draw_ends={};current={}
    for e in events:
        w=e.get('window')
        if e['event']=='draw':current[w]=e['draw']
        if e.get('kind')=='l1_draw_end':draw_ends[(w,current.get(w))]=e
    pixels={}
    for e in sorted(data['G5_PIXELS '],key=lambda e:e['display_mach']):pixels.setdefault(e['version'],e['display_mach']*scale)
    configurations = defaultdict(list)
    for event in events:
        if event['event'] == 'config':
            configurations[event['window'], event['kind']].append(event)
    timelines = {}
    for key, config in configurations.items():
        config.sort(key=lambda event: event['mach'])
        timelines[key] = ([event['mach'] for event in config], config)

    def between(window, kind, low, high):
        ticks, records = timelines.get((window, kind), ([], []))
        return records[bisect_left(ticks, low):bisect_right(ticks, high)]

    def frame_path(window, n, attempt, commit, draw):
        queue = between(window, 'g5_queue', attempt['mach'], commit['mach'])
        if not queue:
            return 'unknown', 'submission_without_path_marker'
        if queue[-1]['values'][2]:
            return 'immediate', 'immediate'
        if draw['mach'] <= n['handler_end_mach']:
            return 'scheduled', 'cpu_draw_before_native_handler_exit'
        if between(window, 'g53_cpu_build_deferred', n['handler_mach'], draw['mach']):
            return 'scheduled', 'admission_deferral_observed'
        if between(window, 'g5_link_recreate', n['handler_mach'], commit['mach']):
            return 'scheduled', 'link_recreated_after_input'
        state = between(window, 'g5_input_state', n['handler_mach'], draw['mach'])
        if state and state[0]['values'][0]:
            return 'scheduled', 'link_attached_at_input'
        return 'scheduled', 'scheduled_without_attributed_trigger'

    rows=[];seen=set();presentations=[];invalid=0
    all_native_paths=Counter()
    for e in sorted((e for e in events if e['event']=='displayed'),key=lambda e:e['presented_host']):
        w=e['window'];key=(w,e['drawable']);commit=commits.get(key)
        if not commit:continue
        attempt=attempts.get((w,commit['attempt']))
        if not attempt:continue
        paint=paints.get((w,attempt['draw']))
        if not paint or not e['presented_host']>0:invalid+=1;continue
        version=paint['version'];presentations.append((e['presented_host'],version))
        if version not in native:continue
        n=native[version]
        draw=draws.get((w,attempt['draw']))
        path, reason = frame_path(w, n, attempt, commit, draw) if draw else ('unknown','missing_draw')
        if start['mach']*scale <= e['presented_host'] <= end['mach']*scale:
            all_native_paths[path] += 1
        if version in seen:continue
        gpu=gpus.get(key);acquire=acquires.get(key);draw_end=draw_ends.get((w,attempt['draw']))
        if not all((gpu,acquire,draw,draw_end)):continue
        seen.add(version);h=n['handler_mach']*scale;g=gpu['gpu_end_host'];p=e['presented_host'];d=pixels.get(version)
        assert g>=h and p>=g,('clock ordering',h,g,p)
        rows.append(dict(path=path, reason=reason, version=version,hardware=n['hardware_s'],handler=h,draw_start=draw['mach']*scale,
                         draw_end=draw_end['mach']*scale,acquired=acquire['mach']*scale,committed=commit['mach']*scale,
                         gpu=g,gpu_callback=gpu['mach']*scale,display=d,presented=p,
                         input_delivery_ms=(h-n['hardware_s'])*1000,P=(g-h)*1000,
                         draw_start_ms=(draw['mach']*scale-h)*1000,draw_cpu_ms=(draw_end['mach']-draw['mach'])*scale*1000,
                         acquire_to_commit_ms=(commit['mach']-acquire['mach'])*scale*1000,
                         gpu_callback_lag_ms=(gpu['mach']*scale-g)*1000,
                         presented_total=(p-h)*1000,hardware_presented_total=(p-n['hardware_s'])*1000))
    margins=sorted(r['display']-r['gpu'] for r in rows if r['display'] is not None)
    positive=[v for v in margins if v>=0]
    # Lower envelope is an estimate, not an observed WindowServer deadline.
    deadline=positive[0] if positive else None
    for r in rows:
        if r['display'] is not None and deadline is not None:
            r.update(decompose(r['handler'],r['gpu'],r['display'],r['presented'],deadline))
            r['hardware_sck_total'] = (r['display'] - r['hardware']) * 1000
    fields=['input_delivery_ms','P','draw_start_ms','draw_cpu_ms','acquire_to_commit_ms','gpu_callback_lag_ms',
            'D_estimate','wait','S','sck_total','hardware_sck_total','presented_total','hardware_presented_total','sck_residual','presented_residual']
    stats={f:common.distribution([r[f] for r in rows if f in r]) for f in fields}
    refresh=[e['timestamp'] for e in data['G5_REFRESH ']]
    gaps=[(b[0]-a[0])*1000 for a,b in zip(presentations,presentations[1:]) if b[0]>a[0]]
    configs=[e for e in events if e.get('event')=='config' and start['mach']<=e['mach']<=end['mach']]
    recreates=[e for e in configs if e['kind']=='g5_link_recreate']
    bursts=[]
    for tick in sorted(n['handler_mach'] for n in native.values()):
        if not bursts or (tick-bursts[-1][1])*scale >= .25:
            bursts.append([tick,tick])
        else:
            bursts[-1][1]=tick
    recreate_ticks=sorted(e['mach'] for e in recreates)
    recreates_per_burst=[bisect_left(recreate_ticks, last+.25/scale)-bisect_left(recreate_ticks, first)
                         for first,last in bursts]
    attached=[e for e in configs if e['kind']=='g5_acquire_start'  and e['values'][3]!=0]
    cpu=end['cpu_user_s']+end['cpu_sys_s']
    firsts=[r['version'] for r in rows];out_of_order=sum(b<a for a,b in zip([v for t,v in presentations],[v for t,v in presentations][1:]))
    shown = sorted({v for _, v in presentations if v in native})
    superseded = {}
    unaccounted = []
    for version in sorted(set(native) - set(shown)):
        index = bisect_right(shown, version)
        if index < len(shown):
            superseded[version] = shown[index]
        else:
            unaccounted.append(version)
    keyboard = all(e['kind'] == 0 for e in native.values())
    first_input=min(n['handler_mach'] for n in native.values())
    last_input=max(n['handler_end_mach'] for n in native.values())
    revisions = [int(e['values'][1]) for e in events
                 if e.get('kind') in ('g53_kit_change_queued','g53_bare_text_applied')
                 and first_input <= e['mach'] <= last_input]
    lost = None
    if keyboard:
        lost=max(0,len(native)-len(revisions))
        assert not revisions or revisions == list(range(revisions[0],revisions[0]+len(revisions))), 'non-consecutive editor revisions'
    acquisition=common.acquisition_checks(events,start['mach'],end['mach'])
    return dict(seconds=seconds,posted=end['ticks'],inputs=len(native),shown=len(rows),captured=sum(r['display'] is not None for r in rows),
                missing_delivered_versions_superseded=superseded,
                versions_without_later_presentation=unaccounted,
                lost_edits=lost, edit_revisions=len(revisions),
                watchdog_fallbacks=sum(e.get('kind')=='immediate_fallback' for e in events),
                presentation_paths=dict(Counter(r['path'] for r in rows)),
                all_native_presentation_paths=dict(all_native_paths), recreates_per_burst=recreates_per_burst,
                presentation_path_reasons=dict(Counter(r['reason'] for r in rows)),
                final_valid=end['valid'],last_version=max(firsts,default=0),expected=end['ticks'],out_of_order=out_of_order,
                cpu_s=cpu,cpu_s_per_s=cpu/seconds,cpu_s_per_attempt=cpu/max(1,sum(e['event']=='attempt' and start['mach']<=e['mach']<=end['mach'] for e in events)),
                stats=stats,D_lower_envelope_ms=deadline*1000 if deadline is not None else None,
                completion_to_sck_ms=common.distribution([v*1000 for v in margins]),
                negative_completion_to_sck=sum(v<0 for v in margins),rows=rows,
                presented_intervals_ms=common.distribution(gaps),passive_intervals_ms=common.distribution([(b-a)*1000 for a,b in zip(refresh,refresh[1:])]),
                measured_render_link_recreates=len(recreates),measured_acquisitions_with_link=len(attached),
                **acquisition)


def analyse_appkit(path):
    """Join the NSTextView's actual native handler and displayed barcode."""
    data = load(path)
    records = data['G5_REF ']
    start = next(row for row in records if row['event'] == 'start')
    end = next(row for row in records if row['event'] == 'end')
    assert end['valid'], 'AppKit final text differed from the typing workload'
    scale = start['numer'] / start['denom'] / 1e9
    native = {row['version']: row for row in data['L1_NATIVE '] if row['event'] == 'native'}
    assert len(native) == end['inputs'] and native, 'missing AppKit native input'
    assert all(row['consumed'] for row in native.values()), 'NSTextView did not receive a native key'
    handled = {row['version']: row['mach'] * scale for row in records if row['event'] == 'handled'}
    pixels = {}
    for row in sorted(records + data['G5_PIXELS '], key=lambda row: row.get('display_mach', 0)):
        if row['event'] == 'displayed_pixels':
            pixels.setdefault(row['version'], row['display_mach'] * scale)
    drawn = {}
    for row in records:
        if row['event'] == 'draw':
            drawn.setdefault(row['version'], row['mach'] * scale)
    rows = []
    for version, event in native.items():
        if version not in pixels or version not in drawn:
            continue
        cocoa = event['handler_mach'] * scale
        display = pixels[version]
        assert display >= cocoa >= event['hardware_s'], 'AppKit clock order'
        rows.append(dict(version=version,hardware=event['hardware_s'],handler=cocoa,
                         text_view_handler=handled[version],draw_end=drawn[version],display=display,
                         input_delivery_ms=(cocoa-event['hardware_s'])*1000,
                         sck_total=(display-cocoa)*1000,
                         hardware_sck_total=(display-event['hardware_s'])*1000))
    assert rows and rows[-1]['version'] == end['inputs'], 'final AppKit text version was not captured'
    refresh = [row['timestamp'] for row in records if row['event'] == 'cadence']
    stats = {name:common.distribution([row[name] for row in rows])
             for name in ('input_delivery_ms','sck_total','hardware_sck_total')}
    cpu = end.get('cpu_s')
    seconds = (end['mach'] - start['mach']) * scale if cpu is not None else None
    return dict(inputs=len(native),captured=len(rows),stats=stats,rows=rows,seconds=seconds,
                cpu_s=cpu,cpu_s_per_s=cpu/seconds if seconds else None,
                presented_time_available=False,
                passive_intervals_ms=common.distribution([(b-a)*1000 for a,b in zip(refresh,refresh[1:])]))
