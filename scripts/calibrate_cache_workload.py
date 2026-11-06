#!/usr/bin/env python3
"""Repeat the existing cache example and record whole-process CPU and RSS.

This is a same-binary calibration, not a cached-versus-uncached speed comparison.
It launches an already built example directly and never invokes Cargo.
"""

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import platform
import signal
import statistics
import subprocess
import sys
import tempfile
import time


ROOT = Path(__file__).resolve().parent.parent
BINARY = ROOT / "target/headstart/debug/examples/cache_benchmark"
WARMUP_RUNS = 1
MEASURED_RUNS = 5
TIMEOUT_SECONDS = 45
REAP_SECONDS = 5
SCENARIOS = {"small": 1, "medium": 8, "large": 32}


def json_value_matches(expected, actual):
    """Compare decoded JSON without Python's bool/int equivalence."""
    if type(expected) is not type(actual):
        return False
    if isinstance(expected, dict):
        return expected.keys() == actual.keys() and all(
            json_value_matches(value, actual[key]) for key, value in expected.items()
        )
    if isinstance(expected, list):
        return len(expected) == len(actual) and all(
            json_value_matches(left, right) for left, right in zip(expected, actual)
        )
    return expected == actual


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def version(command):
    result = subprocess.run(command, capture_output=True, text=True, timeout=10, check=True)
    return result.stdout.strip()


def file_identity(path):
    path = path.resolve(strict=True)
    return {"path": str(path), "sha256": sha256(path), "size_bytes": path.stat().st_size}


def compiler_identity():
    host_version = version(["rustc", "-vV"])
    host = next(line.removeprefix("host: ") for line in host_version.splitlines() if line.startswith("host: "))
    headstart_root = Path(os.environ.get("RUSTUI_HEADSTART_ROOT", Path.home() / ".local/opt/headstart"))
    rustc = headstart_root / "rustc/build" / host / "stage1/bin/rustc"
    return {**file_identity(rustc), "version_verbose": version([str(rustc), "-vV"])}


def check_workload(result, scenario):
    if not isinstance(result, dict):
        raise ValueError("workload must be an object")
    expected_children = SCENARIOS[scenario]
    expected = {
        "schema_version": 1,
        "kind": "rustui_cached_child_workload",
        "status": "complete",
        "ax_active_seen": False,
        "warmup_parent_updates": 10,
        "measured_parent_updates": 100,
        "scenario": scenario,
        "retained_children_per_side": expected_children,
        "cache_hit_observed": True,
        "child_invalidation_observed": True,
    }
    for field, value in expected.items():
        if not json_value_matches(value, result.get(field)):
            raise ValueError(f"workload {field}: expected {value!r}, got {result.get(field)!r}")
    expected_counts = {
        "cached_after_parent_updates": 0,
        "uncached_after_parent_updates": expected_children * 100,
    }
    counts = result.get("counts")
    if not isinstance(counts, dict):
        raise ValueError("workload counts must be an object")
    for field, value in expected_counts.items():
        if not json_value_matches(value, counts.get(field)):
            raise ValueError(f"workload counts.{field}: expected {value}, got {counts.get(field)!r}")
    if not json_value_matches({"cached": 1, "uncached": 1}, result.get("rendered_values")):
        raise ValueError(f"unexpected rendered values: {result.get('rendered_values')!r}")


def run_once(binary, phase, number, scenario):
    with tempfile.TemporaryFile() as stdout, tempfile.TemporaryFile() as stderr:
        started = time.monotonic_ns()
        process = subprocess.Popen(
        [str(binary), "--scenario", scenario],
        cwd=ROOT,
        stdout=stdout,
        stderr=stderr,
        start_new_session=True,
        )
        deadline = time.monotonic() + TIMEOUT_SECONDS
        timed_out = False
        reaped = False
        status = None
        usage = None
        while True:
            pid, status, usage = os.wait4(process.pid, os.WNOHANG)
            if pid:
                reaped = True
                break
            if time.monotonic() >= deadline:
                timed_out = True
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                reap_deadline = time.monotonic() + REAP_SECONDS
                while time.monotonic() < reap_deadline:
                    pid, status, usage = os.wait4(process.pid, os.WNOHANG)
                    if pid:
                        reaped = True
                        break
                    time.sleep(0.01)
                break
            time.sleep(0.01)
        if reaped:
            process.returncode = os.waitstatus_to_exitcode(status)
        elapsed_ns = time.monotonic_ns() - started
        stdout.seek(0)
        stderr.seek(0)
        output = stdout.read().decode("utf-8", errors="replace")
        errors = stderr.read().decode("utf-8", errors="replace")
    sample = {
        "phase": phase,
        "run": number,
        "scenario": scenario,
        "elapsed_ns": elapsed_ns,
        "exit_code": process.returncode,
        "timed_out": timed_out,
        "reaped": reaped,
        "user_cpu_seconds": usage.ru_utime if usage else None,
        "system_cpu_seconds": usage.ru_stime if usage else None,
        "total_cpu_seconds": usage.ru_utime + usage.ru_stime if usage else None,
        "peak_rss_bytes": usage.ru_maxrss if usage else None,  # Darwin wait4 reports bytes.
        "stdout": output,
        "stderr": errors,
    }
    try:
        paths = [Path(line.strip()) for line in output.splitlines() if line.strip().endswith(".json")]
        result = None
        if len(paths) == 1 and paths[0].parent.resolve() == (ROOT / "target/headstart").resolve():
            result_path = paths[0].resolve(strict=True)
            result = json.loads(result_path.read_text(encoding="utf-8"))
            sample["workload"] = file_identity(result_path)
            sample["workload_result"] = result
        if timed_out:
            raise ValueError(f"exceeded {TIMEOUT_SECONDS}s; process group killed; reaped={reaped} within {REAP_SECONDS}s")
        if process.returncode != 0:
            raise ValueError(f"exited {process.returncode}")
        if result is None:
            raise ValueError("did not report exactly one workload JSON under target/headstart")
        check_workload(result, scenario)
        sample["validation"] = "accepted"
    except (OSError, ValueError, KeyError, TypeError) as error:
        sample["validation"] = "rejected"
        sample["validation_error"] = str(error)
    return sample


def spread(values):
    median = statistics.median(values)
    return {
        "median": median,
        "min": min(values),
        "max": max(values),
        "range": max(values) - min(values),
        "range_percent_of_median": (max(values) - min(values)) / median * 100 if median else None,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=BINARY, help="existing cache_benchmark executable")
    parser.add_argument("--output", type=Path, help="new JSON record path under target/headstart")
    parser.add_argument("--build-manifest", type=Path,
                        help="existing record_build_provenance.py JSON for this binary's build")
    parser.add_argument(
        "--scenario",
        choices=("all", *SCENARIOS),
        default="all",
        help="scenario to calibrate (default: all three retained-child counts)",
    )
    args = parser.parse_args()
    if platform.system() != "Darwin":
        parser.error("peak RSS units are calibrated for macOS wait4 only")
    binary = args.binary.resolve(strict=True)
    output = args.output or ROOT / "target/headstart" / f"cache-calibration-{time.time_ns()}.json"
    output = output.resolve()
    allowed_output_dir = (ROOT / "target/headstart").resolve()
    if not output.is_relative_to(allowed_output_dir) or output == allowed_output_dir:
        parser.error("--output must be a file under target/headstart")
    if output.exists():
        parser.error(f"output already exists: {output}")
    output.parent.mkdir(parents=True, exist_ok=True)
    record = {
        "schema_version": 1,
        "kind": "rustui_cache_workload_cpu_rss_calibration",
        "status": "running",
        "created_at_utc": datetime.now(timezone.utc).isoformat(),
        "purpose": "same-binary, whole-process repeatability including startup, warmup, measured frames, and teardown; no cached-versus-uncached CPU attribution",
        "protocol": {"fresh_process_warmup_runs_per_scenario": WARMUP_RUNS, "fresh_process_measured_runs_per_scenario": MEASURED_RUNS,
                     "example_warmup_parent_updates": 10, "example_measured_parent_updates": 100,
                     "timeout_seconds_per_run": TIMEOUT_SECONDS},
        "scenarios": [
            {"name": name, "retained_children_per_side": SCENARIOS[name]}
            for name in (SCENARIOS if args.scenario == "all" else (args.scenario,))
        ],
        "machine": {"platform": platform.platform(), "machine": platform.machine(),
                    "processor": platform.processor(), "cpu_count": os.cpu_count()},
        "compiler": compiler_identity(),
        "binary": file_identity(binary),
        "inputs": {name: file_identity(ROOT / name) for name in
                   ("crates/rustui/examples/cache_benchmark.rs", "Cargo.lock", "scripts/cargo-headstart",
                    "scripts/calibrate_cache_workload.py")},
        "samples": [],
    }
    if args.build_manifest is not None:
        manifest_path = args.build_manifest.resolve(strict=True)
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        record["build_manifest"] = {
            **file_identity(manifest_path),
            "declared_build_facts": manifest.get("build_facts_declared_by_caller"),
            "time_scope_note": manifest.get("time_scope_note"),
            "note": "Linked provenance; this record does not prove the selected binary came from that build.",
        }
    try:
        record["summary_by_scenario"] = {}
        for scenario in (SCENARIOS if args.scenario == "all" else (args.scenario,)):
            for phase, count in (("warmup", WARMUP_RUNS), ("measured", MEASURED_RUNS)):
                for number in range(1, count + 1):
                    sample = run_once(binary, phase, number, scenario)
                    record["samples"].append(sample)
                    if sample["validation"] != "accepted":
                        raise ValueError(f"{scenario} {phase} run {number}: {sample['validation_error']}")
                    print(f"{scenario} {phase} {number}/{count}: CPU {sample['total_cpu_seconds']:.6f}s, RSS {sample['peak_rss_bytes']} B", flush=True)
            measured = [sample for sample in record["samples"] if sample["phase"] == "measured" and sample["scenario"] == scenario]
            record["summary_by_scenario"][scenario] = {
                "retained_children_per_side": SCENARIOS[scenario],
                "measured_runs": MEASURED_RUNS,
                "total_cpu_seconds": spread([sample["total_cpu_seconds"] for sample in measured]),
                "peak_rss_bytes": spread([sample["peak_rss_bytes"] for sample in measured]),
                "elapsed_seconds": spread([sample["elapsed_ns"] / 1e9 for sample in measured]),
            }
        record["status"] = "complete"
    except (AssertionError, OSError, ValueError, KeyError, RuntimeError) as error:
        record["status"] = "failed"
        record["error"] = str(error)
    with output.open("x", encoding="utf-8") as stream:
        json.dump(record, stream, indent=2)
        stream.write("\n")
    print(output)
    return 0 if record["status"] == "complete" else 1


if __name__ == "__main__":
    sys.exit(main())
