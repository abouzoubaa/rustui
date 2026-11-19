#!/usr/bin/env python3
"""Run five seconds of normal scroll input in one already-built Playground."""
from __future__ import annotations

import argparse
import json
import mmap
import os
from pathlib import Path
import subprocess
import time


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


def capture(binary: Path, panel: str, log: Path) -> dict:
    # Cargo treats registry sources as immutable. Reusing a pre-patch artifact
    # can otherwise silently produce a release binary without the recorder.
    with binary.open("rb") as executable, mmap.mmap(executable.fileno(), 0, access=mmap.ACCESS_READ) as contents:
        if contents.find(b"RUSTUI_FRAME_LOG") < 0:
            raise ValueError("binary lacks the frame recorder; rebuild in a fresh backend target directory")
    env = dict(os.environ, RUSTUI_F0_ACCEPTANCE="1", RUSTUI_F0_PANEL=panel,
               RUSTUI_F0_SCROLL="1", RUSTUI_FRAME_LOG="1")
    log.parent.mkdir(parents=True, exist_ok=True)
    with log.open("w") as output:
        process = subprocess.Popen([str(binary)], env=env, stdout=output, stderr=subprocess.STDOUT)
        try:
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
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
    return summarize(log.read_text())


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--panel", choices=["collection", "list", "keyed", "editor", "grid"], required=True)
    parser.add_argument("--log", type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(capture(args.binary.resolve(), args.panel, args.log), indent=2))


if __name__ == "__main__":
    main()
