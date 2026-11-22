#!/usr/bin/env python3
"""Frozen-build G5-2 source comparisons; never compile while capturing."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))
import g5_latency as metrics

ARMS = {'baseline': ('0', '0', '0'),
        'view': ('0', '1', '0'), 'legacy': ('0', '0', '1')}


def capture(output, comparison, application, scene, hz, pair, arm, seconds, env):
    stem = f'{comparison}-{application}-{scene}-{hz}-{pair}-{arm}'
    log = output / (stem + '.log')
    if log.exists():
        raise RuntimeError(f'Refusing to overwrite {log}')
    bare = ROOT / 'target/headstart/g5-1/G5 Bare.app/Contents/MacOS/latency-gpui'
    lab = Path.home() / '.cache/rustui-target/release/rustui-lab'
    binary = bare if application == 'bare' else lab
    ahead, view, legacy = ARMS[arm]
    flags = dict(env, GPUI_FAST_FRAME_AHEAD=ahead, GPUI_FAST_VIEW_DISPLAY_LINK=view,
                 GPUI_FAST_FORCE_LEGACY_LINK=legacy, GPUI_FAST_IMMEDIATE_PRESENT='1',
                 GPUI_FAST_IMMEDIATE_HOLD_MS='250', GPUI_FAST_IMMEDIATE_TRANSACTION='0',
                 RUSTUI_G5_SCENE=scene, RUSTUI_G5_INPUT_HZ=str(hz), RUSTUI_G5_PIXEL_CAPTURE='0')
    if application == 'lab':
        flags['DYLD_INSERT_LIBRARIES'] = str(ROOT / 'target/headstart/g5-1/libG5Refresh.dylib')
    else:
        flags.pop('DYLD_INSERT_LIBRARIES', None)
    print(f'{stem} start {datetime.now(timezone.utc).isoformat()}', flush=True)
    top = output / (stem + '.top-summary.txt')
    with top.open('w') as stream:
        subprocess.run(['top', '-l', '1', '-o', 'cpu', '-n', '5', '-stats', 'pid,ppid,cpu,command'], stdout=stream, check=True)
    # The owner explicitly bypassed the quiet CPU gate. Preserve top five and
    # corrected start/end load; do not silently substitute a 25% limit.
    before = metrics.collect_load(str(output / (stem + '.top-start.txt')), [str(binary)], None)
    assert before['accepted'], before
    command = [str(binary), str(seconds)] if application == 'bare' else [str(binary), '--g0-scene', scene, '--duration', str(seconds)]
    with log.open('w') as stream:
        result = subprocess.run(command, cwd=ROOT, env=flags, stdout=stream, stderr=subprocess.STDOUT)
    if result.returncode:
        raise RuntimeError(f'{stem} exited {result.returncode}; inspect {log}')
    after = metrics.collect_load(str(output / (stem + '.top-end.txt')), [str(binary)], None)
    assert after['accepted'], after
    floor = os.environ.get("RUSTUI_G5_FLOOR_" + str(hz))
    summary = metrics.summarize(log, int(floor) if floor is not None else 1 if hz == 10 else 2 if hz == 30 else None)
    metrics.annotate_load(summary, before, after)
    summary["machine_load"]["top_five_start"] = before["top_processes"]
    expected_source={'baseline':1,'view':2,'legacy':0}[arm]
    assert summary['frame_source_selections'] and all(int(v[0])==expected_source for v in summary['frame_source_selections']), (stem,'requested source did not run',summary['frame_source_selections'])
    if arm=='view':
        raw=metrics.records(log)['RUSTUI_PX4_LOG ']
        assert not any(r.get('kind') in ('g5_link_stop','g5_link_recreate','g5_hold_expired') for r in raw), (stem,'view path used Metal attach/detach or hold timer')
    summary.update(comparison=comparison, application=application, scene=scene, input_hz=hz, pair=pair, arm=arm)
    (output / (stem + '.json')).write_text(json.dumps(summary, indent=2) + '\n')
    assert summary['final_text_valid'] and summary['versions_monotonic'] and summary['out_of_order_presentations']==0, stem
    assert summary['last_version_shown'] == summary['expected_final_version'], stem
    print(f'{stem} complete; shown/s={summary["distinct_versions_per_second"]:.3f}; '
          f'CPU={1000*summary["cpu_s_per_attempt"]:.3f} ms/attempt; '
          f'nextDrawable>2ms={summary["next_drawable_over_2_ms"]}', flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--deadline', help='UTC ISO timestamp; reserve time to push/report')
    parser.add_argument('--functional', action='store_true')
    args = parser.parse_args()
    if os.environ.get('RUSTUI_G5_REFERENCE') != '1':
        parser.error('Set RUSTUI_G5_REFERENCE=1 for local controls')
    args.output.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, RUSTUI_G5_REFERENCE='1', RUSTUI_DAMAGE='0', RUSTUI_FRAME_LOG='buffered',
               RUSTUI_FRAME_PROFILE='1', RUSTUI_G0_LATENCY='1', RUSTUI_LATENCY_BREAKDOWN='1',
               RUSTUI_G5_REFRESH='1', RUSTUI_HL_FASTPATH='0')
    for key in ('RUSTUI_G5_ASYNC_WAKE', 'RUSTUI_G5_FULLSCREEN', 'RUSTUI_G5_WARMUP_S', 'MTL_HUD_ENABLED'):
        env.pop(key, None)
    paths = [ROOT / 'target/headstart/g5-1/G5 Bare.app/Contents/MacOS/latency-gpui',
             ROOT / 'target/headstart/g5-1/libG5Refresh.dylib',
             ROOT / 'target/headstart/g5-1/libG5Pixels.dylib',
             Path.home() / '.cache/rustui-target/release/rustui-lab']
    frozen = {str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in paths}
    (args.output / 'frozen-build.json').write_text(json.dumps(frozen, indent=2))
    if args.functional:
        groups = [('functional', 'bare', scene, 10 if scene=='typing' else 120, ('baseline','view','legacy'))
                  for scene in ('hover','typing','drag')]
    else:
        groups = [('view', 'bare', scene, hz, ('baseline','view'))
                  for scene,hz in [('hover',120),('drag',120),('typing',30),('typing',10)]]
        groups += [('legacy','bare',scene,hz,('baseline','legacy'))
                   for scene,hz in [('hover',120),('typing',30)]]
    deadline = datetime.fromisoformat(args.deadline.replace('Z', '+00:00')).timestamp() if args.deadline else None
    try:
        for comparison, application, scene, hz, arms in groups:
            for pair in range(1, 2 if args.functional else 6):
                for arm in arms if pair % 2 else reversed(arms):
                    if deadline and time.time() + 85 > deadline:
                        print('Timebox: not starting another capture; completed data retained.', flush=True)
                        return 75
                    capture(args.output, comparison, application, scene, hz, pair, arm, 5 if args.functional else 60, env)
        print('Completed; frozen executables unchanged.', flush=True)
        return 0
    finally:
        current={str(path):hashlib.sha256(path.read_bytes()).hexdigest() for path in paths}
        (args.output/'frozen-build-after.json').write_text(json.dumps(current,indent=2))
        assert frozen==current,'build changed during timing'


if __name__ == '__main__':
    raise SystemExit(main())
