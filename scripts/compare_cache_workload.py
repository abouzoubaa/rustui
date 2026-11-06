#!/usr/bin/env python3
"""Plan CUA launches, attach to existing native probes, and compare paired CPU/RSS.

Never launches a GUI process, invokes Cargo, signals a process, or promotes a patch.
"""

import argparse
from datetime import datetime, timezone
import hashlib
import json
import math
from pathlib import Path
import platform
import re
import statistics
import subprocess
import sys
import time

import collect_runtime_metrics as collector
from calibrate_cache_workload import compiler_identity, json_value_matches
from record_build_provenance import INPUT_FILES, file_record, git_commit, snapshot_digest, source_records
from runtime_metrics import OUTPUT_DIR, validate_output_path, write_json_exclusive

ROOT = Path(__file__).resolve().parent.parent
WORKLOADS = {
    "8-leaf": {"leaves": 8, "lines_per_leaf": 12, "width": 210.0, "height": 220.0,
               "warmup_updates": 10, "measured_updates": 100},
    "32-leaf": {"leaves": 32, "lines_per_leaf": 12, "width": 210.0, "height": 220.0,
                "warmup_updates": 10, "measured_updates": 100},
}
INPUTS = INPUT_FILES
WARMUP_PAIRS = 1
MEASURED_PAIRS = 5
INTERVAL_NS = 100_000_000
DURATION_NS = 45_000_000_000
FINAL_SNAPSHOT_TIMEOUT = 2.0
FINAL_SNAPSHOT_POLL = 0.02


def identity(path):
    path = Path(path).resolve(strict=True)
    if not path.is_file():
        raise ValueError(f"not a regular file: {path}")
    return {"path": str(path), "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            "size_bytes": path.stat().st_size}


def safe_id(value):
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]{0,39}", value):
        raise ValueError("session ID requires 1-40 safe ASCII filename characters")
    return value


def path_for(run_id, suffix):
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]{0,79}", run_id):
        raise ValueError("unsafe run ID")
    return validate_output_path(OUTPUT_DIR / f"cache-comparison-{run_id}-{suffix}.json")


def read_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def schedule(session, diagnostics_overhead=False):
    safe_id(session)
    runs = []
    for pair in range(WARMUP_PAIRS + MEASURED_PAIRS):
        modes = (("diagnostics_on", "diagnostics_off") if pair % 2 == 0 else
                 ("diagnostics_off", "diagnostics_on")) if diagnostics_overhead else (
                     ("cached", "uncached") if pair % 2 == 0 else ("uncached", "cached"))
        for mode in modes:
            index = len(runs)
            runs.append({"index": index, "pair": pair, "phase": "warmup" if pair < WARMUP_PAIRS else "measured",
                         "run_id": f"{session}-{index:02}-{mode}", "mode": mode})
    return runs


def validate_build_provenance(path, binary, compiler, inputs):
    provenance_path = Path(path).resolve(strict=True)
    provenance = read_json(provenance_path)
    if provenance.get("schema_version") != 1:
        raise ValueError("unsupported build provenance schema")
    build_log = provenance.get("build_log")
    if (
        not isinstance(build_log, dict)
        or not isinstance(build_log.get("path"), str)
        or not build_log["path"]
        or not isinstance(build_log.get("sha256"), str)
        or not re.fullmatch(r"[0-9a-f]{64}", build_log["sha256"])
        or type(build_log.get("size_bytes")) is not int
        or build_log["size_bytes"] < 0
        or build_log.get("parsed_for_success") is not False
    ):
        raise ValueError("build provenance has an invalid build log record")
    facts = provenance.get("build_facts_declared_by_caller", {})
    if facts.get("status") != "observed_success" or facts.get("exit_status") != 0:
        raise ValueError("build provenance does not report an observed successful build")
    # Current provenance uses pinned-root-relative paths; older schema-1 records
    # used absolute paths. Bind either spelling to the same current rooted file.
    current_binary = file_record(ROOT, binary["path"])
    selected = provenance.get("selected_binary")
    if (current_binary.get("status") != "captured"
            or current_binary.get("sha256") != binary["sha256"]
            or current_binary.get("size_bytes") != binary["size_bytes"]
            or not isinstance(selected, dict)
            or selected.get("path") not in (current_binary["path"], binary["path"])
            or {key: value for key, value in selected.items() if key != "path"} != {
                "sha256": binary["sha256"], "size_bytes": binary["size_bytes"],
                "status": "captured"}):
        raise ValueError("build provenance binary does not match selected executable")
    if provenance.get("compiler_identity") != compiler:
        raise ValueError("build provenance compiler differs from planned compiler")
    captured = provenance.get("captured_current_workspace", {})
    revision = captured.get("git_commit")
    if not isinstance(revision, str) or not re.fullmatch(r"[0-9a-f]{40,64}", revision):
        raise ValueError("build provenance has no valid source revision")
    if captured.get("source_snapshot_sha256") != snapshot_digest(captured):
        raise ValueError("build provenance source snapshot digest is invalid")
    records = captured.get("input_file_hashes", [])
    if not isinstance(records, list) or any(not isinstance(entry, dict) for entry in records):
        raise ValueError("build provenance input file hashes must be a list of objects")
    paths = [entry.get("path") for entry in records]
    if any(not isinstance(path, str) or not path for path in paths):
        raise ValueError("build provenance input paths must be nonempty strings")
    if len(paths) != len(set(paths)):
        raise ValueError("build provenance input paths must be unique")
    if set(paths) != set(inputs):
        raise ValueError("build provenance input paths do not match expected inputs")
    by_path = {entry["path"]: entry for entry in records}
    for name, expected in inputs.items():
        entry = by_path.get(name)
        if not entry or entry.get("status") != "captured" or entry.get("sha256") != expected["sha256"]:
            raise ValueError(f"build provenance input mismatch: {name}")
    sources = captured.get("workspace_crate_app_file_hashes", [])
    if not sources or any(entry.get("status") != "captured" for entry in sources):
        raise ValueError("build provenance has incomplete workspace source hashes")
    current_sources = source_records(ROOT)
    if current_sources != sources or git_commit(ROOT) != revision:
        raise ValueError("workspace source revision changed after build provenance capture")
    for name in INPUTS:
        if file_record(ROOT, name) != by_path.get(name):
            raise ValueError(f"workspace build input changed after provenance capture: {name}")
    return identity(provenance_path)


def make_plan(session, binary, workload="8-leaf", diagnostics_overhead=False, build_provenance=None):
    if workload not in WORKLOADS:
        raise ValueError(f"unknown workload: {workload}")
    protocol = WORKLOADS[workload]
    runs = schedule(session, diagnostics_overhead)
    comparison = "diagnostics_overhead" if diagnostics_overhead else "cache"
    binary_identity = identity(binary)
    compiler = compiler_identity()
    provenance_identity = None
    if diagnostics_overhead:
        if build_provenance is None:
            raise ValueError("diagnostics-overhead plans require --build-provenance")
        provenance_identity = validate_build_provenance(
            build_provenance, binary_identity, compiler,
            {name: identity(ROOT / name) for name in INPUTS})
    plan = {"schema_version": 1, "kind": "rustui_cache_comparison_plan", "session_id": session,
            "created_at_utc": datetime.now(timezone.utc).isoformat(), "binary": binary_identity,
            "inputs": {name: identity(ROOT / name) for name in INPUTS}, "compiler_at_plan_time": compiler,
            "build_provenance": provenance_identity,
            "machine": {"platform": platform.platform(), "machine": platform.machine()},
            "workload": workload, "protocol": protocol, "comparison": comparison, "runs": runs,
            "collector": {"duration_ns": DURATION_NS, "interval_ns": INTERVAL_NS},
            "provenance_limit": "Build metadata is caller-declared; hashes bind the selected binary, compiler, revision, and inputs but do not independently certify historical compiler execution.",
            "launch_policy": "Each run is a new native process launched only via CUA; this script never launches it."}
    for run in runs:
        runtime_mode = "uncached" if diagnostics_overhead else run["mode"]
        diagnostics_enabled = run["mode"] != "diagnostics_off" if diagnostics_overhead else True
        run["runtime_mode"] = runtime_mode
        run["diagnostics_enabled"] = diagnostics_enabled
        run["bundle_configuration"] = {"run_id": run["run_id"], "mode": runtime_mode,
                                        "diagnostics_enabled": diagnostics_enabled,
                                        "leaves": protocol["leaves"]}
        run["configuration_path"] = str(path_for(run["run_id"], "config"))
        run["launch_argv_for_cua"] = [plan["binary"]["path"], "--config", run["configuration_path"]]
    # Refuse reusing any artifacts even when the session plan was removed.
    for run in runs:
        for suffix in ("config", "ready", "start", "result", "ack", "final", "metrics"):
            if path_for(run["run_id"], suffix).exists():
                raise ValueError(f"run artifacts already exist: {run['run_id']}")
    destination = path_for(session, "plan")
    if destination.exists():
        raise ValueError("session plan already exists")
    write_json_exclusive(destination, plan)
    for run in runs:
        write_json_exclusive(Path(run["configuration_path"]), run["bundle_configuration"])
    return destination


def load_plan(path, verify_inputs=True):
    path = validate_output_path(Path(path))
    plan = read_json(path)
    if plan.get("schema_version") != 1 or plan.get("kind") != "rustui_cache_comparison_plan":
        raise ValueError("unsupported comparison plan")
    comparison = plan.get("comparison", "cache")
    if comparison not in ("cache", "diagnostics_overhead"):
        raise ValueError("unknown comparison kind")
    expected = schedule(plan["session_id"], comparison == "diagnostics_overhead")
    workload = plan.get("workload")
    if workload not in WORKLOADS or plan.get("protocol") != WORKLOADS[workload]:
        raise ValueError("unknown or changed workload protocol")
    if plan.get("collector") != {"duration_ns": DURATION_NS, "interval_ns": INTERVAL_NS}:
        raise ValueError("comparison protocol changed")
    if len(plan.get("runs", [])) != len(expected):
        raise ValueError("comparison schedule length changed")
    for actual, required in zip(plan["runs"], expected):
        if any(actual.get(field) != value for field, value in required.items()):
            raise ValueError("comparison schedule changed")
        runtime_mode = "uncached" if comparison == "diagnostics_overhead" else actual["mode"]
        diagnostics_enabled = actual["mode"] != "diagnostics_off" if comparison == "diagnostics_overhead" else True
        if actual.get("runtime_mode") != runtime_mode or actual.get("diagnostics_enabled") is not diagnostics_enabled:
            raise ValueError("run instrumentation configuration changed")
        if actual.get("bundle_configuration") != {"run_id": actual["run_id"], "mode": runtime_mode,
                                                   "diagnostics_enabled": diagnostics_enabled,
                                                   "leaves": plan["protocol"]["leaves"]}:
            raise ValueError("bundle configuration changed")
        if actual.get("configuration_path") != str(path_for(actual["run_id"], "config")):
            raise ValueError("configuration path changed")
    if verify_inputs:
        if identity(plan["binary"]["path"]) != plan["binary"]:
            raise ValueError("planned binary changed")
        if set(plan.get("inputs", {})) != set(INPUTS):
            raise ValueError("incomplete source provenance")
        for name in INPUTS:
            if identity(ROOT / name) != plan["inputs"][name]:
                raise ValueError(f"input changed after plan: {name}")
    if comparison == "diagnostics_overhead":
        provenance_identity = validate_build_provenance(
            plan.get("build_provenance", {}).get("path", ""), plan["binary"],
            plan.get("compiler_at_plan_time"), plan.get("inputs", {}))
        if provenance_identity != plan["build_provenance"]:
            raise ValueError("build provenance record changed after planning")
    return plan


def check_workload(result, run, pid, protocol):
    if not isinstance(result, dict):
        raise ValueError("workload must be an object")
    expected = {"schema_version": 1, "kind": "rustui_cache_comparison_workload", "run_id": run["run_id"],
                "pid": pid, "mode": run.get("runtime_mode", run["mode"]),
                "diagnostics_enabled": run.get("diagnostics_enabled", True),
                "status": "complete", "ax_active_seen": False,
                "native_drag_seen": False, "environment_scope": "empty", "correctness_passed": True,
                "protocol": protocol, "values": [1] * protocol["leaves"], "mounts": [1] * protocol["leaves"]}
    for field, value in expected.items():
        if not json_value_matches(value, result.get(field)):
            raise ValueError(f"workload {field}: expected {value!r}, got {result.get(field)!r}")
    diagnostic_counts = run.get("diagnostics_enabled", True)
    if not diagnostic_counts:
        required = {"parent_root": 0, "parent_leaf": 0, "final_leaf": 0, "mutation_leaf_delta": 0}
    else:
        runtime_mode = run.get("runtime_mode", run["mode"])
        leaf = 0 if runtime_mode == "cached" else protocol["leaves"] * protocol["measured_updates"]
        required = {"parent_root": protocol["measured_updates"], "parent_leaf": leaf,
                    "final_leaf": leaf + protocol["leaves"], "mutation_leaf_delta": protocol["leaves"]}
    if not json_value_matches(required, result.get("counts")):
        raise ValueError("wrong raw lowering counts")
    diagnostic_records = result.get("diagnostics")
    if not isinstance(diagnostic_records, list):
        raise ValueError("missing raw diagnostic timings")
    if not diagnostic_counts:
        if diagnostic_records:
            raise ValueError("diagnostic-off run recorded timings")
    else:
        expected_types = {"ComparisonRoot", "ComparisonLeaf"}
        hosts = {}
        for entry in diagnostic_records:
            if not isinstance(entry, dict) or not isinstance(entry.get("component_type"), str):
                raise ValueError("invalid per-type diagnostic timing record")
            component_type = entry["component_type"].rsplit("::", 1)[-1]
            if component_type not in expected_types or component_type in hosts:
                raise ValueError("unexpected or duplicate per-type diagnostic timing")
            hosts[component_type] = entry
        if set(hosts) != expected_types:
            raise ValueError("missing per-type diagnostic timings")
        for host in hosts.values():
            if any(type(host.get(key)) is not int or host[key] < 0
                   for key in ("count", "total_ns", "max_ns")) or host["max_ns"] > host["total_ns"]:
                raise ValueError("invalid per-type diagnostic timing")
        counts = result["counts"]
        # The final mutation phase produces one root lowering after parent_root
        # snapshots the measured update loop.
        expected_counts = {
            "ComparisonRoot": counts["parent_root"] + 1,
            "ComparisonLeaf": counts["final_leaf"],
        }
        if any(hosts[name]["count"] != count for name, count in expected_counts.items()):
            raise ValueError("diagnostic timing counts differ from lowering counts")


def metric_values(raw):
    if raw.get("completion", {}).get("status") != "complete" or len(raw.get("samples", [])) < 2:
        raise ValueError("incomplete process capture")
    points = raw["samples"]
    for index, point in enumerate(points):
        for field in ("cpu_time_ns", "rss_bytes", "elapsed_ns", "ps_duration_ns"):
            if type(point.get(field)) is not int or point[field] < 0:
                raise ValueError("invalid raw process sample")
        if index and (point["cpu_time_ns"] < points[index - 1]["cpu_time_ns"] or point["elapsed_ns"] <= points[index - 1]["elapsed_ns"]):
            raise ValueError("process counters moved backwards")
    elapsed = points[-1]["elapsed_ns"] - points[0]["elapsed_ns"]
    if elapsed <= 0:
        raise ValueError("zero sampling interval")
    return {"attached_cpu_ns": points[-1]["cpu_time_ns"] - points[0]["cpu_time_ns"],
            "attached_elapsed_ns": elapsed, "median_sampled_rss_bytes": statistics.median(p["rss_bytes"] for p in points),
            "max_sampled_rss_bytes": max(p["rss_bytes"] for p in points)}


def inspect_process(pid, executable):
    result = subprocess.run(["ps", "-ww", "-p", str(pid), "-o", "comm="],
                            capture_output=True, text=True, timeout=5, check=True)
    if Path(result.stdout.strip()).resolve(strict=True) != executable:
        raise ValueError("attached PID executable does not match ready record")


def collect_run(plan_path, index, pid):
    if platform.system() != "Darwin":
        raise ValueError("this comparison protocol is scoped to macOS")
    if pid <= 0:
        raise ValueError("PID must be positive")
    plan = load_plan(plan_path)
    plan_identity = identity(plan_path)
    if not 0 <= index < len(plan["runs"]):
        raise ValueError("run index outside launch schedule")
    run = plan["runs"][index]
    destination = path_for(run["run_id"], "metrics")
    if destination.exists():
        raise ValueError("capture already exists; preserve it and use a new session")
    for prior in plan["runs"][:index]:
        record = read_json(path_for(prior["run_id"], "metrics"))
        if record["status"] != "complete":
            raise ValueError("prior run failed or was excluded; start a new session")
        if record["pid"] == pid:
            raise ValueError("each run requires a fresh PID")
    ready_path = path_for(run["run_id"], "ready")
    ready = read_json(ready_path)
    for field, value in {"schema_version": 1, "kind": "rustui_cache_comparison_ready",
                         "run_id": run["run_id"], "mode": run["runtime_mode"],
                         "diagnostics_enabled": run["diagnostics_enabled"], "pid": pid}.items():
        if ready.get(field) != value:
            raise ValueError(f"ready record mismatch: {field}")
    executable = Path(ready["executable"]).resolve(strict=True)
    executable_identity = identity(executable)
    if executable_identity["sha256"] != plan["binary"]["sha256"]:
        raise ValueError("launched binary differs from plan")
    inspect_process(pid, executable)
    started, result_seen = False, False
    gate = {"run_id": run["run_id"], "mode": run["runtime_mode"], "pid": pid}

    def stop_requested():
        nonlocal started, result_seen
        result_exists = path_for(run["run_id"], "result").exists()
        if not started and not result_exists:
            write_json_exclusive(path_for(run["run_id"], "start"), gate)
            started = True
        if result_exists:
            # Capture another ps point after the completed result was observed.
            if result_seen:
                return True
            result_seen = True
        return False

    raw = collector.collect(pid, DURATION_NS, INTERVAL_NS, "P3D cache comparison", "retained subtree",
                            run["run_id"], provenance={"plan": plan_identity, "ready": identity(ready_path)},
                            stop_requested=stop_requested)
    record = {"schema_version": 1, "kind": "rustui_cache_comparison_capture", "run_id": run["run_id"],
                "pid": pid, "mode": run["mode"], "runtime_mode": run["runtime_mode"],
                "diagnostics_enabled": run["diagnostics_enabled"], "phase": run["phase"], "pair": run["pair"],
              "status": "failed", "error": None, "plan": plan_identity, "ready": ready,
              "executable": executable_identity, "raw_process_metrics": raw,
              "measurement_scope": "attached interval including warmup, updates, mutation, and final collection wait; excludes startup/exit; RSS is sampled, not OS peak"}
    try:
        result_path = path_for(run["run_id"], "result")
        result = read_json(result_path)
        record["workload"] = result
        record["workload_artifact"] = identity(result_path)
        record["metrics"] = metric_values(raw)
        check_workload(result, run, pid, plan["protocol"])
        if identity(executable) != executable_identity:
            raise ValueError("launched executable changed during capture")
        load_plan(plan_path)
        if identity(plan_path) != plan_identity:
            raise ValueError("plan changed during capture")
        record["status"] = "complete"
    except (OSError, ValueError, KeyError, TypeError) as error:
        record["error"] = f"{type(error).__name__}: {error}"
        if record.get("workload", {}).get("status", "").startswith("excluded"):
            record["status"] = "excluded"
    # Acknowledge completion; the app samples guards and writes its final snapshot.
    write_json_exclusive(path_for(run["run_id"], "ack"), gate)
    final_path = path_for(run["run_id"], "final")
    deadline = time.monotonic() + FINAL_SNAPSHOT_TIMEOUT
    while not final_path.exists() and time.monotonic() < deadline:
        time.sleep(min(FINAL_SNAPSHOT_POLL, max(0, deadline - time.monotonic())))
    try:
        final = read_json(final_path)
        record["final_workload"] = final
        record["final_workload_artifact"] = identity(final_path)
        for field, value in {"schema_version": 1, "kind": "rustui_cache_comparison_final",
                             "run_id": run["run_id"], "mode": run["runtime_mode"],
                             "diagnostics_enabled": run["diagnostics_enabled"], "pid": pid}.items():
            if final.get(field) != value:
                raise ValueError(f"final snapshot mismatch: {field}")
        if final.get("workload") != record.get("workload"):
            raise ValueError("final snapshot workload differs from initial result")
        ax_seen, drag_seen = final.get("ax_active_seen"), final.get("native_drag_seen")
        if type(ax_seen) is not bool or type(drag_seen) is not bool:
            raise ValueError("invalid final guard flags")
        if ax_seen or drag_seen:
            record["status"] = "excluded"
            record["error"] = "ValueError: interaction guard observed after initial result"
        elif final.get("status") != record.get("workload", {}).get("status"):
            raise ValueError("final snapshot status differs from workload status")
    except (OSError, ValueError, KeyError, TypeError) as error:
        record["status"] = "failed"
        record["error"] = f"{type(error).__name__}: {error}"
    write_json_exclusive(destination, record)
    return destination, record["status"]


def spread(values):
    if not values or any(not math.isfinite(value) or value < 0 for value in values):
        raise ValueError("invalid summary values")
    middle = statistics.median(values)
    return {"median": middle, "min": min(values), "max": max(values), "range": max(values) - min(values),
            "range_percent_of_median": (max(values) - min(values)) / middle * 100 if middle else None}


def summarize(plan_path):
    plan = load_plan(plan_path)
    records, artifacts, pids, starts = [], [], set(), set()
    plan_identity = identity(plan_path)
    for run in plan["runs"]:
        path = path_for(run["run_id"], "metrics")
        record = read_json(path)
        if record.get("status") != "complete" or record.get("plan") != plan_identity:
            raise ValueError("missing, failed, excluded, or mismatched run")
        for field in ("run_id", "mode", "runtime_mode", "diagnostics_enabled", "phase", "pair"):
            if record.get(field) != run[field]:
                raise ValueError(f"capture schedule mismatch: {field}")
        pid = record["pid"]
        start = record["raw_process_metrics"]["identity"]
        start_key = (pid, start["lstart"])
        if pid in pids or start_key in starts or start["pid"] != pid:
            raise ValueError("runs did not use distinct matching process identities")
        pids.add(pid)
        starts.add(start_key)
        if record["executable"]["sha256"] != plan["binary"]["sha256"]:
            raise ValueError("mixed binaries")
        if record["raw_process_metrics"].get("pid") != pid or record["raw_process_metrics"].get("run_id") != run["run_id"]:
            raise ValueError("raw process capture identity mismatch")
        if record["raw_process_metrics"].get("provenance", {}).get("plan") != plan_identity:
            raise ValueError("raw process capture plan mismatch")
        if identity(record["workload_artifact"]["path"]) != record["workload_artifact"]:
            raise ValueError("workload artifact changed")
        if read_json(record["workload_artifact"]["path"]) != record["workload"]:
            raise ValueError("embedded workload differs from artifact")
        if identity(record["final_workload_artifact"]["path"]) != record["final_workload_artifact"]:
            raise ValueError("final artifact changed")
        final = read_json(record["final_workload_artifact"]["path"])
        if final != record["final_workload"] or final.get("workload") != record["workload"]:
            raise ValueError("embedded final workload differs from artifact")
        for field, value in {"schema_version": 1, "kind": "rustui_cache_comparison_final",
                         "run_id": run["run_id"], "mode": run["runtime_mode"], "pid": pid,
                         "diagnostics_enabled": run["diagnostics_enabled"],
                             "status": "complete", "ax_active_seen": False,
                             "native_drag_seen": False}.items():
            if final.get(field) != value:
                raise ValueError(f"final snapshot mismatch: {field}")
        check_workload(record["workload"], run, pid, plan["protocol"])
        if metric_values(record["raw_process_metrics"]) != record["metrics"]:
            raise ValueError("derived metrics do not match raw samples")
        records.append(record)
        artifacts.append(identity(path))
        artifacts.append(record["final_workload_artifact"])
    paired = []
    fields = ("attached_cpu_ns", "attached_elapsed_ns", "median_sampled_rss_bytes", "max_sampled_rss_bytes")
    comparison = plan["comparison"]
    left, right = (("diagnostics_on", "diagnostics_off") if comparison == "diagnostics_overhead"
                   else ("cached", "uncached"))
    ratio_name = "diagnostics_on_over_off" if comparison == "diagnostics_overhead" else "cached_over_uncached"
    for pair in range(WARMUP_PAIRS, WARMUP_PAIRS + MEASURED_PAIRS):
        selected = {record["mode"]: record for record in records if record["pair"] == pair}
        ratios = {field: selected[left]["metrics"][field] / selected[right]["metrics"][field]
                  if selected[right]["metrics"][field] else None for field in fields}
        paired.append({"pair": pair, ratio_name: ratios})
    labels = ("diagnostics_on", "diagnostics_off") if comparison == "diagnostics_overhead" else ("cached", "uncached")
    diagnostic_timings = {}
    for mode in labels:
        samples_by_type = {}
        for record in records:
            if record["phase"] != "measured" or record["mode"] != mode:
                continue
            for host in record["workload"]["diagnostics"]:
                samples_by_type.setdefault(host["component_type"], []).append(host)
        diagnostic_timings[mode] = {
            component_type: {field: spread([sample[field] for sample in samples])
                             for field in ("count", "total_ns", "max_ns")}
            for component_type, samples in samples_by_type.items()
        }
    summary = {"schema_version": 1, "kind": "rustui_cache_comparison_summary", "status": "complete",
               "plan": plan_identity, "workload": plan["workload"], "protocol": plan["protocol"],
               "comparison": comparison,
               "capture_artifacts": artifacts, "measured_pairs": MEASURED_PAIRS,
               "modes": {mode: {field: spread([r["metrics"][field] for r in records
                                                 if r["phase"] == "measured" and r["mode"] == mode]) for field in fields}
                         for mode in labels},
               "diagnostic_timings": diagnostic_timings,
               "paired_ratios": paired,
               "ratio_spread": {field: spread([p[ratio_name][field] for p in paired])
                                if all(p[ratio_name][field] is not None for p in paired)
                                else {"status": "unavailable_zero_control"} for field in fields},
               "limits": ("No acceptance thresholds. Diagnostics calibration pairs use identical uncached workload and differ only in diagnostic recording; process CPU includes normal workload and collector overhead. Elapsed is collector attachment interval, not frame timing. Sampled RSS is not OS peak."
                          if comparison == "diagnostics_overhead" else
                          "No acceptance thresholds or automatic speedup verdict; elapsed is collector attachment interval, not frame timing. CPU includes instrumentation; sampled RSS is not OS peak. Native visual acceptance remains separate.")}
    destination = path_for(plan["session_id"], "summary")
    write_json_exclusive(destination, summary)
    return destination


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    create = commands.add_parser("plan", help="write immutable inputs and a CUA launch schedule")
    create.add_argument("--session", required=True)
    create.add_argument("--binary", type=Path, required=True)
    create.add_argument("--workload", choices=WORKLOADS, default="8-leaf")
    create.add_argument("--diagnostics-overhead", action="store_true",
                        help="pair diagnostics off/on with identical uncached workload")
    create.add_argument("--build-provenance", type=Path,
                        help="required build record for diagnostics-overhead plans")
    capture = commands.add_parser("collect", help="attach to an already CUA-launched PID")
    capture.add_argument("--plan", type=Path, required=True)
    capture.add_argument("--run-index", type=int, required=True)
    capture.add_argument("--pid", type=int, required=True)
    report = commands.add_parser("summarize")
    report.add_argument("--plan", type=Path, required=True)
    args = parser.parse_args(argv)
    try:
        if args.command == "plan":
            print(make_plan(args.session, args.binary, args.workload, args.diagnostics_overhead,
                            args.build_provenance))
        elif args.command == "collect":
            destination, status = collect_run(args.plan, args.run_index, args.pid)
            print(destination)
            return 0 if status == "complete" else 1
        else:
            print(summarize(args.plan))
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
        print(f"cache comparison: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
