#!/usr/bin/env python3
"""Read-only integrity and measurement validator for the M276 Quad V2 run."""
from __future__ import annotations

import argparse
import errno
import hashlib
import json
import math
import os
from pathlib import Path, PurePosixPath
import re
import statistics
import stat
import sys
from typing import Any


CANDIDATE_ID = "quad-stable-sort-v2-20261020-a"
RUN_ID = "quad-v2-release-20261020-b"
PROTOCOL = "reset_copy_finish_batch_v1"
KINDS = ("dense", "reverse", "ordered", "duplicates", "sparse_max")
LENGTHS = (16, 1023, 1024, 1025, 16384, 131071, 131072, 131073)
SCENE = "path-sources/gpui-pre-0.3.9/src/scene.rs"
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
MAX_JSON_BYTES = 32 * 1024 * 1024


class ValidationError(ValueError):
    pass


def _unique_object(items: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in items:
        if key in result:
            raise ValidationError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def _reject_constant(value: str) -> None:
    raise ValidationError(f"nonstandard JSON constant: {value}")


def _json(raw: bytes, label: str) -> Any:
    try:
        return json.loads(raw.decode("utf-8"), object_pairs_hook=_unique_object,
                          parse_constant=_reject_constant)
    except (UnicodeError, json.JSONDecodeError, ValidationError) as error:
        raise ValidationError(f"{label}: invalid JSON ({error})") from error


def _relative(root: Path, value: str, label: str, *, absolute: bool) -> Path:
    if not isinstance(value, str) or not value or "\x00" in value:
        raise ValidationError(f"{label}: path must be a nonempty string")
    lexical = Path(value)
    if absolute:
        if not lexical.is_absolute():
            raise ValidationError(f"{label}: path must be absolute")
        try:
            relative = lexical.relative_to(root)
        except ValueError as error:
            raise ValidationError(f"{label}: path is outside candidate root") from error
    else:
        if lexical.is_absolute():
            try:
                relative = lexical.relative_to(root)
            except ValueError as error:
                raise ValidationError(f"{label}: path is outside candidate root") from error
        else:
            relative = lexical
    if not relative.parts or any(part in ("", ".", "..") for part in relative.parts):
        raise ValidationError(f"{label}: unsafe relative path")
    if not absolute and PurePosixPath(value.replace("\\", "/")).is_absolute():
        raise ValidationError(f"{label}: POSIX absolute path is not allowed here")
    return relative


def _open_read(root: Path, relative: Path, label: str) -> bytes:
    if (not hasattr(os, "O_NOFOLLOW") or os.open not in os.supports_dir_fd
            or os.stat not in os.supports_dir_fd or os.stat not in os.supports_follow_symlinks):
        raise ValidationError("platform lacks descriptor-relative O_NOFOLLOW support")
    directory_flags = (os.O_RDONLY | getattr(os, "O_DIRECTORY", 0) | os.O_NOFOLLOW)
    file_flags = os.O_RDONLY | getattr(os, "O_NONBLOCK", 0) | os.O_NOFOLLOW
    current_fd: int | None = None
    file_fd: int | None = None
    try:
        current_fd = os.open(root.anchor, directory_flags)
        for part in (*root.parts[1:], *relative.parts[:-1]):
            child = os.open(part, directory_flags, dir_fd=current_fd)
            os.close(current_fd)
            current_fd = child
            if not stat.S_ISDIR(os.fstat(current_fd).st_mode):
                raise ValidationError(f"{label}: non-directory path component")
        file_fd = os.open(relative.parts[-1], file_flags, dir_fd=current_fd)
        info = os.fstat(file_fd)
        if not stat.S_ISREG(info.st_mode):
            raise ValidationError(f"{label}: not a regular file")
        if info.st_size > MAX_JSON_BYTES:
            raise ValidationError(f"{label}: exceeds {MAX_JSON_BYTES}-byte limit")
        with os.fdopen(file_fd, "rb") as stream:
            file_fd = None
            raw = stream.read(MAX_JSON_BYTES + 1)
        if len(raw) > MAX_JSON_BYTES:
            raise ValidationError(f"{label}: exceeds {MAX_JSON_BYTES}-byte limit")
        return raw
    except ValidationError:
        raise
    except (OSError, RuntimeError) as error:
        if error.errno == errno.ELOOP:
            raise ValidationError(f"{label}: symlink rejected") from error
        raise ValidationError(f"{label}: cannot read beneath candidate root ({error})") from error
    finally:
        if file_fd is not None:
            os.close(file_fd)
        if current_fd is not None:
            os.close(current_fd)


def _read_path(root: Path, value: str, label: str, *, absolute: bool = True) -> tuple[Path, bytes]:
    relative = _relative(root, value, label, absolute=absolute)
    return relative, _open_read(root, relative, label)


def _digest(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def _verify_ref(root: Path, ref: Any, label: str, *, absolute: bool = True) -> tuple[Path, bytes]:
    if not isinstance(ref, dict) or set(ref) != {"bytes", "path", "sha256"}:
        raise ValidationError(f"{label}: expected bytes/path/sha256 record")
    expected = ref.get("sha256")
    if not isinstance(expected, str) or not SHA256_RE.fullmatch(expected):
        raise ValidationError(f"{label}: invalid SHA-256")
    relative, raw = _read_path(root, ref.get("path"), label, absolute=absolute)
    if type(ref.get("bytes")) is not int or ref["bytes"] != len(raw):
        raise ValidationError(f"{label}: recorded byte length differs")
    if _digest(raw) != expected:
        raise ValidationError(f"{label}: SHA-256 mismatch")
    return relative, raw


def _expect(condition: bool, message: str) -> None:
    if not condition:
        raise ValidationError(message)


def _same_number(actual: Any, expected: float, label: str) -> None:
    try:
        finite = type(actual) in (int, float) and math.isfinite(actual)
    except OverflowError:
        finite = False
    _expect(finite, f"{label}: non-finite or out-of-range number")
    _expect(math.isclose(actual, expected, rel_tol=1e-12, abs_tol=1e-12),
            f"{label}: value does not match raw measurements")


def _metric(case: dict[str, Any], field: str) -> float:
    return statistics.median(
        statistics.mean(sample[field] for sample in trial["samples"])
        for trial in case["trials"]
    )


def _check_raw_run(raw: bytes, label: str, expected_fnv: str) -> dict[tuple[str, int], dict[str, Any]]:
    data = _json(raw, label)
    _expect(isinstance(data, dict) and type(data.get("schema_version")) is int
            and data["schema_version"] == 2,
            f"{label}: unsupported raw schema")
    for key, expected in (("kind", "scene_finish_cpu_microbenchmark"),
                          ("protocol", PROTOCOL), ("profile", "release"),
                          ("debug_assertions", False), ("probe_source_fnv1a64", expected_fnv),
                          ("timing_method", "reset_copy_plus_finish_batch"),
                          ("reset_timing_method", "reset_copy_only_batch"),
                          ("iterations_policy", "max(1,ceil(8192/len))"),
                          ("seed", "0x51554144"), ("warmup", 5), ("trials", 9)):
        _expect(data.get(key) == expected and type(data.get(key)) is type(expected),
                f"{label}: {key} mismatch")
    audit = data.get("mixed_batch_audit")
    _expect(isinstance(audit, dict) and audit.get("passed") is True
            and type(audit.get("batches")) is int and audit["batches"] >= 0
            and type(audit.get("oracle_batches")) is int and audit["oracle_batches"] >= 0
            and audit.get("batches") == audit.get("oracle_batches"),
            f"{label}: mixed-batch oracle failed")
    cases: dict[tuple[str, int], dict[str, Any]] = {}
    expected_keys = {(kind, length) for kind in KINDS for length in LENGTHS}
    for case in data.get("cases", []):
        _expect(isinstance(case, dict), f"{label}: malformed case")
        key = (case.get("kind"), case.get("len"))
        _expect(key in expected_keys and key not in cases, f"{label}: duplicate or unknown case")
        length = key[1]
        iterations = max(1, (8192 + length - 1) // length)
        _expect(case.get("correctness") == "stable_all_semantic_fields"
                and case.get("output_hash_fnv1a64") == case.get("oracle_hash_fnv1a64"),
                f"{label}: case correctness oracle mismatch")
        _expect(type(case.get("iterations_per_sample")) is int
                and case["iterations_per_sample"] == iterations,
                f"{label}: iteration count mismatch")
        cold = case.get("cold")
        _expect(isinstance(cold, dict) and type(cold.get("iterations")) is int
                and cold["iterations"] == iterations
                and type(cold.get("batch_ns")) is int and cold["batch_ns"] > 0
                and type(cold.get("reset_batch_ns")) is int and cold["reset_batch_ns"] > 0,
                f"{label}: invalid cold timing")
        trials = case.get("trials")
        _expect(isinstance(trials, list) and len(trials) == 9, f"{label}: expected nine trials")
        sample_count = 3 if length >= 131072 else 20
        for trial_index, trial in enumerate(trials):
            _expect(isinstance(trial, dict) and type(trial.get("trial")) is int
                    and trial["trial"] == trial_index, f"{label}: invalid trial ordering")
            samples = trial.get("samples")
            _expect(isinstance(samples, list) and len(samples) == sample_count,
                    f"{label}: invalid sample count")
            for sample in samples:
                _expect(isinstance(sample, dict), f"{label}: malformed sample")
                _expect(type(sample.get("iterations")) is int and sample["iterations"] == iterations,
                        f"{label}: sample iteration count mismatch")
                for field in ("batch_ns", "reset_batch_ns"):
                    _expect(type(sample.get(field)) is int and sample[field] > 0,
                            f"{label}: {field} must be a positive integer")
                for field in ("allocation_calls_total", "allocated_bytes_total", "freed_bytes_total"):
                    _expect(type(sample.get(field)) is int and sample[field] >= 0,
                            f"{label}: {field} must be a nonnegative integer")
                _same_number(sample.get("ns_per_iteration"),
                             sample["batch_ns"] / iterations, f"{label}: ns_per_iteration")
                _same_number(sample.get("reset_ns_per_iteration"),
                             sample["reset_batch_ns"] / iterations, f"{label}: reset_ns_per_iteration")
        cases[key] = case
    _expect(set(cases) == expected_keys, f"{label}: incomplete case matrix")
    return cases


def validate(root_arg: Path, summary_arg: Path) -> dict[str, Any]:
    errors: list[str] = []
    report: dict[str, Any] = {
        "schema_version": 1,
        "status": "invalid",
        "measurement_status": "invalid",
        "acceptance_status": "not_assessed",
        "performance_gate": "not_run",
        "promotion_gate": "not_run",
        "scope": "M276 local provenance and timing integrity only; no performance acceptance decision",
        "errors": errors,
    }
    try:
        root_lexical = Path(os.path.abspath(root_arg))
        root = root_lexical.resolve(strict=True)
        _expect(root.is_dir(), "candidate root must be a directory")
        _expect(root.name == CANDIDATE_ID, "candidate root name does not match expected candidate")
        summary_relative = _relative(root_lexical, str(summary_arg), "summary",
                                     absolute=summary_arg.is_absolute())
        summary_raw = _open_read(root, summary_relative, "summary")
        summary = _json(summary_raw, "summary")
        _expect(isinstance(summary, dict), "summary must be a JSON object")
        _expect(type(summary.get("schema_version")) is int and summary.get("schema_version") == 1 and summary.get("milestone") == "M276"
                and summary.get("stage") == 10 and summary.get("workstream") == "P3",
                "summary milestone identity mismatch")
        _expect(summary.get("candidate_id") == CANDIDATE_ID and root.name == summary.get("candidate_id"),
                "summary candidate identity mismatch")
        _expect(summary.get("run_id") == RUN_ID and summary.get("status") == "completed_measurement_no_performance_acceptance",
                "summary run identity or status mismatch")
        _expect(summary.get("protocol") == PROTOCOL and summary.get("profile") == "release",
                "summary protocol/profile mismatch")
        _expect(summary.get("acceptance_thresholds", {}).get("defined") is False,
                "summary must not claim performance thresholds are defined")

        manifest_rel, manifest_raw = _read_path(root, "manifest.json", "candidate manifest", absolute=False)
        manifest = _json(manifest_raw, "candidate manifest")
        manifest_sha = _digest(manifest_raw)
        _expect(manifest.get("candidate_id") == CANDIDATE_ID, "manifest candidate identity mismatch")
        _expect(summary.get("source_manifest_sha256") == manifest_sha,
                "summary manifest digest does not match candidate manifest")
        _expect(isinstance(manifest.get("snapshots"), dict), "manifest snapshots missing")
        scene_hashes: dict[str, str] = {}
        for side in ("baseline", "candidate"):
            snapshot = manifest["snapshots"].get(side)
            _expect(isinstance(snapshot, dict), f"manifest {side} snapshot missing")
            source_rel = Path(side) / "source"
            _expect(snapshot.get("source") == str(root / source_rel),
                    f"manifest {side} source path does not match candidate root")
            expected_scene = manifest.get("preimage_sha256") if side == "baseline" else manifest.get("postimage_sha256")
            _expect(snapshot.get("files", {}).get(SCENE) == expected_scene,
                    f"manifest {side} scene digest mismatch")
            _, scene_raw = _read_path(root, str(source_rel / SCENE), f"{side} scene", absolute=False)
            scene_hash = _digest(scene_raw)
            _expect(scene_hash == expected_scene, f"{side} scene content digest mismatch")
            scene_hashes[side] = scene_hash

        records = summary.get("records")
        _expect(isinstance(records, list) and len(records) == 5, "summary must list three data records and two scripts")
        record_bytes: dict[str, bytes] = {}
        record_paths: dict[str, str] = {}
        for ref in records:
            relative, raw = _verify_ref(root, ref, "summary record")
            name = relative.name
            if name.endswith("-run-index.json"):
                key = "run_index"
            elif name.endswith("-provenance-verification.json"):
                key = "provenance"
            elif name.endswith("-comparison.json"):
                key = "comparison"
            else:
                key = name
            _expect(key not in record_bytes, f"duplicate summary record: {key}")
            record_bytes[key] = raw
            record_paths[key] = str(root / relative)
        _expect({"run_index", "provenance", "comparison"}.issubset(record_bytes),
                "summary run-index, provenance, or comparison record missing")
        for key in ("run_index", "provenance", "comparison"):
            _expect(Path(record_paths[key]).parent == Path(record_paths["run_index"]).parent,
                    "M276 data records must share one evidence directory")
        run_index = _json(record_bytes["run_index"], "run index")
        provenance = _json(record_bytes["provenance"], "provenance verification")
        comparison = _json(record_bytes["comparison"], "comparison")
        run_index_sha = _digest(record_bytes["run_index"])
        provenance_sha = _digest(record_bytes["provenance"])
        comparison_sha = _digest(record_bytes["comparison"])

        _expect(type(run_index.get("schema_version")) is int and run_index.get("schema_version") == 1 and run_index.get("status") == "complete"
                and run_index.get("run_id") == RUN_ID and run_index.get("candidate_id") == CANDIDATE_ID
                and run_index.get("manifest_sha256") == manifest_sha and run_index.get("protocol") == PROTOCOL
                and run_index.get("profile") == "release", "run-index identity mismatch")
        _expect(run_index.get("source_sha256") == summary.get("probe_source_sha256"),
                "run-index probe source digest mismatch")

        source_paths = {}
        source_bytes = {}
        for side in ("baseline", "candidate"):
            relative = Path("probes") / f"batched-{side}" / "src/main.rs"
            _, raw = _read_path(root, relative.as_posix(), f"{side} probe source", absolute=False)
            source_paths[side] = str(root / relative)
            source_bytes[side] = raw
        _expect(source_bytes["baseline"] == source_bytes["candidate"], "paired probe sources differ")
        source_sha = _digest(source_bytes["baseline"])
        _expect(source_sha == run_index["source_sha256"], "current probe source differs from run index")
        fnv = 0xCBF29CE484222325
        for byte in source_bytes["baseline"]:
            fnv = ((fnv ^ byte) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
        source_fnv = f"{fnv:016x}"

        for side in ("baseline", "candidate"):
            snapshot = manifest["snapshots"][side]
            _expect(snapshot["files"][SCENE] == scene_hashes[side], f"{side} snapshot scene drift")
        builds = run_index.get("builds")
        _expect(isinstance(builds, list) and len(builds) == 2
                and [build.get("side") for build in builds] == ["baseline", "candidate"],
                "run index must contain baseline and candidate builds")
        for build in builds:
            side = build["side"]
            _expect(type(build.get("exit_code")) is int and build["exit_code"] == 0
                    and build.get("manifest_sha256") == manifest_sha
                    and build.get("probe_source_sha256") == source_sha
                    and build.get("scene_sha256") == scene_hashes[side],
                    f"{side} build provenance mismatch")
            binary = Path(build.get("binary_path", ""))
            _, binary_raw = _read_path(root, str(binary), f"{side} release binary")
            _expect(_digest(binary_raw) == build.get("binary_sha256"), f"{side} release binary digest mismatch")

        raw_refs = summary.get("raw_outputs")
        _expect(isinstance(raw_refs, list) and len(raw_refs) == 10, "summary must list ten raw outputs")
        raw_by_path: dict[str, bytes] = {}
        for ref in raw_refs:
            relative, raw = _verify_ref(root, ref, "raw output")
            path = str(root / relative)
            _expect(path not in raw_by_path, "duplicate raw output path")
            raw_by_path[path] = raw

        runs = run_index.get("runs")
        _expect(isinstance(runs, list) and len(runs) == 10, "run index must contain five paired runs")
        _expect(run_index.get("pair_order") == "five fresh-process pairs; alternate baseline/candidate and candidate/baseline",
                "run-index pair-order contract mismatch")
        raw_cases: dict[tuple[int, str], dict[tuple[str, int], dict[str, Any]]] = {}
        output_paths: set[str] = set()
        for pair in range(5):
            pair_rows = runs[pair * 2:pair * 2 + 2]
            expected_order = ("baseline", "candidate") if pair % 2 == 0 else ("candidate", "baseline")
            _expect([row.get("variant") for row in pair_rows] == list(expected_order)
                    and all(type(row.get("pair")) is int and row["pair"] == pair for row in pair_rows),
                    f"invalid run pair {pair}")
            for row in pair_rows:
                side = row["variant"]
                _expect(row.get("candidate_id") == CANDIDATE_ID
                        and row.get("manifest_sha256") == manifest_sha
                        and row.get("source_sha256") == source_sha
                        and row.get("scene_sha256") == scene_hashes[side]
                        and row.get("profile") == "release"
                        and type(row.get("exit_code")) is int and row["exit_code"] == 0,
                        f"run pair {pair} {side} identity or exit code mismatch")
                binary_path = row.get("binary_path")
                _expect(isinstance(binary_path, str), f"run pair {pair} missing binary path")
                _, binary_raw = _read_path(root, binary_path, f"run pair {pair} {side} binary")
                _expect(_digest(binary_raw) == row.get("binary_sha256"),
                        f"run pair {pair} {side} binary digest mismatch")
                output = row.get("output")
                _expect(isinstance(output, str) and output not in output_paths,
                        f"run pair {pair} duplicate output path")
                output_paths.add(output)
                _expect(output in raw_by_path, f"run pair {pair} output absent from summary")
                raw = raw_by_path[output]
                _expect(_digest(raw) == row.get("output_sha256"), f"run pair {pair} output digest mismatch")
                raw_cases[(pair, side)] = _check_raw_run(raw, f"run pair {pair} {side}", source_fnv)
        _expect(output_paths == set(raw_by_path), "summary raw outputs do not exactly match run-index outputs")

        _expect(provenance.get("run_index_path") == record_paths["run_index"]
                and provenance.get("run_index_sha256") == run_index_sha,
                "provenance record does not bind the run index")
        probe_source = provenance.get("probe_source", {})
        _expect(probe_source.get("sha256") == source_sha and probe_source.get("fnv1a64") == source_fnv,
                "provenance probe-source identity mismatch")
        snapshots = provenance.get("snapshots", {})
        _expect(snapshots.get("manifest_path") == str(root / manifest_rel)
                and snapshots.get("manifest_sha256") == manifest_sha,
                "provenance snapshot manifest identity mismatch")
        selected_scene = snapshots.get("selected_scene", {})
        for side in ("baseline", "candidate"):
            expected_path = str(root / side / "source" / SCENE)
            _expect(selected_scene.get(side, {}).get("manifest_sha256") == scene_hashes[side]
                    and selected_scene.get(side, {}).get("current_sha256") == scene_hashes[side]
                    and selected_scene.get(side, {}).get("path") == expected_path,
                    f"provenance {side} scene identity mismatch")
        verified_rows = provenance.get("verified_rows")
        _expect(isinstance(verified_rows, list) and len(verified_rows) == 10
                and all(isinstance(row, dict) and type(row.get("pair")) is int
                        for row in verified_rows),
                "provenance must contain ten verified rows")
        for pair, side in ((p, s) for p in range(5) for s in ("baseline", "candidate")):
            matching = [row for row in verified_rows if row.get("pair") == pair and row.get("variant") == side]
            _expect(len(matching) == 1, f"provenance missing or duplicates run {pair} {side}")
            output = runs[next(i for i, row in enumerate(runs) if row.get("pair") == pair and row.get("variant") == side)]["output"]
            run = next(row for row in runs if row.get("pair") == pair and row.get("variant") == side)
            _expect(matching[0].get("output") == output
                    and matching[0].get("source_sha256") == source_sha
                    and matching[0].get("selected_scene_sha256") == scene_hashes[side]
                    and matching[0].get("binary_path_from_probe_layout") == run.get("binary_path")
                    and matching[0].get("binary_sha256_current") == run.get("binary_sha256"),
                    f"provenance row {pair} {side} conflicts with run index")

        _expect(comparison.get("schema_version") == 2 and comparison.get("candidate_id") == CANDIDATE_ID
                and comparison.get("protocol") == PROTOCOL and comparison.get("profile") == "release"
                and comparison.get("run_index_sha256") == run_index_sha
                and comparison.get("snapshot_manifest_sha256") == manifest_sha
                and comparison.get("provenance_verification_path") == record_paths["provenance"],
                "comparison identity does not bind provenance and run index")
        _expect(comparison.get("source", {}).get("sha256") == source_sha,
                "comparison probe-source digest mismatch")
        baseline_runs = comparison.get("baseline_runs")
        candidate_runs = comparison.get("candidate_runs")
        _expect(isinstance(baseline_runs, list) and len(baseline_runs) == 5
                and isinstance(candidate_runs, list) and len(candidate_runs) == 5,
                "comparison must contain five runs per side")
        for side, rows in (("baseline", baseline_runs), ("candidate", candidate_runs)):
            for pair, item in enumerate(rows):
                expected_path = next(row["output"] for row in runs
                                      if row.get("pair") == pair and row.get("variant") == side)
                _expect(item.get("path") == expected_path
                        and item.get("sha256") == next(row["output_sha256"] for row in runs
                                                        if row.get("pair") == pair and row.get("variant") == side),
                        f"comparison {side} run {pair} does not match run index")
        comp_cases = comparison.get("cases")
        expected_keys = {(kind, length) for kind in KINDS for length in LENGTHS}
        _expect(isinstance(comp_cases, list) and len(comp_cases) == len(expected_keys),
                "comparison case matrix is incomplete")
        seen_cases: set[tuple[str, int]] = set()
        for case in comp_cases:
            key = (case.get("kind"), case.get("len"))
            _expect(key in expected_keys and key not in seen_cases, "comparison has duplicate/unknown case")
            seen_cases.add(key)
            pair_rows = case.get("pairs")
            _expect(isinstance(pair_rows, list) and len(pair_rows) == 5,
                    "comparison case must contain five pairs")
            ratios: list[float] = []
            for pair, item in enumerate(pair_rows):
                base = raw_cases[(pair, "baseline")][key]
                cand = raw_cases[(pair, "candidate")][key]
                for field, metric_field in (("baseline_ns_per_iteration", "ns_per_iteration"),
                                            ("candidate_ns_per_iteration", "ns_per_iteration"),
                                            ("baseline_reset_ns_per_iteration", "reset_ns_per_iteration"),
                                            ("candidate_reset_ns_per_iteration", "reset_ns_per_iteration")):
                    side = "baseline" if field.startswith("baseline") else "candidate"
                    raw_case = base if side == "baseline" else cand
                    _same_number(item.get(field), _metric(raw_case, metric_field), f"comparison {key} {field}")
                ratio = _metric(cand, "ns_per_iteration") / _metric(base, "ns_per_iteration")
                _same_number(item.get("candidate_over_baseline"), ratio,
                             f"comparison {key} pair ratio")
                ratios.append(ratio)
                for field, side, total in (
                    ("baseline_allocation_calls_per_iteration", base, "allocation_calls_total"),
                    ("candidate_allocation_calls_per_iteration", cand, "allocation_calls_total"),
                    ("baseline_allocated_bytes_per_iteration", base, "allocated_bytes_total"),
                    ("candidate_allocated_bytes_per_iteration", cand, "allocated_bytes_total"),
                    ("baseline_freed_bytes_per_iteration", base, "freed_bytes_total"),
                    ("candidate_freed_bytes_per_iteration", cand, "freed_bytes_total"),
                ):
                    _same_number(item.get(field), _metric(side, total) / side["iterations_per_sample"],
                                 f"comparison {key} {field}")
            _same_number(case.get("median_candidate_over_baseline"), statistics.median(ratios),
                         f"comparison {key} median ratio")
            _same_number(case.get("pair_ratio_min"), min(ratios), f"comparison {key} minimum ratio")
            _same_number(case.get("pair_ratio_max"), max(ratios), f"comparison {key} maximum ratio")
            _same_number(case.get("pair_ratio_range"), max(ratios) - min(ratios),
                         f"comparison {key} ratio range")
        _expect(seen_cases == expected_keys, "comparison case matrix differs from raw runs")

        # Verify all summary-listed build/process logs using the same no-follow reader.
        process_record = summary.get("processes", {})
        exit_codes = process_record.get("exit_codes")
        _expect(type(process_record.get("count")) is int and process_record["count"] == 10
                and isinstance(exit_codes, list) and len(exit_codes) == 10
                and all(type(code) is int and code == 0 for code in exit_codes),
                "summary process status is not ten successful runs")
        process_logs = process_record.get("logs")
        _expect(isinstance(process_logs, list) and len(process_logs) == 10,
                "summary process logs are incomplete")
        for ref in process_logs:
            _verify_ref(root, ref, "process log")
        summary_builds = summary.get("builds")
        _expect(isinstance(summary_builds, list) and len(summary_builds) == 2,
                "summary build records are incomplete")
        for build in summary_builds:
            _expect(build.get("side") in ("baseline", "candidate")
                    and type(build.get("exit_code")) is int and build["exit_code"] == 0,
                    "summary build status is invalid")
            _verify_ref(root, build.get("log"), "build log")

        report["status"] = "valid"
        report["measurement_status"] = "valid"
        report["candidate_id"] = CANDIDATE_ID
        report["run_id"] = RUN_ID
        report["manifest_sha256"] = manifest_sha
        report["run_index_sha256"] = run_index_sha
        report["comparison_sha256"] = comparison_sha
    except (OSError, TypeError, KeyError, IndexError, AttributeError, ValueError) as error:
        errors.append(str(error))
    return report


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("summary", type=Path, help="M276 rerun summary beneath candidate root")
    parser.add_argument("--root", type=Path, required=True, help="quad-stable-sort-v2 candidate root")
    args = parser.parse_args()
    report = validate(args.root, args.summary)
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0 if report["status"] == "valid" else 1


if __name__ == "__main__":
    raise SystemExit(main())
