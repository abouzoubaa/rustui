#!/usr/bin/env python3
"""Time the first matched frame of each key, beyond coarse draw/commit stages."""
import argparse
from bisect import bisect_left
from collections import Counter, defaultdict
import csv
import json
from pathlib import Path

import g5_budget
from g5_latency import distribution


def classify(gpu_end, display, refresh_s):
    if gpu_end is None or display is None:
        return "unobserved"
    margin = display - gpu_end
    if margin < 0:
        return "negative_clock_margin"
    return "hit" if margin < refresh_s else "miss"


def exclusive(intervals, priority):
    """Assign overlapping CPU spans to the earliest phase in priority order."""
    edges = sorted({edge for start, end, _ in intervals for edge in (start, end)})
    values = defaultdict(float)
    order = {name: i for i, name in enumerate(priority)}
    for low, high in zip(edges, edges[1:]):
        owners = [kind for start, end, kind in intervals if start <= low and high <= end]
        if owners:
            values[min(owners, key=order.__getitem__)] += (high - low) * 1000
    return values


DRAW_MAPPED = {
    "layout": "layout",
    "prepaint": "prepaint",
    "paint": "paint_scene_build",
    "scene_sort": "scene_sort",
}
RENDER_MAPPED = {
    "scene_instances": "scene_to_instances",
    "encode": "encode",
    "acquire": "drawable_acquire",
}
PHASE_PRIORITY = (*DRAW_MAPPED.values(), *RENDER_MAPPED.values(),
                  "draw_setup_finish", "present_cpu_other")


def analyze(path, refresh_s, detailed):
    data = g5_budget.load(path)
    summary = g5_budget.analyse(path)
    events = data["RUSTUI_PX4_LOG "]
    clock = next(event for event in events if event["event"] == "clock")
    scale = clock["numer"] / clock["denom"] / 1e9
    start, end = (next(event["mach"] for event in data["RUSTUI_G0_LOG "]
                       if event["event"] == phase) for phase in ("start", "end"))
    configs = sorted((event for event in events if event.get("kind") == "metal_layer"
                      and event["mach"] <= end), key=lambda event: event["mach"])
    assert any(event["mach"] <= start for event in configs), "missing window layer setting"
    measured = [next(event for event in reversed(configs) if event["mach"] <= start)]
    measured += [event for event in configs if event["mach"] > start]
    assert all(event["values"][1] == 0 for event in measured), "display sync was enabled"

    by_attempt = defaultdict(list)
    by_draw = defaultdict(list)
    handlers = []
    gpu_commits = {}
    attempts = {}
    for event in events:
        kind = event["event"]
        if kind == "phase_dropped":
            assert event["count"] == 0, "phase buffer overflow"
        if kind == "phase_span":
            if event["kind"] == "native_handler_end":
                handlers.append(event["start"] * scale)
            else:
                index = by_attempt if event["attempt"] else by_draw
                index[event["window"], event["frame"]].append(event)
        elif kind == "attempt":
            attempts[event["window"], event["attempt"]] = event
        elif kind == "gpu_completed":
            gpu_commits[event["gpu_end_host"]] = event
    handlers.sort()

    rows = {r["version"]: r for r in summary["rows"]}
    result = []
    for key in range(1, summary["expected"] + 1):
        source = rows.get(key, {})
        gpu = source.get("gpu")
        display = source.get("display")
        row = {
            "capture": path.parent.name,
            "key": key,
            "class": classify(gpu, display, refresh_s),
            "gpu_to_sck_ms": (display - gpu) * 1000 if display is not None and gpu is not None else None,
            "P": source.get("P"),
        }
        for field in ("handler", "draw_start", "draw_end", "acquired", "committed", "gpu", "display"):
            row[field] = source.get(field)
        if detailed and source:
            frame = gpu_commits[gpu]
            window = frame["window"]
            attempt = attempts[window, frame["attempt"]]["mach"] * scale
            handler, draw, draw_end = (source[name] for name in ("handler", "draw_start", "draw_end"))
            handler_idx = bisect_left(handlers, handler)
            if handler_idx == len(handlers):
                result.append(row)
                continue
            handler_end = handlers[handler_idx]
            # A nested Cocoa callback or render at an unexpected boundary is incomplete.
            if handler_end > draw or not handler <= draw <= draw_end <= attempt <= source["committed"]:
                result.append(row)
                continue
            draw_records = by_draw[window, frame["draw"]]
            render_records = by_attempt[window, frame["attempt"]]
            if not set(DRAW_MAPPED) <= {span["kind"] for span in draw_records}:
                result.append(row)
                continue
            if not {"scene_instances", "encode"} <= {span["kind"] for span in render_records}:
                result.append(row)
                continue
            commit = source["committed"]
            gpu_start = frame["gpu_start_host"]
            if not 0 < gpu_start <= gpu or gpu_start + 0.0001 < commit:
                result.append(row)
                continue
            intervals = [(draw, draw_end, "draw_setup_finish"),
                         (attempt, commit, "present_cpu_other")]
            for spans, names, lo, hi in ((draw_records, DRAW_MAPPED, draw, draw_end),
                                         (render_records, RENDER_MAPPED, attempt, commit)):
                for span in spans:
                    if span["kind"] not in names:
                        continue
                    left, right = max(lo, span["start"] * scale), min(hi, span["end"] * scale)
                    if left < right:
                        intervals.append((left, right, names[span["kind"]]))
            phases = exclusive(intervals, PHASE_PRIORITY)
            row.update({name: phases[name] for name in PHASE_PRIORITY})
            row.update(
                handler_end=handler_end, gpu_start=gpu_start,
                handler_state=(handler_end-handler)*1000,
                dispatch_to_draw=(draw-handler_end)*1000,
                draw_to_attempt=(attempt-draw_end)*1000,
                commit_to_gpu_start=(gpu_start-commit)*1000,
                gpu_execution=(gpu-gpu_start)*1000,
            )
            error = sum(row[name] for name in ALL_PHASES) - row["P"]
            assert abs(error) < .0001, ("phase accounting", path, key, error)
        result.append(row)
    return result


ALL_PHASES = ("handler_state", "dispatch_to_draw", *PHASE_PRIORITY,
              "draw_to_attempt", "commit_to_gpu_start", "gpu_execution")


def aggregate(rows):
    counts = Counter(row["class"] for row in rows)
    eligible = counts["hit"] + counts["miss"]
    complete = [r for r in rows if "gpu_execution" in r]
    return {
        "counts": counts,
        "miss_percent": counts["miss"] * 100 / eligible if eligible else None,
        "hit_P": distribution([r["P"] for r in rows if r["class"] == "hit"]),
        "miss_P": distribution([r["P"] for r in rows if r["class"] == "miss"]),
        "detailed_keys": len(complete),
        "phases": {name: distribution([r[name] for r in complete]) for name in ALL_PHASES},
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--refresh-hz", type=float, required=True)
    parser.add_argument("--detailed", action="store_true")
    parser.add_argument("--csv", type=Path, required=True)
    parser.add_argument("logs", nargs="+", type=Path)
    args = parser.parse_args()
    if args.refresh_hz <= 0:
        parser.error("refresh rate must be positive")
    rows = []
    for log in args.logs:
        rows.extend(analyze(log, 1 / args.refresh_hz, args.detailed))
    with args.csv.open("w", newline="") as output:
        columns = (
            "capture", "key", "class", "gpu_to_sck_ms", "P", "handler", "handler_end",
            "draw_start", "draw_end", "acquired", "committed", "gpu_start", "gpu",
            "display", *ALL_PHASES,
        )
        writer = csv.DictWriter(output, fieldnames=columns)
        writer.writeheader()
        writer.writerows(rows)
    print(json.dumps(aggregate(rows), indent=2))


if __name__ == "__main__":
    main()
