#!/usr/bin/env python3
"""Run ordered G-R2c paired captures with thermal and in-window CPU qualification."""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import time

import run_g_r0

SCENES = ("typing", "scroll", "hover", "drag")
MAX_ATTEMPTS = 4


def thermal(path):
    raw = subprocess.check_output(["/usr/bin/pmset", "-g", "therm"], text=True)
    path.write_text(raw)
    limits = [int(limit) for limit in re.findall(
        r"(?im)\bCPU[ _]+Speed[ _]+Limit\s*[:=]\s*(\d+)", raw
    )]
    return {"speed_limits": limits, "raw": raw.strip(), "throttled": any(limit < 100 for limit in limits)}


def capture(out, scene, arm, seconds, frozen, probe=False):
    native = os.environ.get("RUSTUI_G_FS1_SEQUENCE") in ("A", "B")
    os.environ.update(
        RUSTUI_G_R1_FULLSCREEN=str(int(not native)),
        RUSTUI_G_R1_TRACE="0",
        RUSTUI_G_R2_LOAD=str(int(not probe)),
        RUSTUI_G_R2_ATTRIBUTION=str(int(not probe)),
        RUSTUI_G_R2_EARLY_REJECT=str(int(scene == "scroll")),
        RUSTUI_G_R2_ORDER_TRACE=str(int(probe)),
        RUSTUI_G_R2_DISJOINT_ORDER=str(int(scene == "hover" and not probe)),
    )
    result = run_g_r0.capture(out, scene, arm, seconds, frozen, observe_pixels=probe)
    log = list(run_g_r0.frames(out))
    start, _ = [row for prefix, row in log if prefix == "RUSTUI_G0_LOG "]
    drawable = tuple(round(start[axis] * start["scale"]) for axis in ("width", "height"))
    if drawable != ((3456, 2168) if native else (3456, 2234)):
        raise RuntimeError(f"unexpected drawable {drawable} in {out}")
    if probe:
        return result
    records = (out / "capture.log").read_text().splitlines()
    end = next(i for i, line in enumerate(records)
               if line.startswith('RUSTUI_G0_LOG {"event":"end"'))
    if ("RUSTUI_G_R2_LOG_LOST 0" not in records[end + 1:]
            or any(line.startswith("RUSTUI_FRAME_LOG_LOST ") for line in records)
            or any(line.startswith("RUSTUI_FRAME_LOG ") for line in records[:end])):
        raise RuntimeError(f"frame telemetry was streamed or dropped in {out}")
    prefix = "RUSTUI_G_R2_LOAD "
    load, = [json.loads(line[len(prefix):]) for line in records if line.startswith(prefix)]
    if (
        "RUSTUI_G_R2_LOST 0" not in records
        or not any(row["event"] == "damage_attribution" for _, row in log)
    ):
        raise RuntimeError(f"missing or dropped attribution samples in {out}")
    result["measured_background"] = load
    result["process_cpu_s_s"] = result["process_cpu_s"] / load["elapsed_s"]
    return result


def qualifies(off, on):
    return max(off, on) <= 100.0 and abs(on - off) <= 15.0


def attempt(output, scene, pair, retry, frozen, use_gate=True):
    root = output / f"{scene}-{pair}-try{retry}"
    root.mkdir()
    state = thermal(root / "thermal.txt") if use_gate else None
    result = {"scene": scene, "pair": pair, "attempt": retry, "thermal": state,
              "qualified": False}
    if state is None or not state["throttled"]:
        ordering = ("0", "1") if pair % 2 else ("1", "0")
        trials = {arm: capture(root / arm, scene, arm, 60, frozen) for arm in ordering}
        background = {arm: row["measured_background"]["one_core_percent"]
                      for arm, row in trials.items()}
        result.update(
            qualified=use_gate and qualifies(background["0"], background["1"]),
            background_percent_one_core=background, arms=trials,
        )
    (root / "pair.json").write_text(json.dumps(result, indent=2) + "\n")
    return result["qualified"], state is not None and state["throttled"]


def matrix(output, scenes, frozen, cutoff):
    outcomes = {f"{scene}-{pair}": "TIMEBOX" for scene in scenes for pair in range(1, 6)}
    status = output / "status.json"
    status.write_text(json.dumps(outcomes, indent=2) + "\n")
    for scene in scenes:
        for pair in range(1, 6):
            key = f"{scene}-{pair}"
            for retry in range(1, MAX_ATTEMPTS + 1):
                if cutoff and time.time() + 155 > cutoff:
                    return
                qualified, throttled = attempt(output, scene, pair, retry, frozen)
                state = "qualified" if qualified else ("thermal hold" if throttled else "load retry")
                print(f"{scene} pair {pair}, attempt {retry}: {state}", flush=True)
                if qualified:
                    outcomes[key] = "QUALIFIED"
                elif retry == MAX_ATTEMPTS:
                    outcomes[key] = "SKIPPED"
                status.write_text(json.dumps(outcomes, indent=2) + "\n")
                if outcomes[key] != "TIMEBOX":
                    break
                if throttled:
                    if cutoff and time.time() + 60 > cutoff:
                        return
                    time.sleep(60)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("matrix", "order", "diagnostic"))
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--scene", action="append", choices=SCENES)
    parser.add_argument("--stop-before", type=float)
    args = parser.parse_args()
    if args.mode != "order":
        pids = subprocess.check_output(["/usr/bin/pgrep", "-x", "WindowServer"], text=True).split()
        if len(pids) != 1:
            parser.error("expected exactly one WindowServer process")
        os.environ["RUSTUI_G_R2_WINDOW_SERVER_PID"] = pids[0]
    args.output.mkdir(parents=True, exist_ok=False)
    frozen = run_g_r0.fingerprint()
    (args.output / "frozen-build.json").write_text(json.dumps(frozen, indent=2) + "\n")
    if args.mode == "order":
        capture(args.output / "hover", "hover", "1", 12, frozen, probe=True)
    elif args.mode == "diagnostic":
        attempt(args.output, "typing", 1, 1, frozen, use_gate=False)
    else:
        matrix(args.output, args.scene or SCENES, frozen, args.stop_before)


if __name__ == "__main__":
    os.chdir(run_g_r0.REPO)
    main()
