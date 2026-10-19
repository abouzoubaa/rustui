#!/usr/bin/env python3
"""Record external process CPU time and RSS without launching or changing the process."""

import argparse
from datetime import datetime, timezone
from decimal import Decimal, InvalidOperation
import hashlib
import os
from pathlib import Path
import platform
import re
import subprocess
import sys
import time
from uuid import uuid4

from runtime_metrics import SCHEMA_VERSION, default_output_path, validate_output_path, write_json_exclusive


PS_FIELDS = ("pid=", "lstart=", "time=", "rss=")
LSTART = r"(?P<lstart>[A-Za-z]{3}\s+[A-Za-z]{3}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2}\s+\d{4})"
PS_ROW = re.compile(rf"^\s*(?P<pid>\d+)\s+{LSTART}\s+(?P<cpu>\S+)\s+(?P<rss>\d+)\s*$")
MONTHS = {name: index for index, name in enumerate(
    ("Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"), 1
)}
WEEKDAYS = ("Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun")


def positive_ns(value: str) -> int:
    try:
        number = Decimal(value)
        if not number.is_finite() or number <= 0:
            raise argparse.ArgumentTypeError("must be a positive finite number of seconds")
        nanoseconds = number * 1_000_000_000
    except InvalidOperation as error:
        raise argparse.ArgumentTypeError("must be a positive finite number of seconds") from error
    if nanoseconds != nanoseconds.to_integral_value():
        raise argparse.ArgumentTypeError("must be positive, finite, and representable in nanoseconds")
    return int(nanoseconds)


def parse_cpu_time(value: str) -> int:
    """Parse ps cumulative CPU time exactly to nanoseconds."""
    if not re.fullmatch(r"(?:\d+-)?\d+(?::\d+){1,2}(?:\.\d+)?", value):
        raise ValueError(f"invalid ps CPU time: {value!r}")
    days = 0
    has_days = "-" in value
    if has_days:
        day_part, value = value.split("-", 1)
        days = int(day_part)
    parts = value.split(":")
    if has_days and len(parts) != 3:
        raise ValueError("day-qualified ps CPU time needs hours, minutes, and seconds")
    if len(parts) == 2:
        hours, minutes, seconds = 0, int(parts[0]), Decimal(parts[1])
    else:
        hours, minutes, seconds = int(parts[0]), int(parts[1]), Decimal(parts[2])
        if minutes >= 60:
            raise ValueError("ps CPU minutes exceed 59")
    if seconds >= 60 or (has_days and hours >= 24):
        raise ValueError("ps CPU time component out of range")
    ns = (Decimal(((days * 24 + hours) * 60 + minutes) * 60) + seconds) * 1_000_000_000
    if ns != ns.to_integral_value():
        raise ValueError("ps CPU time has subnanosecond precision")
    return int(ns)


def parse_ps_row(output: str, expected_pid: int) -> tuple[str, int, int]:
    if len(output.splitlines()) != 1:
        raise ValueError("ps returned no row or multiple rows")
    match = PS_ROW.fullmatch(output.strip())
    if match is None or int(match["pid"]) != expected_pid:
        raise ValueError("ps returned malformed data or a different PID")
    parts = match["lstart"].split()
    if parts[0] not in WEEKDAYS or parts[1] not in MONTHS:
        raise ValueError("ps returned invalid process start date")
    hour, minute, second = (int(part) for part in parts[3].split(":"))
    started = datetime(int(parts[4]), MONTHS[parts[1]], int(parts[2]), hour, minute, second)
    if WEEKDAYS[started.weekday()] != parts[0]:
        raise ValueError("ps returned inconsistent process start weekday")
    return " ".join(parts), parse_cpu_time(match["cpu"]), int(match["rss"]) * 1024


def sample(pid: int, start_ns: int, timeout: float = 5.0) -> tuple[str, dict]:
    argv = ["ps", "-p", str(pid), *(part for field in PS_FIELDS for part in ("-o", field))]
    before = time.monotonic_ns()
    result = subprocess.run(argv, capture_output=True, text=True, timeout=timeout,
                            env={**os.environ, "LC_ALL": "C", "LANG": "C"}, check=False)
    after = time.monotonic_ns()
    if result.returncode != 0:
        raise ValueError(f"ps exited {result.returncode}: {result.stderr.strip()[:200]}")
    identity, cpu_ns, rss_bytes = parse_ps_row(result.stdout, pid)
    return identity, {
        "elapsed_ns": (before + after) // 2 - start_ns,
        "ps_duration_ns": after - before,
        "cpu_time_ns": cpu_ns,
        "rss_bytes": rss_bytes,
    }


def collect(pid: int, duration_ns: int, interval_ns: int, label: str, workload: str,
            run_id: str, provenance: dict | None = None, stop_requested=None) -> dict:
    argv = ["ps", "-p", str(pid), *(part for field in PS_FIELDS for part in ("-o", field))]
    document = {
        "schema_version": SCHEMA_VERSION,
        "kind": "rustui_process_metrics",
        "run_id": run_id,
        "label": label,
        "workload": workload,
        "pid": pid,
        "identity": None,
        "system_context": {"platform": platform.platform(), "machine": platform.machine(),
                           "python": platform.python_version(), "ps_command": argv},
        "config": {"duration_ns": duration_ns, "interval_ns": interval_ns},
        "provenance": provenance,
        "metrics_availability": {"cycle_time": "unavailable", "frame_time": "unavailable",
                                 "main_thread_time": "unavailable"},
        "samples": [],
        "completion": {"status": "incomplete", "error": None},
    }
    start_ns = time.monotonic_ns()
    due_ns = 0
    try:
        while True:
            now = time.monotonic_ns()
            if now < start_ns + due_ns:
                time.sleep((start_ns + due_ns - now) / 1_000_000_000)
            identity, point = sample(pid, start_ns)
            if document["identity"] is None:
                document["identity"] = {"pid": pid, "lstart": identity}
            elif identity != document["identity"]["lstart"]:
                raise ValueError("PID process start identity changed during collection")
            if document["samples"] and point["cpu_time_ns"] < document["samples"][-1]["cpu_time_ns"]:
                raise ValueError("ps cumulative CPU time decreased during collection")
            document["samples"].append(point)
            elapsed = time.monotonic_ns() - start_ns
            if due_ns == duration_ns or elapsed >= duration_ns or (stop_requested is not None and stop_requested()):
                break
            # Skip missed ticks so a slow ps call cannot trigger a sampling storm.
            next_tick = max(due_ns // interval_ns + 1, elapsed // interval_ns + 1) * interval_ns
            due_ns = min(next_tick, duration_ns)
        if len(document["samples"]) < 2:
            raise ValueError("fewer than two samples captured within the requested duration")
        document["completion"] = {"status": "complete", "error": None}
    except (OSError, ValueError, OverflowError, UnicodeError, subprocess.TimeoutExpired) as error:
        document["completion"]["error"] = f"{type(error).__name__}: {error}"
    return document


def run_id_arg(value: str) -> str:
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]{0,79}", value):
        raise argparse.ArgumentTypeError("run ID must be 1-80 safe ASCII filename characters")
    return value


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pid", type=int, required=True)
    parser.add_argument("--duration", type=positive_ns, required=True, metavar="SECONDS")
    parser.add_argument("--interval", type=positive_ns, required=True, metavar="SECONDS")
    parser.add_argument("--label", required=True)
    parser.add_argument("--workload", required=True)
    parser.add_argument("--run-id", type=run_id_arg,
                        default=datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S.%fZ") + "-" + uuid4().hex[:8])
    parser.add_argument("--output", type=Path)
    parser.add_argument("--provenance", type=Path, help="existing build provenance JSON to reference by SHA-256")
    args = parser.parse_args(argv)
    if args.pid <= 0:
        parser.error("--pid must be positive")
    if args.interval > args.duration:
        parser.error("--interval must not exceed --duration")
    if not args.label or not args.workload:
        parser.error("--label and --workload must be nonempty")
    try:
        destination = validate_output_path(args.output or default_output_path(args.run_id, "process"))
        provenance = None
        if args.provenance is not None:
            source = args.provenance.expanduser().resolve(strict=True)
            if not source.is_file():
                raise ValueError("--provenance must name an existing regular file")
            provenance = {"path": str(source), "sha256": hashlib.sha256(source.read_bytes()).hexdigest()}
        document = collect(args.pid, args.duration, args.interval, args.label, args.workload,
                           args.run_id, provenance)
        write_json_exclusive(destination, document)
    except (OSError, ValueError) as error:
        parser.exit(2, f"collector: {error}\n")
    print(destination)
    if document["completion"]["status"] != "complete":
        print(document["completion"]["error"], file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
