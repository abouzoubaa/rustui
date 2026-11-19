"""Match recorded row labels to read-only native scroll positions.

ScreenCaptureKit delivers pixels after submission. Calibrate that fixed delay
from the first moving frames, then check every subsequent captured row too.
"""
import json
import re
import statistics


def anchors(frame, panel):
    pattern = r"^\s*(\d+)\s*$" if panel == "editor" else r"^Product\s+(\d+)$" if panel == "grid" else r"^[Ff][O0]\s*row\s*(\d+)$"
    height = frame.get("height", float("inf"))
    margin = frame.get("pitch", 0) * .5
    return [(int(match[1]), text["center"]) for text in frame["texts"]
            if margin <= text["center"] <= height - margin
            and (panel != "editor" or text.get("right", text.get("x", 0)) < 140)
            and (match := re.match(pattern, text["text"]))]


def analyze(folder, panel):
    setup = json.loads((folder / "setup.json").read_text())
    frames = [json.loads(line) for line in (folder / "rows.jsonl").read_text().splitlines()]
    geometry_path = folder / "frames.jsonl"
    if geometry_path.is_file():
        geometry = {row["frame"]: row for row in map(json.loads, geometry_path.read_text().splitlines())}
        for frame in frames:
            frame["height"] = geometry[frame["frame"]]["height"]
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
        return abs(center - (shift + (ident - base) * pitch + (source["offset_y"] + source.get("visual_offset_y", 0)) * scale))

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
    # Keep the original 16.7 ms candidate count. A source observer sampled at
    # 30 Hz cannot resolve position within a 16.7 ms window; report a second
    # count using one measured observer interval, plus its 1 ms clock quantum.
    timestamps = sorted({oracle["epoch_ms"] for oracle in oracles})
    intervals = [b - a for a, b in zip(timestamps, timestamps[1:]) if b > a]
    interval = statistics.median(intervals) if intervals else 0
    tolerance = max(16.7, interval + 1)
    cadence_count = cadence_missing = 0
    count = missing = observed = stale_held = 0
    with (folder / "row-positions.jsonl").open("w") as output:
        for frame in frames:
            nearby = [oracle["source"] for oracle in oracles if abs(oracle["epoch_ms"] - (frame["epoch_ms"] - lag)) <= 16.7]
            cadence_nearby = [oracle["source"] for oracle in oracles if abs(oracle["epoch_ms"] - (frame["epoch_ms"] - lag)) <= tolerance]
            found = anchors(frame, panel)
            cadence_stale = sum(not any(error(source, ident, center) <= pitch * .45 for source in cadence_nearby) for ident, center in found)
            cadence_count += cadence_stale
            cadence_missing += not bool(cadence_nearby)
            stale = sum(not any(error(source, ident, center) <= pitch * .45 for source in nearby) for ident, center in found)
            missing += not bool(nearby)
            observed += len(found)
            count += stale
            stale_held += bool(stale and frame.get("stalled"))
            output.write(json.dumps(dict(frame=frame["frame"], recognized_rows=len(found), stale=stale, source_available=bool(nearby), cadence_aware_stale=cadence_stale, cadence_source_available=bool(cadence_nearby))) + "\n")
    return dict(cadence_aware_position_candidates=cadence_count, alignment_tolerance_ms=tolerance, source_sample_interval_ms=interval, cadence_frames_without_source=cadence_missing, stale_rows=count, held_position_mismatch_frames=stale_held, recognized_row_observations=observed, capture_delay_ms=lag, delay_calibration_samples=len(lags), frames_without_aligned_source=missing)
