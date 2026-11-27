#!/usr/bin/env python3
"""Attribute hardware counter intervals to complete shift frames from a G3b capture."""
import argparse
from collections import Counter, defaultdict
import json
import math
from pathlib import Path
from statistics import mean


PHASES = ("shift_copy", "exposed", "other", "mixed", "present_copy")


def analyze(path):
    bounds = []
    expected = {}
    stages = defaultdict(list)
    for line in path.open():
        if line.startswith("RUSTUI_G0_LOG "):
            event = json.loads(line.split(" ", 1)[1])
            if event["event"] in ("start", "end"):
                bounds.append(event["epoch_ms"])
        elif line.startswith("RUSTUI_SHIFT_GPU_EXPECT "):
            _, start, end, exposed, other, mixed = line.split()
            expected[start, end] = Counter(dict(zip(PHASES, (1, int(exposed), int(other), int(mixed), 1))))
        elif line.startswith("RUSTUI_SHIFT_GPU "):
            _, epoch, phase, first, last, start, end, *ticks = line.split()
            if len(ticks) != 4:
                raise ValueError("Missing encoder stage samples")
            stages[start, end].append((phase, int(first), int(last), float(epoch), tuple(map(int, ticks))))
    begin, finish = bounds
    frames = {}
    rejected = 0
    for times, values in stages.items():
        first_log = min(v[3] for v in values)
        if not begin <= first_log <= finish:
            continue
        if Counter(phase for phase, *_ in values) != expected.get(times):
            rejected += 1
            continue
        frames[times] = values
    if len(frames) < 2:
        raise ValueError("Insufficient complete timestamped shift frames")
    # Fit all first-encoder timestamps against the same command buffers' GPU start times.
    points = [(float(start), next(item[1] for item in v if item[0] == "shift_copy"))
              for (start, _), v in frames.items()]
    x0, y0 = points[0]
    xs, ys = zip(*((x - x0, y - y0) for x, y in points))
    xm, ym = mean(xs), mean(ys)
    rate = sum((x - xm) * (y - ym) for x, y in zip(xs, ys)) / sum((x - xm) ** 2 for x in xs)
    if not math.isfinite(rate) or rate <= 0:
        raise ValueError("Invalid GPU timestamp calibration")
    rows = defaultdict(list)
    for (start, end), values in frames.items():
        total = (float(end) - float(start)) * 1e6
        if total <= 0:
            raise ValueError("Non-positive command buffer duration")
        rows["command"].append(total)
        def elapsed(intervals):
            ticks = 0
            prior_end = 0
            for a, b in sorted(intervals):
                if b < a:
                    raise ValueError("Reversed hardware timestamps")
                ticks += max(0, b - max(a, prior_end))
                prior_end = max(prior_end, b)
            return ticks * 1e6 / rate

        copies = []
        for phase in PHASES:
            active, fragments = [], []
            for name, _, _, _, (vs, ve, fs, fe) in values:
                if name != phase:
                    continue
                active.append((vs, ve))
                if phase == "shift_copy":
                    copies.append((vs, ve))
                else:
                    active.append((fs, fe))
                    fragments.append((fs, fe))
                    if phase == "present_copy":
                        copies.append((fs, fe))
            rows[phase].append(elapsed(active))
            if phase != "shift_copy":
                rows[phase + "_fragment"].append(elapsed(fragments))
        rows["copy_pixels"].append(elapsed(copies))
    return {
        "complete_frames": len(frames), "incomplete_frames_with_samples": rejected,
        "ticks_per_second": rate,
        "microseconds": {k: {"mean": mean(v), "p95": sorted(v)[math.ceil(0.95 * len(v)) - 1]}
                         for k, v in rows.items()},
        "copy_percent": 100 * sum(rows["copy_pixels"]) / sum(rows["command"]),
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("capture", type=Path)
    print(json.dumps(analyze(parser.parse_args().capture), indent=2))
