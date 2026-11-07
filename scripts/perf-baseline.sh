#!/usr/bin/env bash
# Measure the existing test-window workload; this is not a native frame benchmark.
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/rustui-perf-baseline.XXXXXX")"
trap 'rm -rf "$work_dir"' EXIT
for run in 1 2 3; do
  printf 'Running workload %s/3...\n' "$run" >&2
  if rustup run 1.97.1 cargo test --release -p rustui --lib owner_update_and_draw_workload --locked --offline -- --ignored --nocapture > "$work_dir/run-$run.log" 2>&1; then
    :
  else
    status=$?
    cat "$work_dir/run-$run.log" >&2
    exit "$status"
  fi
done
python3 scripts/perf_baseline.py \
  --commit "$(git rev-parse HEAD)" \
  --date "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --rustc "$(rustup run 1.97.1 rustc --version)" \
  --macos "$(sw_vers -productVersion)" \
  --cpu "$(sysctl -n machdep.cpu.brand_string)" \
  --output perf_baseline.json \
  "$work_dir/run-1.log" "$work_dir/run-2.log" "$work_dir/run-3.log"
