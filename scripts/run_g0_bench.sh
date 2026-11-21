#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repository_root"
. scripts/pinned-toolchain.sh
export CARGO_TARGET_DIR="$HOME/.cache/rustui-target"
g0_scenes=(idle typing hover log drag)

g0_build() {
  pinned_cargo build -p rustui-lab --release --locked --offline
}

g0_launch() {
  RUSTUI_DAMAGE="${RUSTUI_DAMAGE:-0}" RUSTUI_FRAME_LOG=buffered RUSTUI_FRAME_PROFILE=1 \
    "$CARGO_TARGET_DIR/release/rustui-lab" --g0-scene "$1" --duration 60 > "$2" 2>&1 &
  g0_app_pid=$!
}

g0_wait() {
  wait "$g0_app_pid"
  g0_app_pid=
  # GPUI's native quit can terminate the process before run_cli returns.
  # A successful process exit alone therefore cannot prove completion.
  python3 - "$1" <<'PY'
import json, sys
from pathlib import Path
markers = [json.loads(line.removeprefix('RUSTUI_G0_LOG '))
           for line in Path(sys.argv[1]).read_text().splitlines()
           if line.startswith('RUSTUI_G0_LOG ')]
assert [m['event'] for m in markers] == ['start', 'end'], 'scene did not finish; inspect frame log'
assert markers[-1]['valid'], markers[-1]
PY
}

g0_summarize() {
  python3 - "$1" "${RUSTUI_DAMAGE:-0}" "${@:2}" <<'PY'
import collections, json, math, os, statistics, sys
from pathlib import Path
root = Path(sys.argv[1])
damage = sys.argv[2]
backoff = os.environ.get("RUSTUI_DAMAGE_BACKOFF", "0")
assert backoff in ("0", "1")
assert damage in ("0", "1", "verify"), "RUSTUI_DAMAGE must be 0, 1 or verify"
results = []
for scene in sys.argv[3:] or ('idle', 'typing', 'hover', 'log', 'drag'):
    markers, records = [], []
    for line in (root / f'{scene}-frames.log').read_text().splitlines():
        if line.startswith('RUSTUI_G0_LOG '):
            markers.append(json.loads(line.removeprefix('RUSTUI_G0_LOG ')))
        elif line.startswith('RUSTUI_FRAME_LOG '):
            records.append(json.loads(line.removeprefix('RUSTUI_FRAME_LOG ')))
    starts = [m for m in markers if m['event'] == 'start']
    ends = [m for m in markers if m['event'] == 'end']
    assert len(starts) == len(ends) == 1, (scene, 'missing measurement markers')
    start, end = starts[0], ends[0]
    assert end['valid'] and start['scene'] == end['scene'] == scene, (scene, end)
    seconds = (end['epoch_ms'] - start['epoch_ms']) / 1000
    assert 59.9 <= seconds <= 62, (scene, seconds)
    assert not any(r['event'] == 'damage_mismatch' for r in records), (scene, 'damage mismatch anywhere in scene log')
    records = [r for r in records if start['epoch_ms'] <= r['epoch_ms'] <= end['epoch_ms']]
    presents = [r for r in records if r['event'] == 'gpu']
    modes = collections.Counter('direct' if r['mode'] == 'full' else r['mode'] for r in presents)
    allowed = ('skip', 'copy', 'partial', 'retained', 'direct')
    assert set(modes) <= set(allowed), (scene, modes)
    assert len(presents) >= (20 if scene == 'idle' else 100), (scene, 'too few frame-mode samples')
    if damage == '0':
        assert modes['direct'] == len(presents), (scene, 'damage off must draw Direct', modes)
    for record in presents:
        if record['mode'] == 'skip':
            assert record['gpu_ms'] is None, (scene, 'Skip must have no GPU time')
        else:
            assert record['gpu_ms'] is not None, (scene, 'submitted frame needs GPU time')
    gpu = [r['gpu_ms'] for r in presents if r['mode'] != 'skip']
    assert all(math.isfinite(n) and n >= 0 for n in gpu)
    damage_records = [r for r in records if r['event'] == 'damage_cpu']
    damage_cpu = dict(samples=len(damage_records),
                      signatures_mean_ms=statistics.mean(r['signatures_ms'] for r in damage_records) if damage_records else 0.0,
                      plan_mean_ms=statistics.mean(r['plan_ms'] for r in damage_records) if damage_records else 0.0,
                      total_mean_ms=statistics.mean(r['total_ms'] for r in damage_records) if damage_records else 0.0,
                      hashed=sum(r['hashed'] for r in damage_records), reused=sum(r['reused'] for r in damage_records),
                      reasons=dict(collections.Counter(r['reason'] for r in damage_records)))
    phases = {}
    for r in records:
        if r['event'] not in ('gpu', 'damage_cpu', 'encode'):
            phases.setdefault(r['event'], []).append(r['cpu_ms'])
    frames, draw = [], 0.0
    for r in records:
        if r['event'] == 'draw':
            draw = r['cpu_ms']
        elif r['event'] == 'present':
            frames.append(draw + r['cpu_ms'])
            draw = 0.0
    assert frames, (scene, 'missing CPU presents')
    user_s, sys_s = end.get('cpu_user_s'), end.get('cpu_sys_s')
    assert all(isinstance(v, (float, int)) and math.isfinite(v) and v >= 0 for v in (user_s, sys_s)), (scene, 'missing or invalid getrusage window')
    process_cpu_s = user_s + sys_s
    assert process_cpu_s > 0, (scene, 'empty process CPU measurement')
    encoding = [r for r in records if r['event'] == 'encode']
    assert len(encoding) >= (20 if scene == 'idle' else 100), (scene, 'missing instance encoding samples')
    assert all(r['mode'] in allowed and r['instances_encoded'] >= 0 and r['instances_total'] >= 0 for r in encoding), (scene, 'invalid encoding sample')
    mode_runs = []
    for r in sorted(encoding, key=lambda r: r['sequence']):
        if mode_runs and mode_runs[-1]['mode'] == r['mode']:
            mode_runs[-1]['frames'] += 1
        else:
            mode_runs.append(dict(mode=r['mode'], frames=1))
    planning_runs = []
    for r in sorted(encoding, key=lambda r: r['sequence']):
        label = r.get('reason', r['mode'])
        assert label == r['mode'] or (label == 'backoff' and r['mode'] == 'direct'), (scene, r)
        if planning_runs and planning_runs[-1]['mode'] == label:
            planning_runs[-1]['frames'] += 1
        else:
            planning_runs.append(dict(mode=label, frames=1))
    instances = dict(samples=len(encoding), encoded=sum(r['instances_encoded'] for r in encoding),
                     total=sum(r['instances_total'] for r in encoding), mode_runs=mode_runs,
                     planning_runs=planning_runs, backoff_frames=sum(r.get('reason') == 'backoff' for r in encoding))
    results.append(dict(scene=scene, seconds=seconds, damage=damage, backoff=backoff, window=start, workload=end,
                        damage_cpu=damage_cpu, instances=instances,
                        process_cpu_user_s=user_s, process_cpu_sys_s=sys_s,
                        process_cpu_s=process_cpu_s, process_cpu_s_per_attempt=process_cpu_s/len(frames),
                        frame_mode_counts={mode: modes[mode] for mode in allowed},
                        frame_mode_samples=len(presents),
                        gpu_samples=len(gpu), gpu_mean_ms=statistics.mean(gpu) if gpu else 0.0,
                        gpu_p95_ms=sorted(gpu)[math.ceil(len(gpu)*.95)-1] if gpu else None,
                        gpu_ms_per_second=sum(gpu)/seconds,
                        cpu_present_samples=len(frames), cpu_mean_ms=statistics.mean(frames),
                        frames_over_8_3_ms=sum(value > 8.3 for value in frames),
                        cpu_phases_mean_ms={k: statistics.mean(v) for k,v in phases.items()}))
(root / 'summary.json').write_text(json.dumps(results, indent=2) + '\n')
for row in results:
    print(f"{row['scene']:7} GPU {row['gpu_mean_ms']:.4f} ms/present "
          f"CPU {row['cpu_mean_ms']:.4f} ms/attempt "
          f"process CPU {row['process_cpu_s']:.6f} s ({row['process_cpu_s_per_attempt']:.9f} s/attempt) "
          f">8.3 ms={row['frames_over_8_3_ms']} "
          f"instances {row['instances']['encoded']}/{row['instances']['total']} "
          f"signatures {row['damage_cpu']['signatures_mean_ms']:.4f} ms plan {row['damage_cpu']['plan_mean_ms']:.4f} ms "
          f"modes={row['frame_mode_counts']}")
PY
}

g0_output() {
  g0_dir="${1:-target/headstart/performance/g0-$(date -u +%Y%m%dT%H%M%SZ)}"
  mkdir "$g0_dir"
  g0_dir="$(cd "$g0_dir" && pwd)"
}

g0_summarize_power() {
  python3 - "$1" <<'PY'
import json, re, statistics, sys
from pathlib import Path
root = Path(sys.argv[1])
rows = []
for scene in ('idle', 'typing', 'hover', 'log', 'drag'):
    text = (root / f'{scene}-power.txt').read_text()
    row = {'scene': scene}
    for component in ('CPU', 'GPU'):
        samples = [float(value) * (1000 if unit == 'W' else 1)
                   for value, unit in re.findall(
                       rf'^\s*{component} Power:\s*([0-9.]+)\s*(mW|W)\s*$', text, re.M)]
        if not samples:
            raise SystemExit(f'{scene}: missing {component} power samples; inspect raw output')
        row[f'{component.lower()}_mean_mw'] = statistics.mean(samples)
        row[f'{component.lower()}_samples'] = len(samples)
    rows.append(row)
(root / 'power-summary.json').write_text(json.dumps(rows, indent=2) + '\n')
for row in rows:
    print(f"{row['scene']:7} CPU {row['cpu_mean_mw']:.2f} mW GPU {row['gpu_mean_mw']:.2f} mW")
PY
}

g0_interleaved_summary() {
  python3 - "$1" "${2:-damage}" <<'PY'
import json, math, statistics, sys
from pathlib import Path
root = Path(sys.argv[1])
comparison = sys.argv[2]
assert comparison in ('damage', 'backoff')
rows = []
for scene in (('log', 'hover') if comparison == 'backoff' else ('idle', 'typing', 'hover', 'log', 'drag')):
    groups = {}
    for mode in ('0', '1'):
        records = [json.loads((root / scene / f'{run}-{mode}' / 'summary.json').read_text())
                   for run in range(1, 6)]
        assert all(len(r) == 1 and r[0]['scene'] == scene and r[0][comparison] == mode and (comparison != 'backoff' or r[0]['damage'] == '1') for r in records)
        values = [r[0]['cpu_mean_ms'] for r in records]
        assert all(math.isfinite(value) and value > 0 for value in values)
        median = statistics.median(values)
        groups[mode] = dict(runs=values, median_ms=median)
    off, on = groups['0'], groups['1']
    process = {}
    for mode in ('0', '1'):
        captures = [json.loads((root / scene / f'{run}-{mode}' / 'summary.json').read_text())[0] for run in range(1, 6)]
        values = [r['process_cpu_s_per_attempt'] for r in captures]
        assert all(math.isfinite(v) and v > 0 for v in values)
        process[mode] = dict(total_s=[r['process_cpu_s'] for r in captures],
                             s_per_attempt=values, median_s_per_attempt=statistics.median(values))
    process_changes = [(b/a-1)*100 for a,b in zip(process['0']['s_per_attempt'], process['1']['s_per_attempt'])]
    process.update(paired_changes_pct=process_changes, median_change_pct=statistics.median(process_changes),
                   min_change_pct=min(process_changes), max_change_pct=max(process_changes))
    changes = [(b / a - 1) * 100 for a, b in zip(off['runs'], on['runs'])]
    passes = [b <= a * 1.03 for a, b in zip(off['runs'], on['runs'])]
    verdict = 'PASS' if all(passes) else 'FAIL' if not any(passes) else 'INCONCLUSIVE'
    row = dict(scene=scene, off=off, on=on, paired_changes_pct=changes,
               process_cpu=process, median_change_pct=statistics.median(changes),
               min_change_pct=min(changes), max_change_pct=max(changes), verdict=verdict)
    rows.append(row)
    print(f"{scene:7} median off={off['median_ms']:.6f} on={on['median_ms']:.6f} ms/attempt "
          f"paired changes={[round(c, 3) for c in changes]} "
          f"median={row['median_change_pct']:+.3f}% min={min(changes):+.3f}% "
          f"max={max(changes):+.3f}% {verdict} "
          f"process paired={[round(c, 3) for c in process_changes]} "
          f"median={statistics.median(process_changes):+.3f}% min={min(process_changes):+.3f}% max={max(process_changes):+.3f}%", flush=True)
(root / ('backoff-summary.json' if comparison == 'backoff' else 'interleaved-summary.json')).write_text(json.dumps(rows, indent=2) + '\n')
PY
}

g0_interleaved() {
  local scene run mode capture
  local -a modes
  shasum -a 256 "$CARGO_TARGET_DIR/release/rustui-lab" > "$g0_dir/binary.sha256"
  for scene in "${g0_scenes[@]}"; do
    for run in 1 2 3 4 5; do
      if (( run % 2 )); then modes=(0 1); else modes=(1 0); fi
      for mode in "${modes[@]}"; do
        capture="$g0_dir/$scene/$run-$mode"
        mkdir -p "$capture"
        echo "Matched $scene pair $run/5 damage=$mode (60 seconds)"
        echo "$scene $run $mode" >> "$g0_dir/run-order.txt"
        RUSTUI_DAMAGE="$mode" g0_launch "$scene" "$capture/$scene-frames.log"
        g0_wait "$capture/$scene-frames.log"
        RUSTUI_DAMAGE="$mode" g0_summarize "$capture" "$scene"
      done
    done
  done
  shasum -a 256 --check "$g0_dir/binary.sha256"
  g0_interleaved_summary "$g0_dir"
}

g0_backoff_pairs() {
  local scene run mode capture
  local -a modes
  shasum -a 256 "$CARGO_TARGET_DIR/release/rustui-lab" > "$g0_dir/binary.sha256"
  for scene in log hover; do
    for run in 1 2 3 4 5; do
      if (( run % 2 )); then modes=(0 1); else modes=(1 0); fi
      for mode in "${modes[@]}"; do
        capture="$g0_dir/$scene/$run-$mode"
        mkdir -p "$capture"
        echo "Matched F $scene pair $run/5 backoff=$mode damage=1 (60 seconds)"
        echo "$scene $run $mode" >> "$g0_dir/run-order.txt"
        RUSTUI_DAMAGE_BACKOFF="$mode" RUSTUI_DAMAGE=1 g0_launch "$scene" "$capture/$scene-frames.log"
        g0_wait "$capture/$scene-frames.log"
        RUSTUI_DAMAGE_BACKOFF="$mode" RUSTUI_DAMAGE=1 g0_summarize "$capture" "$scene"
      done
    done
  done
  shasum -a 256 --check "$g0_dir/binary.sha256"
  g0_interleaved_summary "$g0_dir" backoff
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  g0_interleave=0
  g0_backoff=0
  if [[ "${1:-}" == "--backoff-pairs" ]]; then g0_backoff=1; shift; fi
  if [[ "${1:-}" == "--interleaved" ]]; then g0_interleave=1; shift; fi
  g0_build
  mkdir -p target/headstart/performance
  g0_output "${1:-}"
  trap 'if [[ -n "${g0_app_pid:-}" ]]; then kill "$g0_app_pid" 2>/dev/null || true; fi' EXIT INT TERM
  if [[ "$g0_backoff" == 1 ]]; then
    g0_backoff_pairs
    echo "Results: $g0_dir"
    exit 0
  fi
  if [[ "$g0_interleave" == 1 ]]; then
    g0_interleaved
    echo "Results: $g0_dir"
    exit 0
  fi
  for scene in "${g0_scenes[@]}"; do
    echo "Measuring $scene for 60 seconds"
    g0_launch "$scene" "$g0_dir/$scene-frames.log"
    g0_wait "$g0_dir/$scene-frames.log"
  done
  g0_summarize "$g0_dir"
  echo "Results: $g0_dir"
fi
