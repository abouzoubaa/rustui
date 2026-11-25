#!/usr/bin/env bash
set -euo pipefail
usage() {
  cat <<'EOF'
Usage: ./scripts/g5_owner_session.sh [all|visual|power|physical] [new-results-directory]

Run this from a terminal in the G5 closeout checkout.
The script builds before collecting. "power" and "all" ask you to authorize
sudo powermetrics. The other modes never use sudo.
EOF
}

case "${1:-all}" in
  -h|--help) usage; exit 0 ;;
  all|visual|power|physical) phase="${1:-all}" ;;
  *) usage >&2; exit 2 ;;
esac
if (( $# > 2 )); then usage >&2; exit 2; fi
if [[ "$(uname -s)" != Darwin || ! -t 0 ]]; then
  echo "Run this session from an interactive terminal on the test Mac." >&2
  exit 2
fi

. "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/run_g0_bench.sh"
default_results="target/headstart/performance/g5-owner-$(date -u +%Y%m%dT%H%M%SZ)"
mkdir -p target/headstart/performance
mkdir "${2:-$default_results}"
results="$(cd "${2:-$default_results}" && pwd)"
printf 'Results will be saved in %s\n' "$results"
{
  git rev-parse HEAD
  sw_vers
  pmset -g batt
} > "$results/session.txt"

export RUSTUI_DAMAGE=0 RUSTUI_FRAME_LOG=buffered RUSTUI_FRAME_PROFILE=1
unset GPUI_FAST_IMMEDIATE_PRESENT RUSTUI_DRAW_FIRST GPUI_FAST_IMMEDIATE_COALESCE
unset GPUI_FAST_IMMEDIATE_ANY_CHANGE GPUI_FAST_NATIVE_SCHEDULER GPUI_FAST_INPUT_SCOPED_DEMAND
unset RUSTUI_HL_FASTPATH GPUI_FAST_LATE_DRAWABLE RUSTUI_G5_TRIM_EDITOR
unset RUSTUI_G5_PREPAINT_REUSE RUSTUI_G5_L9_SCOPED RUSTUI_G5_C_RUST_FILE
unset RUSTUI_G5_KEEP_WARM RUSTUI_MAIN_LAG RUSTUI_G5_INCREMENTAL_EDITOR
unset RUSTUI_G5_L9_LOG RUSTUI_G5_PREPAINT_VERIFY RUSTUI_G5_L6_TRACE RUSTUI_G5_GPU_DENSITY
unset RUSTUI_G5_L8_CPU RUSTUI_G5_L8_IDLE RUSTUI_G5_L8_WAIT_STACKS RUSTUI_G5_L7_SINGLE_LINE
unset RUSTUI_G0_HOVER_PIXELS GPUI_FAST_DRAWABLE_RESERVE_TWO GPUI_FAST_TRACE_WAKES
export GPUI_FAST_IMMEDIATE_HOLD_MS=250
export GPUI_FAST_FORCE_LEGACY_LINK=0 GPUI_FAST_VIEW_DISPLAY_LINK=0 GPUI_FAST_FRAME_AHEAD=0
export GPUI_FAST_IMMEDIATE_TRANSACTION=0 RUSTUI_G5_REFRESH=0 RUSTUI_G5_PIXEL_CAPTURE=0
export RUSTUI_LATENCY_BREAKDOWN=1
unset DYLD_INSERT_LIBRARIES RUSTUI_L1_NATIVE RUSTUI_L1_CAPTURE_DIR RUSTUI_L1_KEY_SOCKET

if [[ "$phase" == all || "$phase" == visual ]]; then
  printf 'Building the Playground release once for all three arms.\n'
  pinned_cargo build -p rustui-playground --release --locked --offline
fi
if [[ "$phase" != visual ]]; then
  printf 'Building the Lab and L1 recorder before measurements begin.\n'
  ./scripts/build_g5_refs.sh
fi

cleanup() {
  if [[ -n "${g0_app_pid:-}" ]]; then kill "$g0_app_pid" 2>/dev/null || true; fi
  if [[ -n "${capture_pid:-}" ]]; then kill "$capture_pid" 2>/dev/null || true; fi
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

wait_for_start() {
  local logfile="$1" pid="$2" deadline=$((SECONDS + 25))
  until [[ -f "$logfile" ]] && grep -q '^RUSTUI_G0_LOG {"event":"start"' "$logfile"; do
    if ! kill -0 "$pid" 2>/dev/null || (( SECONDS >= deadline )); then
      printf 'Measurement did not start. Inspect %s\n' "$logfile" >&2
      return 1
    fi
    sleep 0.05
  done
}

verify_sync() {
  python3 - "$1" "$sync_off" <<'PYCODE'
import json
import sys
from pathlib import Path

path, sync_off = sys.argv[1:]
lines = Path(path).read_text().splitlines()
markers = [
    json.loads(line[len("RUSTUI_G0_LOG "):])
    for line in lines
    if line.startswith("RUSTUI_G0_LOG ")
]
start, end = (next(mark["mach"] for mark in markers if mark["event"] == name)
              for name in ("start", "end"))
settings = [
    json.loads(line[len("RUSTUI_PX4_LOG "):])
    for line in lines
    if line.startswith("RUSTUI_PX4_LOG ") and '"kind":"metal_layer"' in line
]
earlier = [entry for entry in settings if entry["mach"] <= start]
measured = [entry for entry in settings if start < entry["mach"] <= end]
if earlier:
    measured.insert(0, max(earlier, key=lambda entry: entry["mach"]))
expected = 1 - int(sync_off)
assert measured and all(entry["values"][1] == expected for entry in measured), (
    f"{path}: expected displaySyncEnabled={expected}, got {measured}"
)
PYCODE
}

for name in profile-off profile-on profile-on-sync-off; do
  profile=1
  sync_off=0
  case "$name" in
    profile-off) profile=0 ;;
    profile-on-sync-off) sync_off=1 ;;
  esac
  export RUSTUI_G5_PROFILE="$profile" GPUI_FAST_NO_DISPLAY_SYNC="$sync_off"
  arm="$results/$name"
  mkdir "$arm"
  printf 'RUSTUI_G5_PROFILE=%s\nGPUI_FAST_NO_DISPLAY_SYNC=%s\n' \
    "$profile" "$sync_off" > "$arm/mode.txt"

  if [[ "$phase" == all || "$phase" == visual ]]; then
    cat <<EOF

Playground tearing check, arm $name. Keep the window windowed.
  1. In the F0 Native Acceptance editor, type rapidly and backspace repeatedly.
     Watch letter edges, the caret and the line under the caret.
  2. Fling the F0 Collection panel down and back up. Watch row and column edges.
  3. Open NodeEditor. Drag a node quickly side to side, then diagonally.
     Watch the node border, its text and the lines attached to it.
  4. In the Card gallery use Light appearance and Dark appearance several times.
     Watch whether the canvas changes together in one frame.
Look for a horizontal seam, a split glyph, offset parts of one moving item,
or upper and lower regions briefly showing different themes. Note the action
and whether you can repeat it. Quit the Playground with Command-Q when done.
EOF
    RUSTUI_F0_ACCEPTANCE=1 "$CARGO_TARGET_DIR/release/rustui-playground" > "$arm/playground.log" 2>&1
    printf 'Enter the tearing result for %s (e.g. "none seen" or describe it): ' "$name"
    read -r visual_note
    printf '%s\n' "$visual_note" > "$arm/tearing.txt"
  fi

  if [[ "$phase" == all || "$phase" == power ]]; then
    printf '\nPower, arm %s. Authenticate for powermetrics when prompted.\n' "$name"
    printf 'Leave the Mac alone while the four automated 60-second scenes run.\n'
    sudo -v
    mkdir "$arm/power"
    native_helper="$repository_root/target/headstart/g5-1/libG5Native.dylib"
    for scene in typing hover drag idle; do
      printf 'Power: %s, arm %s, 60 one-second samples after warmup.\n' "$scene" "$name"
      /usr/bin/top -l 1 -n 5 -o cpu > "$arm/power/$scene-load.txt"
      frame_log="$arm/power/$scene-frames.log"
      RUSTUI_L1_NATIVE=1 RUSTUI_L1_PHYSICAL=0 RUSTUI_L1_LAB=1 RUSTUI_G0_TYPING_HZ=10 \
        DYLD_INSERT_LIBRARIES="$native_helper" g0_launch "$scene" "$frame_log"
      wait_for_start "$frame_log" "$g0_app_pid"
      sudo -n powermetrics --samplers gpu_power,cpu_power -i 1000 -n 60 \
        > "$arm/power/$scene-power.txt" 2>&1
      g0_wait "$frame_log"
      verify_sync "$frame_log"
    done
    python3 - "$arm/power" <<'PY'
import json
import re
import statistics
import sys
from pathlib import Path

root = Path(sys.argv[1])
rows = []
for scene in ("typing", "hover", "drag", "idle"):
    row = {"scene": scene}
    raw = (root / f"{scene}-power.txt").read_text()
    for component in ("CPU", "GPU"):
        samples = [
            float(value) * (1000 if unit == "W" else 1)
            for value, unit in re.findall(
                rf"^\s*{component} Power:\s*([0-9.]+)\s*(mW|W)\s*$", raw, re.M
            )
        ]
        if len(samples) != 60:
            raise SystemExit(f"{scene}: expected 60 {component} power samples, got {len(samples)}")
        row[f"{component.lower()}_mean_mw"] = statistics.mean(samples)
    rows.append(row)
(root / "summary.json").write_text(json.dumps(rows, indent=2) + "\n")
PY
  fi

  if [[ "$phase" == all || "$phase" == physical ]]; then
    printf '\nPhysical typing, arm %s. Press Return to launch the Lab.\n' "$name"
    printf 'Keep the Lab editor focused. At the start cue, alternate X and Delete (Backspace)\n'
    printf 'without modifiers and at a steady pace you can repeat for the other arms.\n'
    printf 'Keep typing for 60 seconds until the Lab closes automatically.\n'
    read -r
    python3 scripts/run_g5_budget.py --app lab --scene typing --launch direct \
      --physical --external-observer --seconds 60 --hl-fastpath 0 \
      --scheduler 0 --input-scoped 0 --switches 0000 --passive 0 --pixels 1 --output "$arm/physical" \
      > "$arm/physical-driver.log" 2>&1 &
    capture_pid=$!
    wait_for_start "$arm/physical/capture.log" "$capture_pid"
    printf '\aStart typing now.\n'
    wait "$capture_pid"
    capture_pid=
    verify_sync "$arm/physical/capture.log"
    printf 'Physical-key results: %s/physical/summary.json\n' "$arm"
  fi
done

printf '\nSession complete. Results: %s\n' "$results"
