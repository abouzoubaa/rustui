"""Capture or summarize native Metal fullscreen diagnostics for G-FS1."""
import argparse
from collections import Counter
import json
import os
from pathlib import Path
import subprocess
import sys


FIELDS = """index epoch_ms surface layer_w layer_h scale framebuffer_only
drawable_w drawable_h drawable_format before_w before_h before_format before_valid
after_w after_h after_format after_valid allocated planned decision reason
budget_used budget_requested""".split()
SUBMITTED = {"direct", "partial", "copy", "retained", "shift"}


def summarize(path):
    frames, markers, entered, lost = [], {}, None, []
    completed = 0
    verified = Counter()
    mismatches = 0
    for line in path.read_text().splitlines():
        if line.startswith("G_FS1_FRAME "):
            frames.append(dict(zip(FIELDS, line.split()[1:], strict=True)))
        elif line.startswith("RUSTUI_FRAME_LOG "):
            record = json.loads(line.split(" ", 1)[1])
            if record.get("event") == "damage_verify":
                verified[record["mode"]] += 1
            mismatches += record.get("event") == "damage_mismatch"
            completed += record.get("event") == "gpu" and record.get("mode") in SUBMITTED
        elif line.startswith("G_FS1_ENTER "):
            entered = float(line.split()[1])
        elif line.startswith("RUSTUI_G0_LOG "):
            record = json.loads(line.split(" ", 1)[1])
            markers[record["event"]] = record
        elif line.startswith("RUSTUI_G_R2_LOG_LOST "):
            lost.append(int(line.split()[1]))
    if lost != [0] or not markers["end"]["valid"]:
        raise ValueError(f"incomplete capture: {path}")
    frames.sort(key=lambda r: int(r["index"]))
    if [int(r["index"]) for r in frames] != list(range(1, len(frames) + 1)):
        raise ValueError("incomplete renderer trace")
    if len({r["surface"] for r in frames}) != 1:
        raise ValueError("expected a single Lab presentation surface")
    draws = [r for r in frames if r["decision"] in SUBMITTED]
    if not draws or len(draws) != completed:
        raise ValueError(f"{len(draws)} traced submissions, {completed} GPU completions")
    if entered is not None and markers["start"]["epoch_ms"] - entered < 5000:
        raise ValueError("native fullscreen did not settle for five seconds")

    def geometry(r):
        return tuple(int(r[k]) for k in ("drawable_w", "drawable_h", "drawable_format"))

    transitions = []
    for i, row in enumerate(draws):
        if i == 0 or geometry(row) != geometry(draws[i - 1]):
            transitions.append((i, int(row["index"]), geometry(row)))
    final_size = geometry(draws[-1])[:2]
    final_start = 1 + max((i for i, r in enumerate(draws) if geometry(r)[:2] != final_size), default=-1)
    after_entry = [r for r in draws if entered and float(r["epoch_ms"]) >= entered]
    drag = [r for r in draws if markers["start"]["epoch_ms"] <= float(r["epoch_ms"]) < markers["end"]["epoch_ms"]]

    def counts(rows):
        return dict(Counter(r["decision"] + "/" + r["reason"] for r in rows))

    def partial(rows):
        return [sum(r["decision"] == "partial" for r in rows), len(rows)]

    # A verification callback may finish after the measured interval. Subtract
    # every unmeasured Partial from completed comparisons for a safe lower bound.
    measured_verified = max(0, verified["partial"] - (partial(draws)[0] - partial(drag)[0]))
    return dict(verified=verified, measured_verified_partial_min=measured_verified, mismatches=mismatches,
                draws=partial(draws), after_last_size=partial(draws[final_start:]),
                measured=partial(drag), first_partial=next((r["index"] for r in draws if r["decision"] == "partial"), None),
                reasons=counts(draws), non_submissions=counts([r for r in frames if r["decision"] not in SUBMITTED]),
                geometry_changes=transitions, entry_ms=entered,
                geometry_after_entry=sorted(set(map(geometry, after_entry))),
                scale=sorted({r["scale"] for r in draws}), framebuffer_only=sorted({r["framebuffer_only"] for r in draws}),
                allocations=[(r["index"], r["after_w"], r["after_h"], r["after_valid"]) for r in draws if r["allocated"] == "true"],
                first_valid=next((r["index"] for r in draws if r["after_valid"] == "true"), None),
                final_valid=draws[-1]["after_valid"], failures=[r for r in frames if int(r["budget_requested"])],
                start=markers["start"], end=markers["end"])


def capture(directory, sequence, verify, seconds=None):
    historical = sequence == "historical"
    if historical and not verify:
        raise ValueError("historical reproduces the G-R3 verify run")
    env = dict(os.environ, RUSTUI_G_FS1_TRACE="1", RUSTUI_G5_PROFILE="1",
               GPUI_FAST_NO_DISPLAY_SYNC="0", RUSTUI_DAMAGE_BACKOFF="0",
               RUSTUI_DAMAGE_DECLARED="1", RUSTUI_G_R2_VIEW_SIGNATURES="1",
               RUSTUI_G_R2_DISJOINT_ORDER="0", RUSTUI_G_R2_EARLY_REJECT="0",
               RUSTUI_G_R1_TRACE="0", RUSTUI_G_R2_ORDER_TRACE="0",
               RUSTUI_DAMAGE_VERIFY=str(int(verify)), RUSTUI_G_R3_AREA=str(int(historical)),
               RUSTUI_G_R1_FULLSCREEN=str(int(historical)))
    env.pop("RUSTUI_G_FS1_SEQUENCE", None)
    if not historical:
        env["RUSTUI_G_FS1_SEQUENCE"] = sequence
    subprocess.run([sys.executable, "scripts/run_g5_budget.py", "--app", "lab", "--launch", "direct",
                    "--scene", "typing" if historical else "drag", "--seconds", str(seconds or (35 if historical else 10)),
                    "--damage", "verify" if verify else "1", "--target-hover", "--passive", "0",
                    "--pixels", "0", "--switches", "0000", "--scheduler", "0", "--input-scoped", "0",
                    "--hl-fastpath", "0", "--timeout", str((seconds or 35) + 40), "--output", str(directory)], env=env, check=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--capture", choices=["A", "B", "C", "historical"])
    parser.add_argument("--verify", action="store_true")
    parser.add_argument("--seconds", type=int)
    parser.add_argument("--oracle-frames", type=int, help="Require this many verified native fullscreen drag frames")
    args = parser.parse_args()
    if args.capture:
        capture(args.directory, args.capture, args.verify, args.seconds)
    result = summarize(args.directory / "capture.log")
    print(json.dumps(result, indent=2))
    if args.oracle_frames is not None:
        if (args.oracle_frames <= 0 or result["start"]["scene"] != "drag" or result["entry_ms"] is None
                or result["geometry_after_entry"] != [(3456, 2168, 80)]
                or result["measured_verified_partial_min"] < args.oracle_frames
                or result["mismatches"] or result["failures"] or "direct/verify_off_path" in result["reasons"]):
            raise SystemExit("fullscreen pixel oracle failed")

