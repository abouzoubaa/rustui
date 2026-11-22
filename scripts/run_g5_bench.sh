#!/usr/bin/env bash
set -euo pipefail
repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repository_root"
: "${RUSTUI_G5_REFERENCE:?Set RUSTUI_G5_REFERENCE=1 to run local G5 controls}"
[[ "$RUSTUI_G5_REFERENCE" == 1 ]]
unset MTL_HUD_ENABLED MTL_HUD_LOG_ENABLED RUSTUI_G5_FULLSCREEN RUSTUI_G5_WARMUP_S
export RUSTUI_DAMAGE=0 RUSTUI_FRAME_LOG=buffered RUSTUI_FRAME_PROFILE=1 RUSTUI_G0_LATENCY=1 RUSTUI_LATENCY_BREAKDOWN=1 RUSTUI_G5_REFRESH=1
output="${1:-target/headstart/g5-1/hold/paired}"
set_name="${RUSTUI_G5_SET:-all}"
case "$set_name" in all|hold|screen|transaction|immediate) ;; *) echo 'RUSTUI_G5_SET must be all, hold, screen, transaction or immediate' >&2;exit 2;; esac
background_limit="${RUSTUI_G5_BACKGROUND_LIMIT:-10}"
[[ "$background_limit" == 10 || "$background_limit" == 25 || "$background_limit" == none ]] || { echo 'Background limit must be 10, 25 or owner-authorized none' >&2; exit 2; }
mkdir -p "$output"
binary="target/headstart/g5-1/G5 Bare.app/Contents/MacOS/latency-gpui"
appkit="target/headstart/g5-1/latency-appkit-display"
[[ -x "$binary" && -x "$appkit" ]]
# Build once separately. Include the shared capture library in the freeze check.
frozen_paths=("$binary" "$appkit" target/headstart/g5-1/libG5Pixels.dylib)
before="$(shasum -a 256 "${frozen_paths[@]}")"
for set in hold screen transaction immediate; do
 if [[ "$set_name" != "$set" && !( "$set_name" == all && "$set" != immediate && "$set" != transaction ) ]]; then continue; fi
 for hz in ${RUSTUI_G5_RATES:-10 30}; do
  [[ "$hz" == 10 || "$hz" == 30 ]] || { echo "G5 rates must be 10 or 30" >&2; exit 2; }
  export RUSTUI_G5_INPUT_HZ="$hz"
  floor_var="RUSTUI_G5_FLOOR_$hz";floor_args=()
  if [[ -n "${!floor_var:-}" ]]; then floor_args=(--floor-frames "${!floor_var}"); fi
  for pair in 1 2 3 4 5; do
   if ((pair%2)); then arms=(0 1); else arms=(1 0); fi
   for arm in "${arms[@]}"; do
    printf 'G5 set=%s hz=%s pair=%s arm=%s start=%s\n' "$set" "$hz" "$pair" "$arm" "$(date -u +%FT%TZ)"
    log="$output/$set-$hz-$pair-$arm.log"
    [[ ! -e "$log" ]] || { echo "Refusing to overwrite $log" >&2; exit 2; }
    # Subtract Codex, its descendants and the measured apps in one top interval.
    # Every sample logs its top five; preserve rejected captures locally only.
    accepted=0
    for capture_try in 1 2 3; do
     top -l 1 -o cpu -n 5 -stats pid,ppid,cpu,command > "${log%.log}.top-summary-$capture_try.txt"
     start_top="${log%.log}.top-start-$capture_try.txt"
     if ! python3 scripts/g5_latency.py "$start_top" --collect-load --measured-executable "$binary" --measured-executable "$appkit" --background-limit "$background_limit"; then
      echo "Quiet gate rejected $set/$hz/$pair/$arm attempt $capture_try before capture"
      if ((capture_try<3)); then sleep 30; fi
      continue
     fi
    case "$set" in
     hold) GPUI_FAST_IMMEDIATE_PRESENT=1 GPUI_FAST_IMMEDIATE_HOLD_MS="$((arm*250))" GPUI_FAST_IMMEDIATE_TRANSACTION=0 RUSTUI_G5_PIXEL_CAPTURE=0 "$binary" 60 > "$log" 2>&1;;
     immediate) GPUI_FAST_IMMEDIATE_PRESENT="$arm" GPUI_FAST_IMMEDIATE_HOLD_MS=250 GPUI_FAST_IMMEDIATE_TRANSACTION=0 RUSTUI_G5_PIXEL_CAPTURE=0 "$binary" 60 > "$log" 2>&1;;
     transaction) GPUI_FAST_IMMEDIATE_PRESENT=1 GPUI_FAST_IMMEDIATE_HOLD_MS=250 GPUI_FAST_IMMEDIATE_TRANSACTION="$arm" RUSTUI_G5_PIXEL_CAPTURE=0 "$binary" 60 > "$log" 2>&1;;
     screen)
      if ((arm==0)); then
       GPUI_FAST_IMMEDIATE_PRESENT=1 GPUI_FAST_IMMEDIATE_HOLD_MS=250 GPUI_FAST_IMMEDIATE_TRANSACTION=0 RUSTUI_G5_PIXEL_CAPTURE=1 "$binary" 60 > "$log" 2>&1
      else
       "$appkit" 60 > "$log" 2>&1
      fi;;
    esac
     end_top="${log%.log}.top-end-$capture_try.txt"
     if ! python3 scripts/g5_latency.py "$end_top" --collect-load --measured-executable "$binary" --measured-executable "$appkit" --background-limit "$background_limit"; then
      mv "$log" "${log%.log}.discarded-$capture_try.log"
      echo "Discarded $set/$hz/$pair/$arm attempt $capture_try: background CPU above $background_limit%"
      if ((capture_try<3)); then sleep 30; fi
      continue
     fi
     python3 scripts/g5_latency.py "$log" "${floor_args[@]}" --slow-output "${log%.log}-slow.jsonl" --start-load "$start_top.json" --end-load "$end_top.json" > "${log%.log}.json"
     accepted=1
     break
    done
    if ((accepted==0)); then echo "No quiet capture for $set/$hz/$pair/$arm" >&2; exit 75; fi
   done
  done
 done
done
after="$(shasum -a 256 "${frozen_paths[@]}")"
[[ "$before" == "$after" ]]
printf 'G5 captures complete; executables and shared capture library unchanged\n'
