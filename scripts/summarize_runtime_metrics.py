#!/usr/bin/env python3
"""Summarize external process samples without making performance claims."""

import argparse
import json
import math
from pathlib import Path
import sys

from runtime_metrics import write_json_exclusive, validate_output_path


SCHEMA_VERSION = 1
REQUIRED_AVAILABILITY = {
    "cycle_time": "unavailable",
    "frame_time": "unavailable",
    "main_thread_time": "unavailable",
}


class RecordError(ValueError):
    pass


def _number(value, field, *, positive=False):
    if isinstance(value, bool) or not isinstance(value, int):
        raise RecordError(f"{field} must be an integer")
    try:
        finite = math.isfinite(value)
    except OverflowError:
        finite = False
    if not finite or (positive and value <= 0):
        raise RecordError(f"{field} must be a finite {'positive ' if positive else ''}number")
    return value


def validate_record(record):
    if not isinstance(record, dict):
        raise RecordError("record must be a JSON object")
    version = record.get("schema_version")
    if isinstance(version, bool) or not isinstance(version, int) or version != SCHEMA_VERSION:
        raise RecordError("schema_version must be 1")
    if record.get("kind") != "rustui_process_metrics":
        raise RecordError('kind must be "rustui_process_metrics"')
    for key in ("run_id", "label", "workload"):
        if not isinstance(record.get(key), str) or not record[key].strip():
            raise RecordError(f"{key} must be a non-empty string")
    pid = record.get("pid")
    if isinstance(pid, bool) or not isinstance(pid, int) or pid <= 0:
        raise RecordError("pid must be a positive integer")
    identity = record.get("identity")
    if (not isinstance(identity, dict) or isinstance(identity.get("pid"), bool)
            or not isinstance(identity.get("pid"), int) or identity["pid"] != pid
            or not isinstance(identity.get("lstart"), str) or not identity["lstart"].strip()):
        raise RecordError("identity must contain the matching pid and non-empty lstart")
    completion = record.get("completion")
    if (not isinstance(completion, dict)
            or completion.get("status") not in ("complete", "incomplete")
            or "error" not in completion
            or (completion.get("error") is not None and not isinstance(completion.get("error"), str))):
        raise RecordError("completion must contain complete/incomplete status and string-or-null error")
    if (completion["status"] == "complete" and completion["error"] is not None
            or completion["status"] == "incomplete"
            and (not isinstance(completion["error"], str) or not completion["error"].strip())):
        raise RecordError("complete records require null error; incomplete records require a non-empty error")
    availability = record.get("metrics_availability")
    if not isinstance(availability, dict) or any(availability.get(k) != v for k, v in REQUIRED_AVAILABILITY.items()):
        raise RecordError("metrics_availability must mark cycle, frame, and main_thread time unavailable")
    samples = record.get("samples")
    if not isinstance(samples, list) or not samples:
        raise RecordError("samples must be a non-empty list")
    elapsed, cpu, rss, overhead = [], [], [], []
    for index, sample in enumerate(samples):
        if not isinstance(sample, dict):
            raise RecordError(f"samples[{index}] must be an object")
        t = _number(sample.get("elapsed_ns"), f"samples[{index}].elapsed_ns")
        c = _number(sample.get("cpu_time_ns"), f"samples[{index}].cpu_time_ns")
        r = _number(sample.get("rss_bytes"), f"samples[{index}].rss_bytes")
        d = _number(sample.get("ps_duration_ns"), f"samples[{index}].ps_duration_ns")
        if t < 0 or c < 0 or r < 0 or d < 0:
            raise RecordError(f"samples[{index}] elapsed time, CPU, RSS, and sampling duration must be nonnegative")
        if elapsed and t <= elapsed[-1]:
            raise RecordError("sample elapsed_ns values must be strictly increasing")
        if cpu and c < cpu[-1]:
            raise RecordError("sample cpu_time_ns values must be nondecreasing")
        elapsed.append(t); cpu.append(c); rss.append(r); overhead.append(d / 1_000_000)
    if len(samples) < 2 or elapsed[-1] <= elapsed[0]:
        raise RecordError("at least two samples with positive observed elapsed duration are required")
    return elapsed, cpu, rss, overhead


def _percentile(values, percentile):
    ordered = sorted(values)
    rank = max(1, math.ceil(percentile * len(ordered)))
    return ordered[rank - 1]


def summarize_record(record):
    elapsed, cpu, rss, overhead = validate_record(record)
    duration_ns = elapsed[-1] - elapsed[0]
    cpu_delta_ns = cpu[-1] - cpu[0]
    return {
        "run_id": record["run_id"],
        "label": record["label"],
        "workload": record["workload"],
        "pid": record["pid"],
        "completion_status": record["completion"]["status"],
        "completion_error": record["completion"]["error"],
        "sample_count": len(elapsed),
        "observed_sample_elapsed_ns": duration_ns,
        "cpu_total_delta_ns": cpu_delta_ns,
        "mean_process_cpu_percent": cpu_delta_ns / duration_ns * 100.0,
        "rss_bytes": {"min": min(rss), "max": max(rss), "median": _median(rss)},
        "sampling_overhead_ms": {
            "median": _median(overhead),
            "p95_nearest_rank": _percentile(overhead, 0.95),
        },
        "cycle_time": "unavailable",
        "frame_time": "unavailable",
        "main_thread_time": "unavailable",
    }


def _median(values):
    ordered = sorted(values)
    middle = len(ordered) // 2
    return ordered[middle] if len(ordered) % 2 else (ordered[middle - 1] + ordered[middle]) / 2


def build_report(records):
    summaries = [summarize_record(record) for record in records]
    return {
        "schema_version": 1,
        "kind": "rustui_process_metrics_summary",
        "purpose": "descriptive external process measurements; no acceptance verdict or performance comparison",
        "runs": summaries,
        "limitations": [
            "Process CPU and RSS do not measure frame rate, frame time, cycle time, or main-thread time.",
            "CPU nanoseconds are converted ps display units; they do not imply nanosecond measurement precision.",
            "RSS is ps resident memory in bytes, not physical footprint or GPU memory.",
            "CPU percentage is process CPU divided by observed sample elapsed time and may exceed 100% on multi-CPU systems.",
            "Runs are reported separately; unlike workloads are not aggregated or compared.",
            "Incomplete source records remain incomplete and are not passing results.",
        ],
    }


def read_record(path):
    try:
        with path.open(encoding="utf-8") as stream:
            return json.load(stream, parse_constant=lambda value: (_ for _ in ()).throw(
                RecordError(f"invalid non-finite JSON number: {value}")
            ))
    except (OSError, json.JSONDecodeError) as error:
        raise RecordError(f"cannot read JSON record {path}: {error}") from error


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("records", nargs="+", type=Path, help="explicit raw collector JSON record(s)")
    parser.add_argument("--output", type=Path, help="exclusive report path under target/headstart/performance; default stdout")
    args = parser.parse_args(argv)
    try:
        records = [read_record(path) for path in args.records]
        report = build_report(records)
        serialized = json.dumps(report, indent=2, allow_nan=False) + "\n"
        if args.output:
            output = validate_output_path(args.output)
            for source in args.records:
                try:
                    if output.resolve() == source.resolve() or (output.exists() and source.exists() and output.samefile(source)):
                        raise RecordError("summary output must not alias a raw input record")
                except OSError as error:
                    raise RecordError(f"cannot check output/input identity: {error}") from error
            write_json_exclusive(output, report)
        else:
            sys.stdout.write(serialized)
    except (RecordError, OSError, ValueError) as error:
        parser.error(str(error))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
