#!/usr/bin/env python3
"""Run the G-R0 native pairs or the separate Metal pixel oracle on a frozen release build."""
import argparse
from collections import Counter
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

import g5_latency

REPO = Path(__file__).resolve().parents[1]
REFS = REPO / "target/headstart/g5-1"
BINARY = REFS / "RustUI Lab.app/Contents/MacOS/rustui-lab"
ASSETS = [BINARY, *(REFS / file for file in (
    "libG5Native.dylib", "libG5Pixels.dylib", "libG5Refresh.dylib", "l1-observer",
))]
SCENES = ("idle", "typing", "hover", "drag", "scroll")


def fingerprint():
    return {str(file): hashlib.sha256(file.read_bytes()).hexdigest() for file in ASSETS}


def load(output):
    data = g5_latency.collect_load(str(output), [str(BINARY)], None)
    if not data["consistent"]:
        raise RuntimeError(f"Inconsistent load sample in {output}")
    return data


def quiet(output, scene, pair):
    stem = f"{scene}-{pair}-prepair"
    before = load(output / f"{stem}-0.txt")
    if before["background_cpu_percent"] <= 5:
        return before["background_cpu_percent"]
    time.sleep(30)
    after = load(output / f"{stem}-1.txt")
    return after["background_cpu_percent"]


def frames(directory):
    with (directory / "capture.log").open() as file:
        for line in file:
            for prefix in ("RUSTUI_G0_LOG ", "RUSTUI_FRAME_LOG ", "RUSTUI_PX4_LOG ", "G5_L8_CPU "):
                if line.startswith(prefix):
                    yield prefix, json.loads(line[len(prefix):])
                    break


def capture(out, scene, damage, seconds, frozen, prepair=None, retry=False, observe_pixels=True):
    print(time.strftime("%H:%M:%S UTC", time.gmtime()), out.name, flush=True)
    if out.exists() and retry:
        archive = out.parent / f"{out.name}-discarded-{int(time.time())}"
        shutil.move(out, archive)
    env = dict(os.environ, RUSTUI_G5_PROFILE="1", GPUI_FAST_NO_DISPLAY_SYNC="0",
               RUSTUI_DAMAGE=damage, RUSTUI_DAMAGE_BACKOFF="0", RUSTUI_FRAME_LOG="buffered",
               RUSTUI_FRAME_PROFILE="1", RUSTUI_LATENCY_BREAKDOWN="1")
    if scene == "idle":
        out.mkdir()
        env.update(RUSTUI_L1_NATIVE="1", RUSTUI_G5_PIXEL_CAPTURE="0", RUSTUI_G5_L8_CPU="1",
                   RUSTUI_G5_L8_IDLE="focused", RUSTUI_G5_L8_WAIT_STACKS="0",
                   DYLD_INSERT_LIBRARIES=str(REFS / "libG5Native.dylib"))
        before = load(out / "load-start.txt")
        with (out / "capture.log").open("w") as logfile:
            subprocess.run([str(BINARY), "--g0-scene", scene, "--duration", str(seconds)],
                           env=env, check=True, stdout=logfile, stderr=subprocess.STDOUT,
                           timeout=seconds + 20)
        after = load(out / "load-end.txt")
        native = {"stats": {}, "machine_load": dict(
            start_background_cpu_percent=before["background_cpu_percent"],
            end_background_cpu_percent=after["background_cpu_percent"])}
    else:
        cmd = [sys.executable, "scripts/run_g5_budget.py", "--app", "lab", "--launch", "direct",
               "--scene", scene, "--seconds", str(seconds), "--damage", damage,
               "--target-hover", "--passive", "0", "--pixels", str(int(observe_pixels)), "--switches", "0000",
               "--scheduler", "0", "--input-scoped", "0", "--hl-fastpath", "0", "--output", str(out)]
        if observe_pixels:
            cmd.append("--external-observer")
        with out.with_suffix(".driver").open("w") as logfile:
            subprocess.run(cmd, env=env, check=True, stdout=logfile, stderr=subprocess.STDOUT)
        native = json.loads((out / "summary.json").read_text())
    if frozen != fingerprint():
        raise RuntimeError("The release build changed during the matrix")
    telemetry = list(frames(out))
    caret_ticks = None
    if scene == "idle":
        boundaries = [r for p, r in telemetry if p == "G5_L8_CPU " and r["event"] == "boundary"]
        if [r["phase"] for r in boundaries] != [0, 1]:
            raise RuntimeError(f"Missing idle boundaries in {out}")
        start, end = boundaries
        caret_ticks = end["caret_ticks"] - start["caret_ticks"]
        if not all(r["active"] and r["key"] for r in boundaries):
            raise RuntimeError(f"Idle must stay focused in {out}")
        cpu = sum(end[k] - start[k] for k in ("user_us", "sys_us")) / 1e6
    else:
        window = [r for p, r in telemetry if p == "RUSTUI_G0_LOG "]
        if [r["event"] for r in window] != ["start", "end"] or not window[-1]["valid"]:
            raise RuntimeError(f"Invalid scene completion in {out}")
        start, end = window
        cpu = end["cpu_user_s"] + end["cpu_sys_s"]
    observations = [r for p, r in telemetry if p == "RUSTUI_PX4_LOG " and r.get("kind") == "metal_layer"]
    before = [r for r in observations if r["mach"] <= start["mach"]]
    measured = [r for r in observations if start["mach"] < r["mach"] <= end["mach"]]
    if not before or any(r["values"][1] != 1 for r in [before[-1], *measured]):
        raise RuntimeError(f"No evidence that display sync was on in {out}")
    all_records = [r for p, r in telemetry if p == "RUSTUI_FRAME_LOG "]
    records = [r for r in all_records if start["epoch_ms"] <= r["epoch_ms"] <= end["epoch_ms"]]
    caret_transitions = None
    if scene == "idle":
        encodings = sorted((r for r in records if r["event"] == "encode"), key=lambda r: r["sequence"])
        sizes = [r["instances_total"] for r in encodings]
        caret_transitions = sum(a != b for a, b in zip(sizes, sizes[1:]))
        if caret_transitions < 2:
            raise RuntimeError(f"No repeated caret primitive changes in {out}")
    modes = Counter(r["mode"] for r in records if r["event"] == "gpu")
    gpu_times = [r["gpu_ms"] for r in records if r["event"] == "gpu" and r["gpu_ms"] is not None]
    reasons = Counter(r["reason"] for r in records if r["event"] == "frame_reason")
    checks = Counter(r["mode"] for r in all_records if r["event"] == "damage_verify")
    mismatches = [r for r in all_records if r["event"] == "damage_mismatch"]
    draw_ms = 0.0
    phase_attempts = []
    for record in records:
        if record["event"] == "draw":
            draw_ms = record["cpu_ms"]
        elif record["event"] == "present":
            phase_attempts.append(draw_ms + record["cpu_ms"])
            draw_ms = 0.0
    attempts = len(phase_attempts)
    elapsed = (end["epoch_ms"] - start["epoch_ms"]) / 1000
    if not attempts or not gpu_times or (damage == "0" and set(modes) != {"direct"}):
        raise RuntimeError(f"No usable renderer timings or unexpected damage mode in {out}")
    sck = native["stats"].get("sck_total")
    if observe_pixels and scene != "idle" and damage != "verify" and (not sck or not sck["samples"]):
        raise RuntimeError(f"No native handler-to-SCK samples in {out}")
    if mismatches and damage != "verify":
        raise RuntimeError(f"Unexpected damage mismatch in timing run {out}")
    result = dict(scene=scene, damage=damage, elapsed=elapsed, modes=modes, reasons=reasons,
                  verified_outputs=checks, mismatches=mismatches, attempts=attempts,
                  caret_ticks=caret_ticks, caret_transitions=caret_transitions,
                  gpu_ms_frame=sum(gpu_times) / len(gpu_times), gpu_ms_s=sum(gpu_times) / elapsed,
                  process_cpu_s=cpu, process_cpu_s_s=cpu / elapsed, process_cpu_ms_attempt=1000 * cpu / attempts,
                  draw_present_ms_attempt=sum(phase_attempts) / attempts,
                  sck=sck, metal=native["stats"].get("hardware_presented_total"),
                  metal_order=native.get("out_of_order"), load=native["machine_load"], prepair=prepair)
    (out / "g-r0.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result), flush=True)
    if damage == "verify":
        if mismatches:
            raise RuntimeError(f"Damage verification found {len(mismatches)} pixel mismatches in {out}")
        if not checks:
            raise RuntimeError(f"Damage verification checked no outputs in {out}")
    return result


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("work", choices=("smoke", "verify", "matrix"))
    p.add_argument("--output", type=Path, default=REPO / "target/headstart/g-r0/runs")
    p.add_argument("--scene", action="append", choices=SCENES)
    p.add_argument("--pair", action="append", type=int, choices=range(1, 6))
    p.add_argument("--stop-before", type=float)
    p.add_argument("--retry", action="store_true", help="Archive any existing selected capture before retrying.")
    args = p.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    frozen = fingerprint()
    proof = args.output / "frozen-build.json"
    if proof.exists() and frozen != json.loads(proof.read_text()):
        p.error("the output directory belongs to a different release build")
    proof.write_text(json.dumps(frozen, indent=2) + "\n")
    for scene in args.scene or SCENES:
        if args.work != "matrix":
            for arm in (("0", "1") if args.work == "smoke" else ("verify",)):
                capture(args.output / f"{args.work}-{scene}-{arm}", scene, arm,
                        8 if args.work == "smoke" else 20, frozen, retry=args.retry)
            continue
        for pair in args.pair or range(1, 6):
            if args.stop_before and time.time() + 185 > args.stop_before:
                raise SystemExit("Timebox cannot fit another complete pair")
            prepair = quiet(args.output, scene, pair)
            for arm in (("0", "1") if pair % 2 else ("1", "0")):
                capture(args.output / f"{scene}-{pair}-{arm}", scene, arm, 60, frozen, prepair, args.retry)


if __name__ == "__main__":
    os.chdir(REPO)
    main()
