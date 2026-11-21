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
  RUSTUI_DAMAGE="${RUSTUI_DAMAGE:-0}" RUSTUI_FRAME_LOG=buffered RUSTUI_FRAME_PROFILE=1 RUSTUI_G0_LATENCY="${RUSTUI_G0_LATENCY:-0}" \
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
sys.path.insert(0, "scripts")
from g0_latency import summarize_latency
from pathlib import Path
root = Path(sys.argv[1])
damage = sys.argv[2]
assert damage in ("0", "1", "verify"), "RUSTUI_DAMAGE must be 0, 1 or verify"
results = []
for scene in sys.argv[3:] or ('idle', 'typing', 'hover', 'log', 'drag'):
    markers, records, input_records = [], [], []
    for line in (root / f'{scene}-frames.log').read_text().splitlines():
        if line.startswith('RUSTUI_G0_LOG '):
            markers.append(json.loads(line.removeprefix('RUSTUI_G0_LOG ')))
        elif line.startswith('RUSTUI_INPUT_LOG '):
            input_records.append(json.loads(line.removeprefix('RUSTUI_INPUT_LOG ')))
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
    latency = summarize_latency(input_records, scene, start['epoch_ms'], end['epoch_ms'], os.environ.get('RUSTUI_G0_LATENCY') == '1')
    records = [r for r in records if start['epoch_ms'] <= r['epoch_ms'] <= end['epoch_ms']]
    reason_records = [r for r in records if r['event'] == 'frame_reason']
    reasons = collections.Counter(r['reason'] for r in reason_records)
    assert all(r['reason'] and r['mode'] != 'partial' for r in reason_records), (scene, 'invalid non-Partial reason')
    presents = [r for r in records if r['event'] == 'gpu']
    modes = collections.Counter('direct' if r['mode'] == 'full' else r['mode'] for r in presents)
    allowed = ('skip', 'copy', 'partial', 'retained', 'direct')
    assert set(modes) <= set(allowed), (scene, modes)
    if any(mode != 'partial' and count for mode, count in modes.items()):
        assert reason_records, (scene, 'non-Partial frames missing reasons')
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
    phases = {}
    for r in records:
        if r['event'] not in ('gpu', 'frame_reason'):
            phases.setdefault(r['event'], []).append(r['cpu_ms'])
    frames, draw = [], 0.0
    for r in records:
        if r['event'] == 'draw':
            draw = r['cpu_ms']
        elif r['event'] == 'present':
            frames.append(draw + r['cpu_ms'])
            draw = 0.0
    assert frames, (scene, 'missing CPU presents')
    process_cpu_s = end['cpu_user_s'] + end['cpu_sys_s']
    assert math.isfinite(process_cpu_s) and process_cpu_s > 0, (scene, 'missing process CPU')
    results.append(dict(scene=scene, seconds=seconds, damage=damage, window=start, workload=end,
                        input_to_screen=latency, non_partial_reasons=dict(reasons),
                        frame_mode_counts={mode: modes[mode] for mode in allowed},
                        frame_mode_samples=len(presents),
                        gpu_samples=len(gpu), gpu_mean_ms=statistics.mean(gpu) if gpu else 0.0,
                        gpu_p95_ms=sorted(gpu)[math.ceil(len(gpu)*.95)-1] if gpu else None,
                        gpu_ms_per_second=sum(gpu)/seconds,
                        cpu_present_samples=len(frames), cpu_mean_ms=statistics.mean(frames),
                        process_cpu_user_s=end['cpu_user_s'], process_cpu_sys_s=end['cpu_sys_s'],
                        process_cpu_s=process_cpu_s, process_cpu_s_per_attempt=process_cpu_s/len(frames),
                        frames_over_8_3_ms=sum(value > 8.3 for value in frames),
                        cpu_phases_mean_ms={k: statistics.mean(v) for k,v in phases.items()}))
(root / 'summary.json').write_text(json.dumps(results, indent=2) + '\n')
for row in results:
    print(f"{row['scene']:7} GPU {row['gpu_mean_ms']:.4f} ms/present "
          f"CPU {row['cpu_mean_ms']:.4f} ms/attempt "
          f"process CPU {1000*row['process_cpu_s_per_attempt']:.4f} ms/attempt "
          f">8.3 ms={row['frames_over_8_3_ms']} modes={row['frame_mode_counts']}")
    latency = row['input_to_screen']
    if latency['samples']:
        print(f"        input→screen median/p95/max {latency['median_ms']:.3f}/{latency['p95_ms']:.3f}/{latency['max_ms']:.3f} ms "
              f"shown={latency['samples']}/{latency['inputs']} coalesced={latency['coalesced_unpainted']} "
              f"painted-not-presented={latency['painted_not_presented']} delays>8.3ms={latency['delays_over_8_3_ms']}")
    else:
        print('        input→screen not measured (off or idle)')
    print(f"        non-Partial reasons={row['non_partial_reasons']}")
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

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  g0_build
  mkdir -p target/headstart/performance
  g0_output "${1:-}"
  trap 'if [[ -n "${g0_app_pid:-}" ]]; then kill "$g0_app_pid" 2>/dev/null || true; fi' EXIT INT TERM
  for scene in "${g0_scenes[@]}"; do
    echo "Measuring $scene for 60 seconds"
    g0_launch "$scene" "$g0_dir/$scene-frames.log"
    g0_wait "$g0_dir/$scene-frames.log"
  done
  g0_summarize "$g0_dir"
  echo "Results: $g0_dir"
fi
