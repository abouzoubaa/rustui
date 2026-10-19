#!/usr/bin/env python3
"""Run the built opacity probe, optionally comparing native 10-FPS scheduling."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_BINARY = ROOT / "target/headstart/debug/examples/opacity_benchmark"
TIMEOUT_SECONDS = 10
PROBE_MAX_FPS = 10
EXPECTED = {
    "animation_duration_ms": 1_000,
    "midpoint_after_baseline_ms": 500,
    "post_duration_after_baseline_ms": 1_200,
    "quiescence_after_post_duration_ms": 1_500,
    "max_baseline_poll_interval_ms": 250,
    "startup_stability_ms": 750,
    "startup_timeout_ms": 3_000,
    "process_timeout_ms": TIMEOUT_SECONDS * 1_000,
}


def _utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _find_probe_result(stdout: str) -> Path | None:
    paths = [
        Path(line.strip())
        for line in stdout.splitlines()
        if "opacity-frame-acceptance-" in line and line.strip().endswith(".json")
    ]
    return paths[-1] if len(paths) == 1 else None


def _count_map(result: dict[str, Any]) -> dict[str, int]:
    counts = result.get("root_host_lowerings")
    keys = {"baseline", "midpoint", "post_duration", "quiescence"}
    if not isinstance(counts, dict) or set(counts) != keys:
        raise ValueError("root_host_lowerings has an unexpected shape")
    if any(type(value) is not int or value < 0 for value in counts.values()):
        raise ValueError("root_host_lowerings values must be nonnegative integers")
    return counts


def _elapsed_map(result: dict[str, Any]) -> dict[str, int]:
    elapsed = result.get("sample_elapsed_ms")
    keys = {"baseline", "midpoint", "post_duration", "quiescence"}
    if not isinstance(elapsed, dict) or set(elapsed) != keys:
        raise ValueError("sample_elapsed_ms has an unexpected shape")
    if any(type(value) is not int or value < 0 for value in elapsed.values()):
        raise ValueError("sample_elapsed_ms values must be nonnegative integers")
    values = [elapsed[key] for key in ("baseline", "midpoint", "post_duration", "quiescence")]
    if values != sorted(values) or len(set(values)) != len(values):
        raise ValueError("sample_elapsed_ms must increase strictly")
    return elapsed


def _validate_result(result: Any, mode: str) -> list[str]:
    errors: list[str] = []
    if not isinstance(result, dict):
        return ["probe result must be a JSON object"]
    if result.get("kind") != "rustui_opacity_frame_acceptance":
        errors.append("unexpected result kind")
    if result.get("status") != "complete":
        errors.append(f"probe status is {result.get('status')!r}, expected 'complete'")
    if result.get("mode") != mode:
        errors.append(f"probe mode is {result.get('mode')!r}, expected {mode!r}")
    if result.get("error") is not None:
        errors.append(f"probe reported error: {result.get('error')!r}")
    expected_reduce_motion = mode in {"reduce_motion", "limited_reduce_motion"}
    expected_max_fps = PROBE_MAX_FPS if mode in {"limited", "limited_reduce_motion"} else None
    if result.get("configured_max_fps") != expected_max_fps:
        errors.append("configured_max_fps does not match the process mode")
    if result.get("requested_reduce_motion") is not expected_reduce_motion:
        errors.append("requested_reduce_motion does not match the process mode")
    if result.get("effective_reduce_motion") is not expected_reduce_motion:
        errors.append("effective app reduce_motion flag does not match the process mode")
    if result.get("reduce_motion_confirmed") is not True:
        errors.append("reduce_motion_confirmed must confirm requested and effective settings match")
    if result.get("measurement_scope") != (
        "RustUI host lowerings only; not exact GPUI frame count, GPU frames, "
        "presentation timing, or captured pixel alpha"
    ):
        errors.append("measurement scope is missing or unexpected")
    if result.get("reduced_motion_endpoint") != (
        "GPUI one-shot contract resolves to to=1.0"
        if expected_reduce_motion
        else "not applicable"
    ):
        errors.append("reduced-motion endpoint statement is unexpected")

    for key, expected in EXPECTED.items():
        if result.get(key) != expected:
            errors.append(f"{key} is {result.get(key)!r}, expected {expected!r}")

    try:
        counts = _count_map(result)
        elapsed = _elapsed_map(result)
    except ValueError as error:
        errors.append(str(error))
        return errors

    if counts["baseline"] == 0:
        errors.append("baseline contains no probe host lowering")
    poll_interval = result.get("baseline_poll_interval_ms")
    if type(poll_interval) is not int or not 0 <= poll_interval <= EXPECTED["max_baseline_poll_interval_ms"]:
        errors.append("mounted baseline was not captured within its polling interval bound")
    if result.get("baseline_stability_ms") is not None:
        errors.append("mounted baseline unexpectedly used a stabilization wait")
    startup_wait = result.get("startup_wait_ms")
    if type(startup_wait) is not int or not EXPECTED["startup_stability_ms"] <= startup_wait <= EXPECTED["startup_timeout_ms"]:
        errors.append("static startup baseline did not stabilize within its bounded window")
    if type(result.get("startup_stable_lowerings")) is not int or result["startup_stable_lowerings"] <= 0:
        errors.append("static startup phase has no stable root lowering count")
    mount_elapsed = result.get("mount_elapsed_ms")
    if type(mount_elapsed) is not int or mount_elapsed < startup_wait:
        errors.append("animation mount timestamp is missing or precedes startup stabilization")
    elif not 0 <= elapsed["baseline"] - mount_elapsed <= EXPECTED["max_baseline_poll_interval_ms"]:
        errors.append("first animation baseline exceeded the mount polling bound")
    if result.get("baseline_timing_valid") is not True:
        errors.append("baseline timing was not valid")

    # Allow timer and app-update dispatch jitter while retaining the declared
    # sample phases. The probe's own count checks remain the progress evidence.
    if not 450 <= elapsed["midpoint"] - elapsed["baseline"] <= 900:
        errors.append("midpoint timing is outside the expected 500 ms window")
    if not 1_100 <= elapsed["post_duration"] - elapsed["baseline"] <= 2_000:
        errors.append("post-duration timing is outside the expected 1.2 s window")
    if not 1_400 <= elapsed["quiescence"] - elapsed["post_duration"] <= 3_000:
        errors.append("quiescence timing is outside the expected 1.5 s window")

    if not expected_reduce_motion:
        if counts["midpoint"] <= counts["baseline"]:
            errors.append("normal mode showed no host-lowering increase by midpoint")
        if counts["post_duration"] <= counts["midpoint"]:
            errors.append("normal mode showed no host-lowering increase after duration")
        if counts["quiescence"] != counts["post_duration"]:
            errors.append("normal mode host-lowering count did not quiesce")
        if result.get("normal_progress_observed") is not True:
            errors.append("probe did not report normal progress")
    else:
        if any(counts[key] != counts["baseline"] for key in ("midpoint", "post_duration", "quiescence")):
            errors.append("reduced-motion mode scheduled further host lowerings")
        if result.get("normal_progress_observed") is not False:
            errors.append("reduced-motion result has inconsistent normal progress flag")
    if result.get("quiescent_after_animation") is not (counts["quiescence"] == counts["post_duration"]):
        errors.append("quiescence flag does not match sampled counts")
    return errors


def _decode_output(output: str | bytes | None) -> str:
    if output is None:
        return ""
    if isinstance(output, bytes):
        return output.decode("utf-8", errors="replace")
    return output


def _run(binary: Path, mode: str, args: list[str]) -> dict[str, Any]:
    entry: dict[str, Any] = {
        "mode": mode,
        "command": [str(binary), *args],
        "arguments": args,
        "started_at": _utc_now(),
        "finished_at": None,
        "timeout_seconds": TIMEOUT_SECONDS,
        "timed_out": False,
        "returncode": None,
        "stdout": "",
        "stderr": "",
        "probe_result_path": None,
        "probe_result": None,
        "validation_errors": [],
    }
    try:
        completed = subprocess.run(
            [str(binary), *args],
            cwd=ROOT,
            capture_output=True,
            text=True,
            timeout=TIMEOUT_SECONDS,
            check=False,
        )
        entry["returncode"] = completed.returncode
        entry["stdout"] = completed.stdout
        entry["stderr"] = completed.stderr
    except subprocess.TimeoutExpired as error:
        entry["timed_out"] = True
        entry["finished_at"] = _utc_now()
        entry["stdout"] = _decode_output(error.stdout)
        entry["stderr"] = _decode_output(error.stderr)
        entry["validation_errors"].append("probe process exceeded hard 10-second timeout")
        return entry
    except OSError as error:
        entry["finished_at"] = _utc_now()
        entry["validation_errors"].append(f"could not start probe: {error}")
        return entry

    entry["finished_at"] = _utc_now()

    if completed.returncode != 0:
        entry["validation_errors"].append(f"probe exited with status {completed.returncode}")
    result_path = _find_probe_result(completed.stdout)
    if result_path is None:
        entry["validation_errors"].append("probe stdout did not identify exactly one result JSON")
        return entry
    entry["probe_result_path"] = str(result_path)
    try:
        result_path = result_path.resolve(strict=True)
        expected_output_dir = (ROOT / "target/headstart").resolve()
        if result_path.parent != expected_output_dir:
            entry["validation_errors"].append(
                "probe result JSON is outside target/headstart"
            )
            return entry
    except OSError as error:
        entry["validation_errors"].append(f"probe result path is unavailable: {error}")
        return entry
    try:
        with result_path.open(encoding="utf-8") as source:
            result = json.load(source)
    except (OSError, json.JSONDecodeError) as error:
        entry["validation_errors"].append(f"could not read probe result JSON: {error}")
        return entry
    entry["probe_result"] = result
    entry["validation_errors"].extend(_validate_result(result, mode))
    return entry


def _write_evidence(output_dir: Path, evidence: dict[str, Any]) -> Path:
    output_dir.mkdir(parents=True, exist_ok=True)
    while True:
        path = output_dir / f"opacity-probe-verification-{os.getpid()}-{time.time_ns()}.json"
        try:
            with path.open("x", encoding="utf-8") as output:
                json.dump(evidence, output, indent=2, sort_keys=True)
                output.write("\n")
            return path
        except FileExistsError:
            continue


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--binary",
        type=Path,
        default=DEFAULT_BINARY,
        help=f"already-built opacity_benchmark executable (default: {DEFAULT_BINARY})",
    )
    parser.add_argument(
        "--with-frame-limit", action="store_true",
        help="also run limited and limited-reduced-motion processes with native 10-FPS scheduling",
    )
    args = parser.parse_args()
    binary = args.binary.expanduser().resolve()
    output_dir = ROOT / "target/headstart"
    evidence: dict[str, Any] = {
        "kind": "rustui_opacity_probe_verification",
        "status": "failed",
        "started_at": _utc_now(),
        "binary": str(binary),
        "provenance": {
            "binary_sha256": None,
            "probe_source": str(ROOT / "crates/rustui/examples/opacity_benchmark.rs"),
            "probe_source_sha256": None,
            "runner_source": str(Path(__file__).resolve()),
            "runner_source_sha256": None,
        },
        "hard_timeout_seconds_per_process": TIMEOUT_SECONDS,
        "frame_limit_requested": args.with_frame_limit,
        "runs": [],
    }
    if not binary.is_file() or not os.access(binary, os.X_OK):
        evidence["error"] = "built probe binary is missing or not executable"
    else:
        try:
            evidence["provenance"]["binary_sha256"] = _sha256(binary)
            evidence["provenance"]["probe_source_sha256"] = _sha256(
                ROOT / "crates/rustui/examples/opacity_benchmark.rs"
            )
            evidence["provenance"]["runner_source_sha256"] = _sha256(Path(__file__).resolve())
        except OSError as error:
            evidence["error"] = f"could not hash probe provenance: {error}"
        if "error" in evidence:
            evidence["finished_at"] = _utc_now()
            evidence_path = _write_evidence(output_dir, evidence)
            print(evidence_path)
            print(json.dumps(evidence, indent=2), file=sys.stderr)
            return 1
        evidence["runs"] = [
            _run(binary, "normal", []),
            _run(binary, "reduce_motion", ["--reduce-motion"]),
        ]
        if args.with_frame_limit:
            evidence["runs"].extend([
                _run(binary, "limited", ["--limit-fps"]),
                _run(binary, "limited_reduce_motion", ["--limit-fps", "--reduce-motion"]),
            ])
            normal, limited = evidence["runs"][0], evidence["runs"][2]
            comparison: dict[str, Any] = {
                "status": "unconfirmed",
                "configured_max_fps": PROBE_MAX_FPS,
                "normal_lowering_delta": None,
                "limited_lowering_delta": None,
                "fewer_host_lowerings_observed": False,
                "measurement_scope": "Observed isolated host-lowering counts; not a strict wall-clock cap or displayed FPS",
            }
            if not normal["validation_errors"] and not limited["validation_errors"]:
                normal_counts = _count_map(normal["probe_result"])
                limited_counts = _count_map(limited["probe_result"])
                normal_delta = normal_counts["post_duration"] - normal_counts["baseline"]
                limited_delta = limited_counts["post_duration"] - limited_counts["baseline"]
                observed = 0 < limited_delta < normal_delta
                comparison.update({
                    "status": "complete" if observed else "unconfirmed",
                    "normal_lowering_delta": normal_delta,
                    "limited_lowering_delta": limited_delta,
                    "fewer_host_lowerings_observed": observed,
                })
            evidence["frame_limit_comparison"] = comparison
        evidence["status"] = (
            "complete"
            if all(not run["validation_errors"] for run in evidence["runs"])
            and (not args.with_frame_limit or evidence["frame_limit_comparison"]["status"] == "complete")
            else "failed"
        )
    evidence["finished_at"] = _utc_now()
    evidence_path = _write_evidence(output_dir, evidence)
    print(evidence_path)
    if evidence["status"] != "complete":
        print(json.dumps(evidence, indent=2), file=sys.stderr)
        return 1
    for run in evidence["runs"]:
        print(f"{run['mode']}: complete ({run['probe_result_path']})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
