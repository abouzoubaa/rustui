#!/usr/bin/env python3
"""Run five seconds of normal scroll input in one already-built Playground."""
from __future__ import annotations

import argparse
import json
import mmap
import os
import re
import statistics
from pathlib import Path
import subprocess
import time
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
from f0_row_positions import analyze as analyze_row_positions


def summarize(log: str) -> dict:
    active = False
    finished = False
    frames = []
    draws = []
    last_cpu = None
    for line in log.splitlines():
        if line.startswith("RUSTUI_SCROLL_LOG start "):
            active = True
        elif line.startswith("RUSTUI_SCROLL_LOG end "):
            finished = True
            active = False
        elif active and line.startswith("RUSTUI_FRAME_LOG "):
            entry = json.loads(line.removeprefix("RUSTUI_FRAME_LOG "))
            if entry["event"] == "draw":
                last_cpu = entry["cpu_ms"]
                draws.append(entry)
            elif entry["event"] == "present":
                entry["cpu_ms"] += last_cpu or 0
                frames.append(entry)
                last_cpu = None
    if not finished or len(frames) < 2 or not draws:
        raise ValueError(f"incomplete native capture: finished={finished}, draws={len(draws)}, presented={len(frames)}")
    # The first interval reaches back into the two-second warm-up/idle period.
    gaps = [frame["gap_ms"] for frame in frames[1:] if frame["gap_ms"] is not None]
    return {
        "draws": len(draws), "presented": len(frames),
        "cpu_over_8_3_ms": sum(frame["cpu_ms"] > 8.3 for frame in frames),
        "cpu_over_16_7_ms": sum(frame["cpu_ms"] > 16.7 for frame in frames),
        "gaps_over_8_3_ms": sum(gap > 8.3 for gap in gaps),
        "gaps_over_16_7_ms": sum(gap > 16.7 for gap in gaps),
        "worst_gap_ms": max(gaps),
        "visible_rows_painted_empty": None,
    }


def profile(log: str) -> dict:
    """Per-drawn-frame medians. Layout and row lowering nest within phases."""
    active = False
    frames, current = [], {}
    for line in log.splitlines():
        if line.startswith("RUSTUI_SCROLL_LOG start "):
            active = True
        elif line.startswith("RUSTUI_SCROLL_LOG end "):
            active = False
        elif active and line.startswith("RUSTUI_ROW_PROFILE "):
            entry = json.loads(line.removeprefix("RUSTUI_ROW_PROFILE "))
            for field in ("total_ms", "lower_ms"):
                current["row_" + field] = current.get("row_" + field, 0) + entry[field]
        elif active and line.startswith("RUSTUI_FRAME_LOG "):
            entry = json.loads(line.removeprefix("RUSTUI_FRAME_LOG "))
            name = entry["event"]
            current[name] = current.get(name, 0) + entry["cpu_ms"]
            if name == "present":
                if "draw" in current:
                    frames.append(current)
                current = {}
    if not frames or not all("prepaint" in frame for frame in frames):
        raise ValueError("missing native phase events; rebuild the instrumented core and relink the app")
    return {"drawn_frames": len(frames), "median_ms": {
        name: statistics.median(frame.get(name, 0) for frame in frames)
        for name in ("setup", "build", "layout", "prepaint", "paint", "draw", "present", "row_total_ms", "row_lower_ms")}}


def workload_sources(entries):
    """Select the moving native entity, independently of captured pixels.

    Sidebar/hidden lists must not validate stale workload rows at offset zero.
    Legacy captures without entity IDs remain readable for forensic review.
    """
    if not entries or not all("entity_id" in entry for entry in entries):
        return entries
    groups = {}
    for entry in entries:
        groups.setdefault(entry["entity_id"], []).append(entry)
    spans = {key: max(e["source"]["offset_y"] for e in values) - min(e["source"]["offset_y"] for e in values)
             for key, values in groups.items()}
    moving = [key for key, span in spans.items() if span > 0]
    if len(moving) != 1:
        raise ValueError(f"expected one moving workload entity, got spans={spans}")
    return groups[moving[0]]


def visual_sources(logical, visual):
    """Carry the last painted displacement through idle callback-only frames.

    A logical change cannot inherit an old visual sample: that state has not
    been painted yet. Keep every genuine prepaint sample as well.
    """
    if not visual:
        return logical
    visual = sorted(visual, key=lambda entry: entry["epoch_ms"])
    result, cursor = list(visual), -1
    for entry in logical:
        while cursor + 1 < len(visual) and visual[cursor + 1]["epoch_ms"] <= entry["epoch_ms"]:
            cursor += 1
        if cursor >= 0 and entry["source"]["offset_y"] == visual[cursor]["source"]["offset_y"]:
            result.append(dict(epoch_ms=entry["epoch_ms"], source=visual[cursor]["source"]))
    return sorted(result, key=lambda entry: entry["epoch_ms"])


def capture(binary: Path, panel: str, log: Path, pixels: bool = False, capture_only: bool = False) -> dict:
    # Cargo treats registry sources as immutable. Reusing a pre-patch artifact
    # can otherwise silently produce a release binary without the recorder.
    with binary.open("rb") as executable, mmap.mmap(executable.fileno(), 0, access=mmap.ACCESS_READ) as contents:
        if contents.find(b"RUSTUI_FRAME_LOG") < 0:
            raise ValueError("binary lacks the frame recorder; rebuild in a fresh backend target directory")
    env = dict(os.environ, RUSTUI_F0_ACCEPTANCE="1", RUSTUI_F0_PANEL=panel,
               RUSTUI_F0_SCROLL="1", RUSTUI_FRAME_LOG="1")
    log.parent.mkdir(parents=True, exist_ok=True)
    pixel_dir = log.with_suffix(".pixels")
    recorder = None
    if pixels:
        pixel_dir.mkdir(parents=True, exist_ok=False)
        env.update(RUSTUI_F0_PIXELS="1", RUSTUI_F0_SOURCE_DUMP=str(pixel_dir / "source.rs"))
    with log.open("w") as output:
        process = subprocess.Popen([str(binary)], env=env, stdout=output, stderr=subprocess.STDOUT)
        try:
            if pixels:
                for _ in range(500):
                    viewport_lines = [line for line in log.read_text().splitlines() if line.startswith("RUSTUI_VIEWPORT_LOG ")]
                    if viewport_lines: break
                    if process.poll() is not None: raise RuntimeError(f"Playground exited before geometry: {process.returncode}; {log}")
                    time.sleep(0.02)
                (pixel_dir / "viewport.json").write_text(viewport_lines[-1].removeprefix("RUSTUI_VIEWPORT_LOG "))
                helper = Path.home() / ".cache/rustui-target/f0-capture-frames"
                with (pixel_dir / "capture.txt").open("w") as recorder_output:
                    recorder = subprocess.Popen([str(helper), "capture", str(process.pid), str(pixel_dir), str(pixel_dir / "stop")], stdout=recorder_output, stderr=subprocess.STDOUT)
            deadline = time.monotonic() + 30
            while time.monotonic() < deadline:
                if "RUSTUI_SCROLL_LOG end " in log.read_text():
                    break
                if process.poll() is not None:
                    raise RuntimeError(f"Playground exited before capture: {process.returncode}; {log}")
                time.sleep(0.1)
            else:
                raise RuntimeError(f"native capture timed out; {log}")
        finally:
            capture_error = None
            if recorder is not None:
                (pixel_dir / "stop").touch()
                recorder.wait(timeout=20)
                if recorder.returncode != 0:
                    capture_error = RuntimeError(f"native pixel capture failed: {recorder.returncode}; {log}")
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
            if capture_error is not None:
                raise capture_error
    result = summarize(log.read_text())
    if pixels:
        lines = log.read_text().splitlines()
        viewport = [json.loads(line.removeprefix("RUSTUI_VIEWPORT_LOG ")) for line in lines if line.startswith("RUSTUI_VIEWPORT_LOG ") and json.loads(line.removeprefix("RUSTUI_VIEWPORT_LOG "))["panel"] == panel][-1]
        viewport.pop("panel", None)
        visual_oracles = [json.loads(line.removeprefix("RUSTUI_VISUAL_ORACLE_LOG ")) for line in lines if line.startswith("RUSTUI_VISUAL_ORACLE_LOG ")]
        oracles = visual_sources(workload_sources([json.loads(line.removeprefix("RUSTUI_ORACLE_LOG ")) for line in lines if line.startswith("RUSTUI_ORACLE_LOG ")]), visual_oracles)
        start = float(re.search(r"RUSTUI_SCROLL_LOG start .* epoch_ms=([0-9]+)", log.read_text())[1])
        end = float(re.search(r"RUSTUI_SCROLL_LOG end .* epoch_ms=([0-9]+)", log.read_text())[1])
        (pixel_dir / "setup.json").write_text(json.dumps(dict(cropped=True, oracles=oracles, viewport=viewport, start=start, end=end, inputs=[[float(v) for v in line.split()[1:]] for line in lines if line.startswith("RUSTUI_SCROLL_STEP ")])))
        metadata = [json.loads(line) for line in (pixel_dir / "frames.jsonl").read_text().splitlines()]
        if any(frame.get("timestamp_kind") == "sample-fallback" for frame in metadata):
            raise ValueError("native capture is missing WindowServer display timestamps")
        if capture_only:
            result["pixel_directory"] = str(pixel_dir)
            return result
        analysis = subprocess.run([str(helper), "analyze", str(pixel_dir), panel], check=True, text=True, capture_output=True)
        row_result = json.loads(analysis.stdout)
        if row_result["frames_without_oracle"]:
            raise ValueError(f"pixel capture has {row_result['frames_without_oracle']} frames without source positions")
        (pixel_dir / "summary.json").write_text(json.dumps(row_result, indent=2))
        row_result["position_match"] = analyze_row_positions(pixel_dir, panel)
        result["pixels"] = row_result
        result["visible_rows_painted_empty"] = row_result["blank_rows"]
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--panel", choices=["collection", "list", "keyed", "editor", "grid"], required=True)
    parser.add_argument("--log", type=Path, required=True)
    parser.add_argument("--pixels", action="store_true", help="Separate native pixel/OCR validation pass; not timing data")
    parser.add_argument("--capture-only", action="store_true", help="Record pixels now; analyze separately after all timing passes")
    args = parser.parse_args()
    if args.capture_only and not args.pixels:
        parser.error("--capture-only requires --pixels")
    result = capture(args.binary.resolve(), args.panel, args.log, args.pixels, args.capture_only)
    if os.environ.get("RUSTUI_FRAME_PROFILE") == "1":
        result["profile"] = profile(args.log.read_text())
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
