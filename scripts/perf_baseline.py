#!/usr/bin/env python3
"""Parse the owner-update workload and write its reproducible baseline."""
from __future__ import annotations

import argparse
import csv
import json
import math
from pathlib import Path
import statistics

ROWS = (10, 100, 1000)
MODES = ("update", "forced_draw")
HEADER = "rows,mode,sample,updates,update_renders,draw_renders,elapsed_ns,ns_per_update"
REFERENCE_MS = 12.2
TARGET_MS = 8.0


def parse_csv(text: str, expected_samples: int = 10) -> dict[tuple[int, str], list[float]]:
    lines = text.splitlines()
    try:
        start = next(index for index, line in enumerate(lines) if line.endswith(HEADER))
    except StopIteration as error:
        raise ValueError("workload CSV header is missing") from error
    cases = {(rows, mode): {} for rows in ROWS for mode in MODES}
    for row in csv.reader(lines[start + 1 :]):
        if not row or not row[0].isdigit():
            continue
        if len(row) != 8:
            raise ValueError(f"invalid workload CSV row: {row}")
        try:
            rows, mode, sample, updates = int(row[0]), row[1], int(row[2]), int(row[3])
            value = float(row[7]) / 1_000_000
        except ValueError as error:
            raise ValueError(f"invalid workload CSV row: {row}") from error
        key = (rows, mode)
        if key not in cases or updates != 10 or sample not in range(1, expected_samples + 1):
            raise ValueError(f"unexpected workload CSV row: {row}")
        if sample in cases[key] or not math.isfinite(value) or value < 0:
            raise ValueError(f"duplicate or invalid workload sample: {row}")
        cases[key][sample] = value
    for key, values in cases.items():
        if len(values) != expected_samples:
            raise ValueError(f"missing workload samples for {key}: {len(values)}/{expected_samples}")
    return {key: list(values.values()) for key, values in cases.items()}


def build_baseline(runs: list[dict[tuple[int, str], list[float]]], metadata: dict) -> dict:
    if len(runs) != 3 or any(set(run) != {(r, m) for r in ROWS for m in MODES} for run in runs):
        raise ValueError("expected three complete workload runs")
    cases = {}
    for rows in ROWS:
        cases[str(rows)] = {
            mode: statistics.median(value for run in runs for value in run[(rows, mode)])
            for mode in MODES
        }
    draw_medians = [statistics.median(run[(1000, "forced_draw")]) for run in runs]
    center = statistics.median(draw_medians)
    if center <= 0:
        raise ValueError("1,000-row forced-draw median must be positive")
    spread = (max(draw_medians) - min(draw_medians)) / center * 100
    return {
        **metadata,
        "median_ms_per_update": cases,
        "forced_draw_1000_run_medians_ms": draw_medians,
        "forced_draw_1000_spread_percent": spread,
        "spread_definition": "(max - min) / median of the three run medians * 100",
        "reproducible": spread <= 15.0,
        "reference_ms": REFERENCE_MS,
        "target_ms": TARGET_MS,
    }


def format_table(baseline: dict) -> str:
    lines = ["case | median ms | vs 12.2 ref | vs 8.0 target"]
    for rows in ROWS:
        for mode in MODES:
            value = baseline["median_ms_per_update"][str(rows)][mode]
            label = "update only" if mode == "update" else "update + forced draw"
            change = (value / REFERENCE_MS - 1) * 100
            lines.append(f"{rows:,} rows, {label} | {value:.4f} | {change:+.1f}% | {'PASS' if value <= TARGET_MS else 'MISS'}")
    lines.append(f"1,000-row forced-draw spread: {baseline['forced_draw_1000_spread_percent']:.2f}% ({'reproducible' if baseline['reproducible'] else 'not reproducible'})")
    return "\n".join(lines)


def write_baseline(path: Path, baseline: dict) -> None:
    path.write_text(json.dumps(baseline, indent=2, allow_nan=False) + "\n")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--commit", required=True)
    parser.add_argument("--date", required=True)
    parser.add_argument("--rustc", required=True)
    parser.add_argument("--macos", required=True)
    parser.add_argument("--cpu", required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("runs", nargs=3, type=Path)
    args = parser.parse_args()
    runs = [parse_csv(path.read_text()) for path in args.runs]
    metadata = {"commit": args.commit, "date": args.date, "rustc_version": args.rustc, "macos_version": args.macos, "cpu_model": args.cpu}
    baseline = build_baseline(runs, metadata)
    write_baseline(args.output, baseline)
    print(format_table(baseline))


if __name__ == "__main__":
    main()
