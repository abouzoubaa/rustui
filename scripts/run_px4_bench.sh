#!/usr/bin/env bash
set -euo pipefail
repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repository_root"
# Build explicitly before timing: ./scripts/build_latency_refs.sh.
# This runner deliberately contains no compiler invocation.
export CARGO_TARGET_DIR="$HOME/.cache/rustui-target"
. scripts/run_g0_bench.sh
export RUSTUI_DAMAGE=0 RUSTUI_FRAME_LOG=buffered RUSTUI_FRAME_PROFILE=1
export RUSTUI_G0_LATENCY=1 RUSTUI_LATENCY_BREAKDOWN=1
mkdir -p target/headstart/performance
for binary in latency-gpui latency-appkit; do test -x "target/headstart/px4/$binary"; done
g0_output "${1:-target/headstart/performance/px4-$(date -u +%Y%m%dT%H%M%SZ)}"
for scene in typing hover drag log; do
  g0_launch "$scene" "$g0_dir/$scene-frames.log"
  g0_wait "$g0_dir/$scene-frames.log"
  python3 scripts/px4_latency.py "$g0_dir/$scene-frames.log" > "$g0_dir/$scene-stages.json"
done
g0_summarize "$g0_dir" typing hover drag log
"$repository_root/target/headstart/px4/latency-gpui" 60 > "$g0_dir/bare-frames.log" 2>&1
python3 scripts/px4_latency.py "$g0_dir/bare-frames.log" > "$g0_dir/bare-stages.json"
"$repository_root/target/headstart/px4/latency-appkit" 60 > "$g0_dir/appkit-frames.log" 2>&1
python3 - "$g0_dir" <<'PY'
import json,sys
from pathlib import Path
sys.path.insert(0,'scripts')
from px4_latency import summarize_appkit
root=Path(sys.argv[1])
(root/'appkit-summary.json').write_text(json.dumps(summarize_appkit(root/'appkit-frames.log'),indent=2)+'\n')
PY
printf 'PX-4 results: %s\n' "$g0_dir"
