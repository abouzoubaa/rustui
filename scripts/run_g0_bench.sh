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
  RUSTUI_DAMAGE=0 RUSTUI_FRAME_LOG=1 RUSTUI_FRAME_PROFILE=1 \
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
  python3 - "$1" <<'PY'
import json, math, statistics, sys
from pathlib import Path
root = Path(sys.argv[1])
results = []
for scene in ('idle', 'typing', 'hover', 'log', 'drag'):
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
    records = [r for r in records if start['epoch_ms'] <= r['epoch_ms'] <= end['epoch_ms']]
    gpu = [r['gpu_ms'] for r in records if r['event'] == 'gpu']
    assert len(gpu) >= (20 if scene == 'idle' else 100), (scene, 'too few GPU samples')
    assert all(r['mode'] in ('full', 'direct') for r in records if r['event'] == 'gpu'), (scene, 'expected whole-frame mode')
    assert all(math.isfinite(n) and n >= 0 for n in gpu)
    phases = {}
    for r in records:
        if r['event'] != 'gpu':
            phases.setdefault(r['event'], []).append(r['cpu_ms'])
    frames, draw = [], 0.0
    for r in records:
        if r['event'] == 'draw':
            draw = r['cpu_ms']
        elif r['event'] == 'present':
            frames.append(draw + r['cpu_ms'])
            draw = 0.0
    assert frames, (scene, 'missing CPU presents')
    results.append(dict(scene=scene, seconds=seconds, window=start, workload=end,
                        gpu_samples=len(gpu), gpu_mean_ms=statistics.mean(gpu),
                        gpu_p95_ms=sorted(gpu)[math.ceil(len(gpu)*.95)-1],
                        gpu_ms_per_second=sum(gpu)/seconds,
                        cpu_present_samples=len(frames), cpu_mean_ms=statistics.mean(frames),
                        cpu_phases_mean_ms={k: statistics.mean(v) for k,v in phases.items()}))
(root / 'summary.json').write_text(json.dumps(results, indent=2) + '\n')
for row in results:
    print(f"{row['scene']:7} GPU {row['gpu_mean_ms']:.4f} ms/present "
          f"CPU {row['cpu_mean_ms']:.4f} ms/present ({row['gpu_samples']} whole GPU frames)")
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
