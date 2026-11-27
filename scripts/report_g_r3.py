#!/usr/bin/env python3
"""Summarize G-R3 Metal copy timestamps and the five editor pixel-oracle cases."""
import argparse
import json
import math
import os
import subprocess
import statistics
from pathlib import Path


def summary(values):
    ordered = sorted(values)
    return {"mean": statistics.mean(ordered), "p95": ordered[math.ceil(.95 * len(ordered)) - 1]}


def area_results(directory):
    rows = {}
    for case, count in (("edit", 120), ("newline", 120), ("paste", 120), ("caret", 60)):
        lines = (directory / f"area-{case}.log").read_text().splitlines()
        if f"G_R2E_ORACLE {case}: {count} verified Partial frames" not in lines:
            raise ValueError(f"{case}: missing Partial and whole-frame equality proof")
        records = [line.split()[1:] for line in lines if line.startswith("RUSTUI_G_R3_AREA ")]
        if len(records) != count:
            raise ValueError(f"{case}: expected {count} pixel records, got {len(records)}")
        for index, record in enumerate(records):
            if case in ("newline", "paste") and index % 2:
                continue  # Inverse action restores the starting text for the next trial.
            edit = ("insert", "delete")[index % 2] if case == "edit" else case
            width, height, declared, changed, scissor = map(int, record)
            if width * height <= 0 or changed <= 0 or changed > scissor:
                raise ValueError(f"{edit}: invalid reference/scissor sizes {record}")
            samples = rows.setdefault(edit, {key: [] for key in ("declared", "changed", "scissor")})
            for key, pixels in zip(samples, (declared, changed, scissor)):
                samples[key].append(100 * pixels / (width * height))
    return {
        edit: {"frames": len(samples["changed"]), **{k: summary(v) for k, v in samples.items()}}
        for edit, samples in rows.items()
    }


def copy_results(log):
    bounds, records, partial = {}, [], []
    for line in log.read_text().splitlines():
        if line.startswith("RUSTUI_G0_LOG "):
            event = json.loads(line.removeprefix("RUSTUI_G0_LOG "))
            bounds[event["event"]] = event["epoch_ms"]
        elif line.startswith("RUSTUI_G_R3_COPY_GPU "):
            _, epoch, first, last, start, end, fragment_ticks = line.split()
            records.append((float(epoch), int(first), int(last), float(start), float(end), int(fragment_ticks)))
        elif line.startswith("RUSTUI_FRAME_LOG "):
            event = json.loads(line.removeprefix("RUSTUI_FRAME_LOG "))
            if event["event"] == "gpu" and event["mode"] == "partial":
                partial.append(event["epoch_ms"])
    rows = sorted(r for r in records if bounds["start"] <= r[0] <= bounds["end"])
    expected = sum(bounds["start"] <= ts <= bounds["end"] for ts in partial)
    if len(rows) != expected or len(rows) < 30:
        raise ValueError(f"expected {expected} Partial samples, found {len(rows)}")

    def ticks_per_second(samples):
        # The last stage in the original copy pass and GPUEndTime describe
        # the same command. Fit clock *rate* across many seconds. The fixed
        # interval between these markers cancels from the slope.
        k = max(1, len(samples) // 10)
        def anchor(col, block):
            return statistics.median(r[col] for r in block)
        return ((anchor(2, samples[-k:]) - anchor(2, samples[:k])) /
                (anchor(4, samples[-k:]) - anchor(4, samples[:k])))

    rate = ticks_per_second(rows)
    half = len(rows) // 2
    rates = [ticks_per_second(rows[:half]), ticks_per_second(rows[half:])]
    if rate <= 0 or any(abs(x / rate - 1) > .01 for x in rates):
        raise ValueError(f"counter clock rate inconsistent: {rate} versus {rates}")
    spans = [(last - first) * 1e6 / rate for _, first, last, _, _, _ in rows]
    commands = [(end - start) * 1e6 for _, _, _, start, end, _ in rows]
    fragments = [r[5] * 1e6 / rate for r in rows]
    if any(not 0 < frag <= span <= whole for frag, span, whole in zip(fragments, spans, commands)):
        raise ValueError("copy pass duration falls outside its enclosing command duration")
    return {"frames": len(rows), "ticks_per_second": rate, "half_rates": rates,
            "pass_span_us": summary(spans), "fragment_us": summary(fragments), "enclosing_command_us": summary(commands)}


def cpu_results(arm):
    """Reconcile a single-window typing capture with the published per-attempt denominator."""
    frame_events, bounds = [], {}
    for line in (arm / "capture.log").read_text().splitlines():
        if line.startswith("RUSTUI_FRAME_LOG "):
            frame_events.append(json.loads(line.removeprefix("RUSTUI_FRAME_LOG ")))
        elif line.startswith("RUSTUI_G0_LOG "):
            event = json.loads(line.removeprefix("RUSTUI_G0_LOG "))
            bounds[event["event"]] = event["epoch_ms"]
    # Renderer records and the core recorder flush separately. At equal
    # timestamps the renderer submission precedes the end of core present.
    frame_events.sort(key=lambda e: (e["epoch_ms"], e["event"] == "present"))
    draw, metal = 0, None
    samples = {key: [] for key in ("total", "plan", "encode", "copy", "present", "wait", "other")}
    for event in frame_events:
        if not bounds["start"] <= event["epoch_ms"] <= bounds["end"]:
            continue
        if event["event"] == "draw":
            draw = 1000 * event["cpu_ms"]
        elif event["event"] == "damage_attribution":
            if metal is not None:
                raise ValueError("expected one renderer per typing submission")
            metal = event
        elif event["event"] == "present":
            elapsed = event["cpu_ms"]
            if metal is None or not 0 <= event["epoch_ms"] - metal["epoch_ms"] <= elapsed + .1:
                raise ValueError("renderer record outside its core present")
            total = draw + 1000 * elapsed
            components = dict(plan=metal["plan_us"], copy=metal["copy_encode_us"],
                              wait=metal["drawable_wait_us"])
            components["encode"] = metal["encode_us"] - components["copy"]
            components["present"] = metal["present_us"] - components["wait"]
            components["other"] = total - sum(components.values())
            components["total"] = total
            for key, value in components.items():
                samples[key].append(value)
            draw, metal = 0, None
    expected = json.loads((arm / "g-r0.json").read_text())
    means = {key: summary(values) for key, values in samples.items()}
    if (len(samples["total"]) != expected["attempts"] or
            abs(means["total"]["mean"] / 1000 - expected["draw_present_ms_attempt"]) > 1e-7):
        raise ValueError("CPU sample does not reconcile to the capture summary")
    return {"attempts": expected["attempts"], **means}


def capture_area(directory, document):
    from test_g_r2e_editor_oracle import ROOT, manifest_text

    directory.mkdir(parents=True, exist_ok=False)
    fixture = directory / "fixture"
    fixture.mkdir()
    manifest = fixture / "Cargo.toml"
    manifest.write_text(manifest_text())
    (fixture / "Cargo.lock").write_bytes((ROOT / "Cargo.lock").read_bytes())
    target = Path.home() / ".cache/rustui-target"
    env = dict(os.environ, CARGO_TARGET_DIR=str(target), RUSTUI_G_R3_AREA="1",
               RUSTUI_G_R3_DOCUMENT=str(document.resolve(strict=True)),
               RUSTUI_DAMAGE="1", RUSTUI_DAMAGE_VERIFY="1", RUSTUI_DAMAGE_DECLARED="1",
               RUSTUI_G_R2_VIEW_SIGNATURES="1", RUSTUI_G_R2_EDITOR_ORACLE="1",
               RUSTUI_G5_PREPAINT_REUSE="1")
    subprocess.run(
        ["bash", "-c", '. scripts/pinned-toolchain.sh; pinned_cargo build --offline '
         '--manifest-path "$1" --features gpui_platform/font-kit', "oracle", str(manifest.resolve())],
        check=True, cwd=ROOT, env=env,
    )
    for case, cycles in (("edit", 60), ("newline", 60), ("paste", 60), ("caret", 30)):
        with (directory / f"area-{case}.log").open("x") as output:
            subprocess.run([str(target / "debug/editor-damage-oracle"), case, str(cycles)],
                           cwd=ROOT, env=env, stdout=output, stderr=subprocess.STDOUT, check=True)
    return area_results(directory)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="kind", required=True)
    commands.add_parser("cpu").add_argument("arm", type=Path)
    commands.add_parser("area").add_argument("directory", type=Path)
    commands.add_parser("copy").add_argument("log", type=Path)
    capture = commands.add_parser("capture-area")
    capture.add_argument("document", type=Path, help="G0 Lab's exported 10,000-line Rust file")
    capture.add_argument("output", type=Path, help="new directory for the verified capture")
    args = parser.parse_args()
    if args.kind == "cpu":
        report = cpu_results(args.arm)
    elif args.kind == "area":
        report = area_results(args.directory)
    elif args.kind == "copy":
        report = copy_results(args.log)
    else:
        report = capture_area(args.output, args.document)
    print(json.dumps(report, indent=2))
