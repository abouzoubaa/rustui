"""Match handled G0 state versions to their first actual displayed drawable."""
from bisect import bisect_left
from collections import defaultdict
import math
import statistics


def summarize_latency(records, scene, start_ms, end_ms, enabled):
    result = dict(enabled=enabled, applicable=scene != 'idle', inputs=0, samples=0,
                  median_ms=None, p95_ms=None, max_ms=None, delays_over_8_3_ms=0,
                  coalesced_unpainted=0, painted_not_presented=0,
                  invalid_presented_callbacks=0)
    if not enabled or scene == 'idle':
        return result
    records = [r for r in records if r['scene'] == scene]
    inputs = [r for r in records if r['event'] == 'input' and start_ms <= r['epoch_ms'] <= end_ms]
    assert inputs, (scene, 'latency enabled but no handled inputs')
    assert len({(r['window'], r['version']) for r in inputs}) == len(inputs), 'duplicate input versions'
    painted = set()
    frames = defaultdict(list)
    paint_tags = defaultdict(list)
    for r in records:
        if r['event'] == 'paint':
            key = (r['window'], r['first_visible'], r['version'])
            paint_tags[key].append(r['mono_ms'])
            assert 0 <= r['version'] - r['first_visible'] <= (19 if scene == 'log' else 0)
            painted.update((r['window'], v) for v in range(r['first_visible'], r['version']+1))
    presented = sorted((r for r in records if r['event'] == 'presented'), key=lambda r: r['mono_ms'])
    for r in presented:
        if not r['valid'] or not math.isfinite(r['presented_time']) or r['presented_time'] <= 0:
            result['invalid_presented_callbacks'] += 1
            continue
        assert math.isfinite(r['mono_ms'])
        key = (r['window'], r['first_visible'], r['version'])
        assert key in paint_tags and min(paint_tags[key]) <= r['mono_ms'], (scene, 'drawable tag was never painted')
        for version in range(r['first_visible'], r['version']+1):
            frames[(r['window'], version)].append(r['mono_ms'])
    delays = []
    for r in inputs:
        key = (r['window'], r['version'])
        times = frames.get(key, [])
        index = bisect_left(times, r['mono_ms'])
        if index == len(times):
            result['painted_not_presented' if key in painted else 'coalesced_unpainted'] += 1
            continue
        delay = times[index] - r['mono_ms']
        assert math.isfinite(delay) and delay >= 0
        delays.append(delay)
    result['inputs'] = len(inputs)
    result['samples'] = len(delays)
    assert delays, (scene, 'no actual presented state versions')
    delays.sort()
    result.update(median_ms=statistics.median(delays),
                  p95_ms=delays[math.ceil(.95*len(delays))-1], max_ms=max(delays),
                  delays_over_8_3_ms=sum(value > 8.3 for value in delays))
    assert result['samples'] + result['coalesced_unpainted'] + result['painted_not_presented'] == len(inputs)
    return result
