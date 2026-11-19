"""Match recorded row labels to read-only native scroll positions.

ScreenCaptureKit delivers pixels after submission. Calibrate that fixed delay
from the first moving frames, then check every subsequent captured row too.
"""
import json
import re
import statistics


def anchors(frame, panel):
    pattern = r"^\s*(\d+)\s*$" if panel == "editor" else r"^Product\s+(\d+)$" if panel == "grid" else r"^[Ff][O0]\s*row\s*(\d+)$"
    return [(int(match[1]), text["center"]) for text in frame["texts"] if (match := re.match(pattern, text["text"]))]


def analyze(folder, panel):
    setup = json.loads((folder / "setup.json").read_text())
    frames = [json.loads(line) for line in (folder / "rows.jsonl").read_text().splitlines()]
    oracles = setup["oracles"]
    scale = setup["viewport"]["scale"]
    viewport_y = setup["viewport"]["y"]
    base = int(panel in ("editor", "grid"))
    calibration_path = folder / "calibration.json"
    if not calibration_path.is_file():
        raise ValueError("missing zero-offset warmup calibration; rerun pixel calibration")
    calibration = json.loads(calibration_path.read_text())
    if calibration["scale"] != scale:
        raise ValueError("warmup calibration scale differs from capture scale")
    pitch = calibration["pitch"]
    shift = calibration["text_shift_px"]

    def error(source, ident, center):
        if panel == "editor":
            return min((abs(center - ((row["y"] - viewport_y) * scale + shift)) for row in source.get("rows", []) if row["id"] == ident), default=float("inf"))
        return abs(center - (shift + (ident - base) * pitch + source["offset_y"] * scale))

    lags = []
    for frame in frames[:100]:
        found = anchors(frame, panel)
        if not found or found[0][0] < 30:
            continue
        ident, center = found[0]
        candidates = [(error(oracle["source"], ident, center), abs(frame["epoch_ms"] - oracle["epoch_ms"]), frame["epoch_ms"] - oracle["epoch_ms"]) for oracle in oracles if abs(oracle["epoch_ms"] - frame["epoch_ms"]) <= 150]
        if candidates:
            position_error, _, lag = min(candidates)
            if position_error <= 3:
                lags.append(lag)
    lag = statistics.median(lags) if len(lags) >= 10 else 0
    count = missing = observed = 0
    with (folder / "row-positions.jsonl").open("w") as output:
        for frame in frames:
            nearby = [oracle["source"] for oracle in oracles if abs(oracle["epoch_ms"] - (frame["epoch_ms"] - lag)) <= 16.7]
            found = anchors(frame, panel)
            stale = sum(not any(error(source, ident, center) <= pitch * .45 for source in nearby) for ident, center in found)
            missing += not bool(nearby)
            observed += len(found)
            count += stale
            output.write(json.dumps(dict(frame=frame["frame"], recognized_rows=len(found), stale=stale, source_available=bool(nearby))) + "\n")
    return dict(stale_rows=count, recognized_row_observations=observed, capture_delay_ms=lag, delay_calibration_samples=len(lags), frames_without_aligned_source=missing)
