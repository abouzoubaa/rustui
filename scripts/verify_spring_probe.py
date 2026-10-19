#!/usr/bin/env python3
"""Run and validate both native SpringOpacity scheduling probe modes."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
import math
import os
from pathlib import Path
import subprocess
import sys
import time
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_BINARY = ROOT / "target/headstart/debug/examples/opacity_benchmark"
SAMPLES = (
    "first_render",
    "before_retarget",
    "second_render",
    "after_retarget",
    "settled",
    "quiescence",
)
MEASUREMENT_SCOPE = (
    "RustUI root host lowerings only; not exact GPUI frames, presented alpha, "
    "or spring velocity continuity"
)


def validate(result: Any, mode: str) -> list[str]:
    errors: list[str] = []
    if not isinstance(result, dict):
        return ["result must be a JSON object"]
    expected_reduced = mode == "reduce_motion"
    for key, expected in {
        "kind": "rustui_spring_opacity_scheduling_probe",
        "status": "complete",
        "mode": mode,
        "error": None,
        "requested_reduce_motion": expected_reduced,
        "effective_reduce_motion": expected_reduced,
        "startup_stability_ms": 750,
        "startup_timeout_ms": 3000,
        "poll_interval_ms": 25,
        "retarget_after_first_render_ms": 250,
        "passive_after_second_render_ms": 250,
        "settle_after_retarget_ms": 2000,
        "quiescence_window_ms": 750,
        "process_timeout_ms": 10000,
        "timings_monotonic": True,
        "quiescent": True,
        "measurement_scope": MEASUREMENT_SCOPE,
    }.items():
        if result.get(key) != expected:
            errors.append(f"{key}: expected {expected!r}, got {result.get(key)!r}")
    targets = result.get("target_sequence")
    if (
        not isinstance(targets, list)
        or len(targets) != 3
        or any(type(value) not in (int, float) or not math.isfinite(value) for value in targets)
        or any(abs(value - expected) > 1e-7 for value, expected in zip(targets, (0.15, 1.0, 0.15)))
    ):
        errors.append("target_sequence must contain finite 0.15, 1.0, 0.15 values within 1e-7")
    stable_lowerings = result.get("startup_stable_lowerings")
    if type(stable_lowerings) is not int or stable_lowerings < 1:
        errors.append("startup did not record a stable host lowering")
    startup_wait = result.get("startup_wait_ms")
    if type(startup_wait) is not int or not 750 <= startup_wait <= 3000:
        errors.append("startup_wait_ms is outside the bounded baseline interval")

    counts = result.get("root_host_lowerings")
    times = result.get("sample_elapsed_ms")
    events = result.get("event_elapsed_ms")
    if not isinstance(counts, dict) or set(counts) != set(SAMPLES):
        return errors + ["root_host_lowerings has an unexpected shape"]
    if not isinstance(times, dict) or set(times) != set(SAMPLES):
        return errors + ["sample_elapsed_ms has an unexpected shape"]
    if not isinstance(events, dict) or set(events) != {"first_update", "retarget"}:
        return errors + ["event_elapsed_ms has an unexpected shape"]
    if any(type(counts[key]) is not int or counts[key] < 0 for key in SAMPLES):
        return errors + ["host lowering counts must be nonnegative integers"]
    if any(type(times[key]) is not int or times[key] < 0 for key in SAMPLES):
        return errors + ["sample times must be nonnegative integers"]
    if any(type(events[key]) is not int or events[key] < 0 for key in events):
        return errors + ["event times must be nonnegative integers"]

    timeline = [
        events["first_update"], times["first_render"],
        times["before_retarget"], events["retarget"],
        times["second_render"], times["after_retarget"],
        times["settled"], times["quiescence"],
    ]
    if any(later < earlier for earlier, later in zip(timeline, timeline[1:])):
        errors.append("event and sample times are not monotonic")
    if times["before_retarget"] - times["first_render"] < 250:
        errors.append("first passive motion window was too short")
    if times["after_retarget"] - times["second_render"] < 250:
        errors.append("second passive motion window was too short")
    if times["settled"] - events["retarget"] < 2000:
        errors.append("settle wait was too short")
    if times["quiescence"] - times["settled"] < 750:
        errors.append("quiescence window was too short")
    ordered_counts = [counts[key] for key in SAMPLES]
    if any(later < earlier for earlier, later in zip(ordered_counts, ordered_counts[1:])):
        errors.append("host lowering count decreased")
    if counts["first_render"] < 1 or counts["second_render"] <= counts["before_retarget"]:
        errors.append("explicit target updates did not cause observable host lowering")
    if counts["settled"] != counts["quiescence"]:
        errors.append("host lowerings continued in the quiescence window")
    if expected_reduced:
        if counts["first_render"] != counts["before_retarget"]:
            errors.append("reduced-motion first passive window was not flat")
        if counts["second_render"] != counts["after_retarget"]:
            errors.append("reduced-motion second passive window was not flat")
        if result.get("first_passive_flat") is not True or result.get("second_passive_flat") is not True:
            errors.append("reduced-motion flatness flags are missing")
    else:
        if counts["before_retarget"] <= counts["first_render"]:
            errors.append("first target did not cause passive scheduling growth")
        if counts["after_retarget"] <= counts["second_render"]:
            errors.append("retarget did not cause passive scheduling growth")
        if result.get("first_motion_growth") is not True or result.get("retarget_motion_growth") is not True:
            errors.append("normal-motion growth flags are missing")
    return errors


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def run_mode(binary: Path, mode: str) -> dict[str, Any]:
    command = [str(binary), "--spring"]
    if mode == "reduce_motion":
        command.append("--reduce-motion")
    record: dict[str, Any] = {
        "mode": mode,
        "command": command,
        "process_exit_code": None,
        "stdout": "",
        "stderr": "",
        "result_artifact": None,
        "result_artifact_sha256": None,
        "result_status": None,
        "errors": [],
    }
    try:
        process = subprocess.run(command, capture_output=True, text=True, timeout=12, check=False)
    except subprocess.TimeoutExpired as timeout:
        record["stdout"] = (timeout.stdout or b"").decode(errors="replace") if isinstance(timeout.stdout, bytes) else (timeout.stdout or "")
        record["stderr"] = (timeout.stderr or b"").decode(errors="replace") if isinstance(timeout.stderr, bytes) else (timeout.stderr or "")
        record["errors"].append(f"{mode}: process exceeded 12-second verifier timeout")
        return record
    record["process_exit_code"] = process.returncode
    record["stdout"] = process.stdout
    record["stderr"] = process.stderr
    paths = [Path(line.strip()) for line in process.stdout.splitlines()
             if "spring-opacity-scheduling-" in line and line.strip().endswith(".json")]
    if len(paths) != 1 or not paths[0].is_file():
        record["errors"].append(f"{mode}: expected one result artifact; exit={process.returncode}; stderr={process.stderr.strip()}")
        return record
    path = paths[0]
    record["result_artifact"] = str(path.resolve())
    try:
        record["result_artifact_sha256"] = sha256(path)
        result = json.loads(path.read_text())
    except (OSError, json.JSONDecodeError) as error:
        record["errors"].append(f"{mode}: cannot read result: {error}")
        return record
    record["result_status"] = result.get("status") if isinstance(result, dict) else None
    record["errors"].extend(validate(result, mode))
    if process.returncode != 0:
        record["errors"].append(f"{mode}: process exited {process.returncode}: {process.stderr.strip()}")
    return record


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=DEFAULT_BINARY)
    args = parser.parse_args()
    if not args.binary.is_file():
        parser.error(f"binary does not exist: {args.binary}")
    runs = [run_mode(args.binary, mode) for mode in ("normal", "reduce_motion")]
    failed = any(run["errors"] for run in runs)
    report = {
        "kind": "rustui_spring_opacity_probe_verification",
        "status": "failed" if failed else "passed",
        "created_at_utc": datetime.now(timezone.utc).isoformat(),
        "binary": str(args.binary.resolve()),
        "binary_sha256": sha256(args.binary),
        "verifier_sha256": sha256(Path(__file__)),
        "runs": runs,
    }
    report_dir = ROOT / "target/headstart"
    report_dir.mkdir(parents=True, exist_ok=True)
    report_path = report_dir / f"spring-opacity-verification-{os.getpid()}-{time.time_ns()}.json"
    with report_path.open("x") as output:
        json.dump(report, output, indent=2)
        output.write("\n")
    for run in runs:
        print(f"{run['mode']}: {run['result_artifact'] or 'no artifact'}")
        for error in run["errors"]:
            print(f"  {error}", file=sys.stderr)
    print(f"verification report: {report_path}")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
