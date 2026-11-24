"""Observed-refresh diagnosis; queue/compositor attribution is explicitly inferential."""
from bisect import bisect_right
from collections import Counter
import statistics


def diagnose(refreshes, inputs, frames, blocked, floor_frames=2):
    """Times share mach/host seconds. frames=(version,commit,gpu,display|None)."""
    refreshes = sorted(set(refreshes))
    period = statistics.median(b-a for a,b in zip(refreshes,refreshes[1:])) if len(refreshes)>1 else 1/120
    inputs = sorted(inputs)
    shown = sorted((display, version) for version,_,_,display in frames if display is not None)
    input_times = [v[0] for v in inputs]
    shown_times = [v[0] for v in shown]
    blocked = sorted(blocked)
    shown_high = []
    for _,version in shown:shown_high.append(max(version,shown_high[-1] if shown_high else 0))
    ordered_frames=sorted(frames,key=lambda f:f[1]);cursor=0;pending=[]
    rows, counts, previous = [], Counter(), 0
    for tick in refreshes:
        i = bisect_right(shown_times, tick+0.0005)
        visible = shown_high[i-1] if i else 0
        j = bisect_right(input_times,tick)
        handled, latest = inputs[j-1] if j else (0,0)
        while cursor<len(ordered_frames) and ordered_frames[cursor][1]<=tick:
            pending.append(ordered_frames[cursor]);cursor+=1
        pending=[f for f in pending if f[0]>visible]
        if visible > previous:
            reason = None
        elif latest <= visible:
            reason = 'no new input'
        else:
            eligible = tick-floor_frames*period
            candidates = [f for f in pending if f[0]<=latest]
            ready = [f for f in candidates if f[2] is not None and f[2]<=eligible]
            last_commit = max((f[1] for f in candidates),default=0)
            k = bisect_right(blocked,tick)
            capacity = k>0 and blocked[k-1]>=handled and blocked[k-1]>last_commit
            replaced = any(f[3] is None and (shown_high[-1] if shown_high else 0)>f[0] for f in ready)
            reason = ('no drawable free' if capacity and not ready else
                      'frame not ready' if not ready else
                      'presented but replaced' if replaced else 'WindowServer late')
        rows.append({'refresh_host':tick,'shown_version':visible,'latest_input_version':latest,'reason':reason})
        if reason:counts[reason]+=1
        previous=max(previous,visible)
    return {'observed_slots':len(rows),'missing_new_version_slots':sum(counts.values()),
            'counts':dict(counts),'rows':rows,'floor_frames':floor_frames,
            'observer_gaps_over_1_5_refreshes':sum(b-a>1.5*period for a,b in zip(refreshes,refreshes[1:]))}


def from_records(data, floor_frames=2):
    events = data['RUSTUI_PX4_LOG ']
    markers = data['RUSTUI_G0_LOG ']
    start = next(r for r in markers if r['event']=='start')['mach']
    end = next(r for r in markers if r['event']=='end')['mach']
    clock = next(r for r in events if r['event']=='clock')
    scale = clock['numer']/clock['denom']/1e9
    attempts = {(r['window'],r['attempt']):r for r in events if r['event']=='attempt'}
    paints = {(r['window'],r['draw']):r for r in events if r['event']=='painted'}
    completed = {(r['window'],r['drawable']):r for r in events if r['event']=='gpu_completed'}
    displayed = {(r['window'],r['drawable']):r['presented_host'] for r in events if r['event']=='displayed' and r['presented_host']>0}
    frames=[]
    for r in events:
        if r['event']!='committed':continue
        a=attempts.get((r['window'],r['attempt']))
        p=paints.get((r['window'],a['draw'])) if a else None
        if not p:continue
        key=(r['window'],r['drawable']);gpu=completed.get(key)
        frames.append((p['version'],r['mach']*scale,gpu['gpu_end_host'] if gpu and gpu['gpu_end_host']>0 else None,displayed.get(key)))
    inputs=[(r['mach']*scale,r['version']) for r in events if r['event']=='handled' and start<=r['mach']<=end]
    blocked=[r['mach']*scale for r in events if r.get('kind')=='g5_capacity_blocked']
    refreshes=[r['timestamp'] for r in data['G5_REFRESH '] if start*scale<=r['timestamp']<=end*scale]
    return diagnose(refreshes,inputs,frames,blocked,floor_frames)
