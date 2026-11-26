"""Frozen native motion matrix for G5-4. Run from the checkout root."""
import argparse
from bisect import bisect_right
from collections import Counter
import csv
import json
import os
from pathlib import Path
from statistics import median
import subprocess
import sys
import time

sys.path.insert(0, str(Path.cwd() / 'scripts'))
import g5_latency

ROOT = Path(__file__).resolve().parents[1] / "target/headstart/g5-4"
ROOT.mkdir(parents=True, exist_ok=True)
BINARY = ROOT.parent / 'g5-1/RustUI Lab.app/Contents/MacOS/rustui-lab'
SCENES = ('drag', 'scroll', 'hover')
RATES = (120, 240)
QUIET_RECHECK_SECONDS = 30


def records(path, prefix):
    with path.open() as f:
        for line in f:
            if line.startswith(prefix):
                yield json.loads(line[len(prefix):])


def quantiles(xs):
    xs = sorted(xs)
    return [median(xs), xs[(95 * len(xs) + 99) // 100 - 1]] if xs else [None, None]


def analyse(directory):
    capture = directory / 'capture.log'
    summary_path = directory / 'summary.json'
    summary = json.loads(summary_path.read_text()) if summary_path.exists() else {}
    events = list(records(capture, 'RUSTUI_PX4_LOG '))
    clock = next(e for e in events if e['event'] == 'clock')
    scale = clock['numer'] / clock['denom'] / 1e9
    markers = list(records(capture, 'RUSTUI_G0_LOG '))
    start = next(e for e in markers if e['event'] == 'start')
    end = next(e for e in markers if e['event'] == 'end')
    native = {e['version']:e for e in records(capture, 'L1_NATIVE ') if e['event'] == 'native' and 1 <= e['version'] <= end['ticks']}
    posted = sorted(records(capture, 'G54_POST '), key=lambda e:e['mach'])
    post_times = [p['mach'] for p in posted]
    contiguous = [p['version'] for p in posted] == list(range(1, 1 + len(posted)))
    frames = sorted(records(capture, 'G5_PIXELS '), key=lambda f:f['display_mach'])
    draw_events = {(e['window'], e['draw']): e['mach'] for e in events if e['event'] == 'draw'}
    painted = {}
    for e in events:
        if e['event'] != 'painted':
            continue
        draw = draw_events.get((e['window'], e['draw']))
        if draw is not None:
            painted.setdefault(e['version'], []).append(draw)
    for values in painted.values():
        values.sort()
    drawn = {r['version']:r['draw_start'] / scale for r in summary.get('rows', [])}
    previous_display, previous_version = None, 0
    missed_slots = 0
    order = 0
    rows = []
    for frame in frames:
        v, t = frame['version'], frame['display_mach']
        if not start['mach'] <= t <= end['mach']:
            continue
        order += v < previous_version
        if v not in native:
            continue
        event = native[v]
        draw = drawn.get(v)
        if draw is None and v in painted:
            j = bisect_right(painted[v], t)
            draw = painted[v][j - 1] if j else None
        display_s = t * scale
        if previous_display is not None and previous_version < bisect_right(post_times, previous_display):
            missed_slots += max(0, round((t - previous_display) * scale * 120) - 1)
        shown = max(0, bisect_right(post_times, t) - v) if contiguous else None
        last_posted = bisect_right(post_times, draw) if contiguous and draw is not None else None
        waiting = max(0, last_posted - v) if last_posted is not None else None
        cocoa_queued = None if waiting is None else sum(
            version not in native or native[version]['handler_mach'] > draw
            for version in range(v + 1, last_posted + 1)
        )
        rows.append(dict(version=v, displayed_s=display_s, age_ms=1000*(display_s - event['hardware_s']),
                         handler_to_sck_ms=1000*(display_s - event['handler_mach']*scale),
                         newer_at_build=waiting, newer_queued_in_cocoa=cocoa_queued, newer_at_display=shown))
        previous_display, previous_version = t, v
    with (directory / 'frames.csv').open('w') as output:
        writer = csv.DictWriter(output, fieldnames=['version','displayed_s','age_ms','handler_to_sck_ms','newer_at_build','newer_queued_in_cocoa','newer_at_display'])
        writer.writeheader()
        writer.writerows(rows)
    phases = {1: 'callback', 2: 'timer', 3: 'commit', 4: 'gpu'}
    configs = [e for e in events if e['event'] == 'config' and start['mach'] <= e['mach'] <= end['mach']]
    starts = sum(e.get('kind') == 'g54_deadline_started' for e in configs)
    misses = Counter(phases[int(e['values'][0])] for e in configs if e.get('kind') == 'g54_deadline_missed')
    final = next(records(capture, 'G54_FINAL '), {})
    committed = {(e['window'], e['drawable']) for e in events if e['event'] == 'committed' and start['mach'] <= e['mach'] <= end['mach']}
    presented = {(e['window'], e['drawable']) for e in events if e['event'] == 'displayed' and e['presented_host'] > 0}
    layer = [e for e in events if e.get('kind') == 'metal_layer' and e['mach'] <= end['mach']]
    layer_before = [e for e in layer if e['mach'] <= start['mach']]
    sync_known = bool(layer_before) and all(e['values'][1] == 1 for e in [layer_before[-1], *(e for e in layer if e['mach'] > start['mach'])])
    latched_rows = [r for r in summary.get('rows', []) if start['mach']*scale <= r['presented'] <= end['mach']*scale]
    motion_presented = [r['hardware_presented_total'] for r in latched_rows]
    with (directory/'acquisition-outliers.json').open('w') as out:
        json.dump(summary.get('next_drawable_outliers', []),out,indent=2)
    data = dict(scene=start['scene'], nominal_hz=start['input_hz'], posts=len(posted),
                duration_s=(end['mach']-start['mach'])*scale, delivered=len(native), frames=len(rows),
                actual_hz=(len(posted)-1)*1.0/((post_times[-1]-post_times[0])*scale) if len(post_times)>1 else None,
                input_age_ms=quantiles([r['age_ms'] for r in rows]),
                handler_sck_ms=quantiles([r['handler_to_sck_ms'] for r in rows]),
                newer_build=quantiles([r['newer_at_build'] for r in rows if r['newer_at_build'] is not None]),
                newer_cocoa=quantiles([r['newer_queued_in_cocoa'] for r in rows if r['newer_queued_in_cocoa'] is not None]),
                newer_display=quantiles([r['newer_at_display'] for r in rows if r['newer_at_display'] is not None]),
                missing_draw_joins=sum(r['newer_at_build'] is None for r in rows),
                estimated_missed_sck_slots=missed_slots, sck_order=order, metal_order=summary.get('out_of_order'),
                native_net=sum(e.get('scroll_y',0.0) for e in native.values()), posted_net=sum(p['delta'] for p in posted),
                native_scroll_abs=sum(abs(e.get('scroll_y',0.0)) for e in native.values()),
                posted_scroll_abs=sum(abs(p['delta']) for p in posted),
                last_pixel_version=max((f['version'] for f in frames),default=0),
                final=final, deadline_starts=starts, deadline_misses=misses, sync_on=sync_known,
                hardware_presented_ms=[*quantiles(motion_presented),max(motion_presented,default=None)],
                gpu_to_presented_ms=quantiles([1000*(r['presented']-r['gpu']) for r in latched_rows])[0],
                distinct_versions=len({r['version'] for r in rows}), rendered_never_shown=len(committed-presented),
                sck_gap_ms=quantiles([1000*(b['displayed_s']-a['displayed_s']) for a,b in zip(rows,rows[1:])]),
                outliers_gt1=summary.get('next_drawable_over_1_ms_all'),
                process_cpu_s_per_s=(end['cpu_user_s']+end['cpu_sys_s'])/((end['mach']-start['mach'])*scale),
                load=summary.get('machine_load'), p95=summary.get('stats',{}).get('P',{}).get('p95'))
    (directory/'motion.json').write_text(json.dumps(data,indent=2)+'\n')
    return data


def quiet(name):
    limit = time.monotonic() + QUIET_RECHECK_SECONDS
    for index in range(2):
        result = g5_latency.collect_load(str(ROOT/f'{name}-load-{index}.txt'),[str(BINARY)],None)
        if not result['consistent']:
            raise RuntimeError(f'inconsistent quiet sample {name}')
        print(name, 'prepair background', result['background_cpu_percent'], flush=True)
        if result['background_cpu_percent'] <= 5 or index == 1:
            return result['background_cpu_percent']
        time.sleep(max(0, limit - time.monotonic()))


def run(name, scene, rate, arm, seconds, p95=None, prepair=None):
    print(time.strftime('%H:%M:%S UTC', time.gmtime()), name, flush=True)
    flags = dict(os.environ, RUSTUI_G5_PROFILE='1', GPUI_FAST_NO_DISPLAY_SYNC='0',
                 RUSTUI_G54_TRACE='1', RUSTUI_G54_INPUT_HZ=str(rate),
                 RUSTUI_G5_LATCH_LATEST=str(int(arm == 'latest')),
                 RUSTUI_G5_LATCH_DEADLINE=str(int(arm == 'deadline')))
    if p95 is not None:
        flags['RUSTUI_G54_P95_MS'] = str(p95)
    cmd = [sys.executable, 'scripts/run_g5_budget.py','--app','lab','--scene',scene,'--launch','direct',
           '--external-observer','--seconds',str(seconds),'--scheduler','0','--input-scoped','0',
           '--switches','0000','--hl-fastpath','0','--passive','0','--pixels','1','--target-hover',
           '--output',str(ROOT/name)]
    with (ROOT/f'{name}.driver').open('w') as output:
        exitcode = subprocess.call(cmd, env=flags, stdout=output, stderr=subprocess.STDOUT)
    try:
        data = analyse(ROOT/name)
    except (AssertionError, IndexError, KeyError, OSError, StopIteration) as exc:
        print('invalid capture', name, 'exit', exitcode, repr(exc), flush=True)
        return
    metadata = ROOT / name / 'method.json'
    if metadata.exists():
        frozen_build = json.loads(metadata.read_text())['frozen']
        freeze = ROOT / ('baseline-build.json' if name.startswith('baseline-') else 'candidate-build.json')
        if freeze.exists() and json.loads(freeze.read_text()) != frozen_build:
            raise RuntimeError(f'{name} used a different release build')
        freeze.write_text(json.dumps(frozen_build, indent=2) + '\n')
    data['runner_exit'] = exitcode
    data['prepair_background'] = prepair
    (ROOT/name/'motion.json').write_text(json.dumps(data,indent=2)+'\n')
    print(json.dumps(data), flush=True)
    return data


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode', choices=('baseline', 'smoke', 'matrix'))
    parser.add_argument('seconds', type=int, help='Measured seconds per capture, after warmup')
    parser.add_argument('--scene', action='append', choices=SCENES)
    parser.add_argument('--rate', action='append', type=int, choices=RATES)
    parser.add_argument('--pair', action='append', type=int, choices=range(1, 6),
                        help='Run selected pair indices and preserve their alternating order.')
    parser.add_argument('--arm', action='append', choices=('latest', 'deadline'),
                        help='Limit the enabled candidates. Off is always the paired control.')
    parser.add_argument('--stop-before', type=float, help='Unix deadline for the run, in seconds')
    args = parser.parse_args()
    mode, seconds = args.mode, args.seconds
    chosen = list(dict.fromkeys(args.arm or ('latest', 'deadline')))
    scenarios = [(s, r) for s in dict.fromkeys(args.scene or SCENES)
                 for r in dict.fromkeys(args.rate or RATES)]
    if not 1 <= seconds <= 270:
        parser.error('use 1–270 seconds to stay within the motion barcode range')
    if mode in ('baseline','smoke'):
        arms = ('off',) if mode == 'baseline' else chosen
        valid = True
        for scene, rate in scenarios:
            for arm in arms:
                p95 = None if mode == 'baseline' else json.loads((ROOT/f'baseline-{scene}-{rate}-off/motion.json').read_text())['p95']
                capture = run(f'{mode}-{scene}-{rate}-{arm}',scene,rate,arm,seconds,p95)
                valid = bool(capture and capture['runner_exit'] == 0 and capture['sync_on']
                             and capture['frames'] and capture['p95'] is not None) and valid
        return 0 if valid else 1

    for scene, rate in scenarios:
        p95 = json.loads((ROOT/f'baseline-{scene}-{rate}-off/motion.json').read_text())['p95']
        for pair in dict.fromkeys(args.pair or range(1, 6)):
            if args.stop_before and time.time() + QUIET_RECHECK_SECONDS + (1 + len(chosen)) * (seconds + 12) > args.stop_before:
                print('Time remaining cannot fit another complete pair.', flush=True)
                return 1
            prefix = f'{scene}-{rate}-{pair}'
            prepair = quiet(prefix)
            order = ['off', *chosen]
            if pair % 2 == 0:
                order.reverse()
            for arm in order:
                run(f'{prefix}-{arm}',scene,rate,arm,seconds,p95,prepair)
    return 0


if __name__ == '__main__':
    sys.exit(main())
