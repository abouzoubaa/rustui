"""Correlate mach-clock input, invalidation, draw, submit and actual display events."""
from bisect import bisect_left
from collections import Counter, defaultdict
import json
import math
from pathlib import Path
import statistics


def distribution(values):
    if not values:
        return dict(samples=0, median_ms=None, p95_ms=None, min_ms=None, max_ms=None)
    values = sorted(values)
    return dict(samples=len(values), median_ms=statistics.median(values),
                p95_ms=values[math.ceil(.95*len(values))-1], min_ms=values[0], max_ms=values[-1])


def summarize(path):
    events, old, markers = [], [], []
    for line in Path(path).read_text().splitlines():
        for prefix, target in [('RUSTUI_PX4_LOG ', events), ('RUSTUI_INPUT_LOG ', old), ('RUSTUI_G0_LOG ', markers)]:
            if line.startswith(prefix):
                target.append(json.loads(line[len(prefix):]))
                break
    clocks = [r for r in events if r['event'] == 'clock']
    assert len(clocks) == 1 and clocks[0]['mach_absolute_time'], 'missing real mach clock'
    scale = clocks[0]['numer']/clocks[0]['denom']/1e6  # raw ticks -> milliseconds
    starts, ends = [[r for r in markers if r['event'] == kind] for kind in ('start', 'end')]
    assert len(starts) == len(ends) == 1 and ends[0]['valid'], 'incomplete workload'
    start, end = starts[0]['epoch_ms'], ends[0]['epoch_ms']
    inputs_in_window = {(r['window'], r['version']) for r in old if r['event'] == 'input' and start <= r['epoch_ms'] <= end}
    inputs = [r for r in events if r['event'] == 'handled' and (r['window'], r['version']) in inputs_in_window]
    assert inputs, 'no measured handled inputs'
    begin_tick=starts[0].get('mach',min(r['mach'] for r in inputs))
    end_tick=ends[0].get('mach',max(r['mach'] for r in inputs))
    histories=defaultdict(list)
    draws, paints, attempts, commits, completed, displays = {}, {}, {}, {}, {}, []
    requests, callbacks = defaultdict(list), defaultdict(list)
    configs, inflight = defaultdict(dict), defaultdict(list)
    invalid = 0
    for r in events:
        kind, w = r['event'], r.get('window')
        if kind == 'requested': requests[w].append(r['mach'])
        elif kind == 'callback': callbacks[w].append(r['mach'])
        elif kind == 'draw': draws[w, r['draw']] = r
        elif kind == 'painted': paints[w, r['draw']] = r
        elif kind == 'attempt': attempts[w, r['attempt']] = r
        elif kind == 'committed': commits[w, r['drawable']] = r
        elif kind == 'gpu_completed': completed[w, r['drawable']] = r
        elif kind == 'acquired' and begin_tick <= r['mach'] <= end_tick: inflight[w].append(r['in_flight'])
        elif kind == 'config':
            histories[w].append(dict(kind=r['kind'],values=r['values'],from_start_ms=(r['mach']-begin_tick)*scale))
            if r['mach'] <= end_tick: configs[w][r['kind']] = r['values']
        elif kind == 'displayed':
            p, host = r['presented_host'], r['callback_host']
            if not all(isinstance(x, (float, int)) and math.isfinite(x) for x in (p, host)) or p <= 0 or host < p:
                invalid += 1
                continue
            r['display_mach'] = r['callback_mach'] - (host-p)*1000/scale
            displays.append(r)
    for group in (requests, callbacks):
        for times in group.values(): times.sort()
    visible = defaultdict(list)
    for display in sorted(displays, key=lambda d: d['display_mach']):
        w = display['window']
        commit = commits.get((w, display['drawable']))
        if commit is None: continue
        attempt = attempts.get((w, commit['attempt']))
        if attempt is None: continue
        paint = paints.get((w, attempt['draw']))
        if paint is None: continue
        assert 0 <= paint['version']-paint['first'] <= (19 if paint['scene'] == 'log' else 0)
        for version in range(paint['first'], paint['version']+1):
            visible[w, version].append((display, commit, attempt, paint))
    names = ('handled_to_requested', 'requested_to_next_callback', 'callback_to_draw',
             'draw_to_committed', 'committed_to_gpu_completed', 'gpu_completed_to_display')
    samples = {name: [] for name in names}
    e2e, gpu_end_to_display, sums = [], [], []
    coverage = Counter()
    for item in inputs:
        w, version, handled = item['window'], item['version'], item['mach']
        matches = [m for m in visible[w, version] if m[0]['display_mach'] >= handled]
        if not matches:
            coverage['without_valid_display'] += 1
            continue
        display, commit, attempt, paint = matches[0]
        finish = display['display_mach']
        e2e.append((finish-handled)*scale)
        draw = draws.get((w, attempt['draw']))
        gpu = completed.get((w, display['drawable']))
        if draw is None or gpu is None:
            coverage['missing_draw_or_completion'] += 1
            continue
        draw_tick = draw['mach']
        if draw.get('callback',0)==0: coverage['outside_callback_draw'] += 1
        times = requests[w]
        i = bisect_left(times, handled)
        request = times[i] if i < len(times) and times[i] <= draw_tick else None
        if request is None: coverage['without_request_before_draw'] += 1
        native = callbacks[w]
        j = bisect_left(native, request) if request is not None else len(native)
        callback = native[j] if draw.get('callback',0) > 0 and j < len(native) and native[j] <= draw_tick else None
        values = [None if request is None else (request-handled)*scale,
                  None if callback is None else (callback-request)*scale,
                  None if callback is None else (draw_tick-callback)*scale,
                  (commit['mach']-draw_tick)*scale,
                  (gpu['mach']-commit['mach'])*scale,
                  (finish-gpu['mach'])*scale]
        for name, value in zip(names, values):
            if value is not None: samples[name].append(value)
        if all(v is not None for v in values):
            assert abs(sum(values)-(finish-handled)*scale) < .00001
            sums.append(sum(values))
            coverage['complete_chain'] += 1
        else: coverage['no_request_or_outside_callback'] += 1
        if gpu['gpu_end_host'] > 0:
            gpu_end_to_display.append((display['presented_host']-gpu['gpu_end_host'])*1000)
    return dict(scene=starts[0]['scene'], inputs=len(inputs), shown=len(e2e), coverage=dict(coverage),
                raw_samples=dict(stages=samples,input_to_screen=e2e,gpu_end_to_display=gpu_end_to_display),
                stages={k: distribution(v) for k,v in samples.items()},
                input_to_screen=distribution(e2e), gpu_end_to_display=distribution(gpu_end_to_display),
                unavailable_presented_callbacks=invalid,
                settings=dict(configs), configuration_history=dict(histories), in_flight={w: dict(counts=dict(Counter(v)), maximum=max(v)) for w,v in inflight.items()},
                callback_intervals={w: distribution([(b-a)*scale for a,b in zip([t for t in v if begin_tick <= t <= end_tick],[t for t in v if begin_tick <= t <= end_tick][1:])]) for w,v in callbacks.items()},
                completion_callback_after_display=sum(v < 0 for v in samples['gpu_completed_to_display']))


if __name__ == '__main__':
    import sys
    print(json.dumps(summarize(sys.argv[1]), indent=2))


def summarize_appkit(path):
    records=[json.loads(line.removeprefix('LATENCY_REF ')) for line in Path(path).read_text().splitlines() if line.startswith('LATENCY_REF ')]
    header=next(r for r in records if r['event']=='reference')
    factor=header['numer']/header['denom']/1e6
    handled={r['version']:r['mach'] for r in records if r['event']=='handled'}
    draws={}
    for r in records:
        if r['event']=='drawn': draws.setdefault(r['version'],r['mach'])
    samples=[];pre=[];post=[]
    for r in records:
        if r['event']!='display_opportunity':continue
        v=r['version']
        if v not in handled or v not in draws:continue
        assert handled[v]<=draws[v]<=r['mach']
        samples.append((r['mach']-handled[v])*factor)
        pre.append((draws[v]-handled[v])*factor)
        post.append((r['mach']-draws[v])*factor)
    return dict(method=header['method'], inputs=len(handled),samples=len(samples),reported_hz=header['reported_hz'],
                handled_to_display_opportunity_proxy=distribution(samples),handled_to_draw=distribution(pre),draw_to_callback=distribution(post),
                actual_screen_delay='not measured')
