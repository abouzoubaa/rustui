#!/usr/bin/env bash
# Owner-run only. Authentication happens before opening any benchmark window.
set -euo pipefail
. "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/run_g0_bench.sh"
g0_build
sudo -v
mkdir -p target/headstart/performance
g0_output "${1:-target/headstart/performance/g0-power-$(date -u +%Y%m%dT%H%M%SZ)}"
trap 'if [[ -n "${g0_app_pid:-}" ]]; then kill "$g0_app_pid" 2>/dev/null || true; fi; if [[ -n "${g0_power_pid:-}" ]]; then sudo -n kill "$g0_power_pid" 2>/dev/null || true; fi' EXIT INT TERM
for scene in "${g0_scenes[@]}"; do
  echo "Measuring $scene GPU/CPU power for 60 seconds"
  g0_launch "$scene" "$g0_dir/$scene-frames.log"
  # Wait for warm-up to finish; fail if the scene exits before measurement.
  while ! grep -q '^RUSTUI_G0_LOG {"event":"start"' "$g0_dir/$scene-frames.log"; do
    kill -0 "$g0_app_pid" 2>/dev/null || { wait "$g0_app_pid"; exit 1; }
    sleep 0.02
  done
  sudo -n powermetrics --samplers gpu_power,cpu_power -i 1000 -n 60 > "$g0_dir/$scene-power.txt" 2>&1 &
  g0_power_pid=$!
  g0_wait "$g0_dir/$scene-frames.log"
  wait "$g0_power_pid"
  g0_power_pid=
done
g0_summarize "$g0_dir"
g0_summarize_power "$g0_dir"
echo "Raw power samples and frame results: $g0_dir"
