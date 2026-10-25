"""Synthetic tests for the read-only Quad Sort performance validator."""

from __future__ import annotations

import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "validate_quad_sort_performance.py"
CANDIDATE_ID = "quad-stable-sort-v2-20261020-a"
RUN_ID = "quad-v2-release-20261020-b"
PROTOCOL = "reset_copy_finish_batch_v1"
KINDS = ("dense", "reverse", "ordered", "duplicates", "sparse_max")
LENGTHS = (16, 1023, 1024, 1025, 16384, 131071, 131072, 131073)


def digest(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def json_bytes(value: object) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode()


def fnv1a64(raw: bytes) -> str:
    value = 0xCBF29CE484222325
    for byte in raw:
        value = ((value ^ byte) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return f"{value:016x}"


class ValidateQuadSortPerformanceTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name).resolve() / CANDIDATE_ID
        self.root.mkdir()
        self.evidence = self.root / "evidence" / "m276-release-test"
        self.evidence.mkdir(parents=True)
        self.manifest_path = self.root / "manifest.json"
        self.summary_path = self.evidence / "m276-rerun-summary.json"
        self.summary = self._make_fixture()
        self._write_summary()

    def _write_json(self, relative: str, value: object) -> tuple[Path, bytes]:
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        raw = json_bytes(value)
        path.write_bytes(raw)
        return path, raw

    def _reference(self, path: Path, raw: bytes) -> dict[str, object]:
        return {"bytes": len(raw), "path": str(path.resolve()), "sha256": digest(raw)}

    def _make_fixture(self) -> dict[str, object]:
        baseline_scene = b"baseline scene fixture\n"
        candidate_scene = b"candidate scene fixture\n"
        scene_relative = "path-sources/gpui-pre-0.3.9/src/scene.rs"
        source_roots = {side: self.root / side / "source" for side in ("baseline", "candidate")}
        scene_hashes: dict[str, str] = {}
        for side, raw in (("baseline", baseline_scene), ("candidate", candidate_scene)):
            scene_path = source_roots[side] / scene_relative
            scene_path.parent.mkdir(parents=True, exist_ok=True)
            scene_path.write_bytes(raw)
            scene_hashes[side] = digest(raw)

        manifest = {
            "schema_version": 1,
            "candidate_id": CANDIDATE_ID,
            "preimage_sha256": scene_hashes["baseline"],
            "postimage_sha256": scene_hashes["candidate"],
            "snapshots": {
                side: {
                    "source": str(source_roots[side].resolve()),
                    "files": {scene_relative: scene_hashes[side]},
                }
                for side in ("baseline", "candidate")
            },
        }
        manifest_raw = json_bytes(manifest)
        self.manifest_path.write_bytes(manifest_raw)
        manifest_hash = digest(manifest_raw)

        probe_source = b"fn main() { /* deterministic synthetic probe */ }\n"
        probe_hash = digest(probe_source)
        probe_fnv = fnv1a64(probe_source)
        probe_paths: dict[str, Path] = {}
        for side in ("baseline", "candidate"):
            probe_path = self.root / "probes" / f"batched-{side}" / "src" / "main.rs"
            probe_path.parent.mkdir(parents=True, exist_ok=True)
            probe_path.write_bytes(probe_source)
            probe_paths[side] = probe_path

        binary_paths: dict[str, Path] = {}
        binary_hashes: dict[str, str] = {}
        for side in ("baseline", "candidate"):
            binary_path = (self.root / "probes" / f"{side}-target-m276-b" / "release"
                           / "rustui-quad-batched-probe")
            binary_path.parent.mkdir(parents=True, exist_ok=True)
            binary_raw = f"synthetic {side} release binary\n".encode()
            binary_path.write_bytes(binary_raw)
            binary_paths[side] = binary_path
            binary_hashes[side] = digest(binary_raw)

        output_refs: list[dict[str, object]] = []
        run_rows: list[dict[str, object]] = []
        verified_rows: list[dict[str, object]] = []
        comparison_run_refs = {"baseline": [], "candidate": []}
        cases_by_run: dict[tuple[int, str], list[dict[str, object]]] = {}

        for pair in range(5):
            first_side = "baseline" if pair % 2 == 0 else "candidate"
            order = (first_side, "candidate" if first_side == "baseline" else "baseline")
            pair_id = f"quad-v2-batched-m276-20261020-b-{pair:02d}"
            for side in order:
                cases = []
                for kind in KINDS:
                    for length in LENGTHS:
                        iterations = max(1, (8192 + length - 1) // length)
                        sample_count = 3 if length >= 131072 else 20
                        ns_per_iteration = 1000 if side == "baseline" else 1100
                        reset_ns_per_iteration = 100
                        samples = [
                            {
                                "iterations": iterations,
                                "batch_ns": iterations * ns_per_iteration,
                                "reset_batch_ns": iterations * reset_ns_per_iteration,
                                "ns_per_iteration": float(ns_per_iteration),
                                "reset_ns_per_iteration": float(reset_ns_per_iteration),
                                "allocation_calls_total": 0,
                                "allocated_bytes_total": 0,
                                "freed_bytes_total": 0,
                            }
                            for _ in range(sample_count)
                        ]
                        cases.append({
                            "kind": kind,
                            "len": length,
                            "iterations_per_sample": iterations,
                            "oracle_hash_fnv1a64": f"{length:016x}",
                            "output_hash_fnv1a64": f"{length:016x}",
                            "correctness": "stable_all_semantic_fields",
                            "cold": {
                                "iterations": iterations,
                                "batch_ns": iterations * ns_per_iteration,
                                "reset_batch_ns": iterations * reset_ns_per_iteration,
                            },
                            "trials": [
                                {"trial": trial, "samples": samples}
                                for trial in range(9)
                            ],
                        })

                output_name = f"quad-v2-batched-m276-20261020-b-{pair:02d}-{side}.json"
                output_path, output_raw = self._write_json(
                    f"evidence/m276-release-test/{output_name}",
                    {
                        "schema_version": 2,
                        "kind": "scene_finish_cpu_microbenchmark",
                        "protocol": PROTOCOL,
                        "profile": "release",
                        "debug_assertions": False,
                        "probe_source_fnv1a64": probe_fnv,
                        "timing_method": "reset_copy_plus_finish_batch",
                        "reset_timing_method": "reset_copy_only_batch",
                        "iterations_policy": "max(1,ceil(8192/len))",
                        "seed": "0x51554144",
                        "warmup": 5,
                        "trials": 9,
                        "mixed_batch_audit": {"passed": True, "batches": 1, "oracle_batches": 1},
                        "cases": cases,
                    },
                )
                output_ref = self._reference(output_path, output_raw)
                output_refs.append(output_ref)
                cases_by_run[(pair, side)] = cases
                run_rows.append({
                    "argv": [str(binary_paths[side].resolve()), "--bench", str(output_path.resolve())],
                    "binary_path": str(binary_paths[side].resolve()),
                    "binary_sha256": binary_hashes[side],
                    "candidate_id": CANDIDATE_ID,
                    "cwd": str((self.root / "probes" / f"batched-{side}").resolve()),
                    "exit_code": 0,
                    "finished_at": f"2026-10-21T00:{pair:02d}:01+00:00",
                    "manifest_sha256": manifest_hash,
                    "output": str(output_path.resolve()),
                    "output_sha256": digest(output_raw),
                    "pair": pair,
                    "profile": "release",
                    "scene_sha256": scene_hashes[side],
                    "source_sha256": probe_hash,
                    "started_at": f"2026-10-21T00:{pair:02d}:00+00:00",
                    "variant": side,
                    "wall_elapsed_ns": 1000000,
                })
                verified_rows.append({
                    "pair": pair,
                    "variant": side,
                    "output": str(output_path.resolve()),
                    "output_sha256": digest(output_raw),
                    "source_sha256": probe_hash,
                    "binary_path_from_probe_layout": str(binary_paths[side].resolve()),
                    "binary_sha256_current": binary_hashes[side],
                    "selected_scene_sha256": scene_hashes[side],
                })
                comparison_run_refs[side].append({
                    "path": str(output_path.resolve()),
                    "sha256": digest(output_raw),
                    "pair_id": pair_id,
                })

        build_rows: list[dict[str, object]] = []
        build_refs: list[dict[str, object]] = []
        for side in ("baseline", "candidate"):
            build_log_path, build_log_raw = self._write_json(
                f"evidence/m276-release-test/build-{side}.log", {"status": "passed"}
            )
            build_log_ref = self._reference(build_log_path, build_log_raw)
            build_refs.append({"side": side, "exit_code": 0, "log": build_log_ref})
            build_rows.append({
                "CARGO_TARGET_DIR": str(binary_paths[side].parents[1].resolve()),
                "argv": ["cargo-headstart", "build", "--release", "--locked", "--offline"],
                "binary_path": str(binary_paths[side].resolve()),
                "binary_sha256": binary_hashes[side],
                "cwd": str((self.root / "probes" / f"batched-{side}").resolve()),
                "exit_code": 0,
                "finished_at": "2026-10-21T00:00:00+00:00",
                "manifest_sha256": manifest_hash,
                "probe_source_sha256": probe_hash,
                "scene_sha256": scene_hashes[side],
                "side": side,
                "started_at": "2026-10-21T00:00:00+00:00",
            })

        run_index, run_index_raw = self._write_json(
            "evidence/m276-release-test/quad-v2-release-test-run-index.json",
            {
                "schema_version": 1,
                "status": "complete",
                "candidate_id": CANDIDATE_ID,
                "manifest_sha256": manifest_hash,
                "source_sha256": probe_hash,
                "protocol": PROTOCOL,
                "run_id": RUN_ID,
                "profile": "release",
                "pair_order": "five fresh-process pairs; alternate baseline/candidate and candidate/baseline",
                "kinds": list(KINDS),
                "lengths": list(LENGTHS),
                "machine": {"architecture": "arm64", "platform": "synthetic", "python": "3.14"},
                "builds": build_rows,
                "runs": run_rows,
            },
        )

        provenance, provenance_raw = self._write_json(
            "evidence/m276-release-test/quad-v2-release-test-provenance-verification.json",
            {
                "run_index_path": str(run_index.resolve()),
                "run_index_sha256": digest(run_index_raw),
                "probe_source": {
                    "sha256": probe_hash,
                    "fnv1a64": probe_fnv,
                    "baseline_path": str(probe_paths["baseline"].resolve()),
                    "candidate_path": str(probe_paths["candidate"].resolve()),
                },
                "snapshots": {
                    "manifest_path": str(self.manifest_path.resolve()),
                    "manifest_sha256": manifest_hash,
                    "selected_scene": {
                        side: {
                            "path": str((source_roots[side] / scene_relative).resolve()),
                            "manifest_sha256": scene_hashes[side],
                            "current_sha256": scene_hashes[side],
                        }
                        for side in ("baseline", "candidate")
                    },
                },
                "verified_rows": verified_rows,
                "scope": "Synthetic fixture; current file/hash verification only.",
            },
        )

        comparison_cases = []
        for kind in KINDS:
            for length in LENGTHS:
                iterations = max(1, (8192 + length - 1) // length)
                pairs = []
                for pair in range(5):
                    pairs.append({
                        "baseline_ns_per_iteration": 1000.0,
                        "candidate_ns_per_iteration": 1100.0,
                        "candidate_over_baseline": 1.1,
                        "baseline_reset_ns_per_iteration": 100.0,
                        "candidate_reset_ns_per_iteration": 100.0,
                        "baseline_allocation_calls_per_iteration": 0.0,
                        "candidate_allocation_calls_per_iteration": 0.0,
                        "baseline_allocated_bytes_per_iteration": 0.0,
                        "candidate_allocated_bytes_per_iteration": 0.0,
                        "baseline_freed_bytes_per_iteration": 0.0,
                        "candidate_freed_bytes_per_iteration": 0.0,
                    })
                comparison_cases.append({
                    "kind": kind,
                    "len": length,
                    "iterations_per_sample": iterations,
                    "median_candidate_over_baseline": 1.1,
                    "pair_ratio_min": 1.1,
                    "pair_ratio_max": 1.1,
                    "pair_ratio_range": 0.0,
                    "pairs": pairs,
                })

        comparison, comparison_raw = self._write_json(
            "evidence/m276-release-test/quad-v2-release-test-comparison.json",
            {
                "schema_version": 2,
                "candidate_id": CANDIDATE_ID,
                "protocol": PROTOCOL,
                "profile": "release",
                "timing_method": "reset-copy plus Scene.finish, separately measured reset-copy-only; no automatic subtraction",
                "source": {
                    "sha256": probe_hash,
                    "fnv1a64": probe_fnv,
                    "baseline_path": str(probe_paths["baseline"].resolve()),
                    "candidate_path": str(probe_paths["candidate"].resolve()),
                },
                "provenance_verification_path": str(provenance.resolve()),
                "run_index_sha256": digest(run_index_raw),
                "snapshot_manifest_sha256": manifest_hash,
                "baseline_runs": comparison_run_refs["baseline"],
                "candidate_runs": comparison_run_refs["candidate"],
                "cases": comparison_cases,
                "scope": "Synthetic paired CPU batch measurements.",
            },
        )

        script_refs = []
        for name, content in (("run_m276_release.py", b"# synthetic runner\n"),
                              ("compare_m276_release.py", b"# synthetic comparer\n")):
            script_path, script_raw = self._write_json(
                f"evidence/m276-release-test/{name}", content.decode()
            )
            # Write exact source bytes rather than JSON encoding for scripts.
            script_path.write_bytes(content)
            script_refs.append(self._reference(script_path, content))

        process_logs = []
        for pair in range(5):
            for side in ("baseline", "candidate"):
                log_path, log_raw = self._write_json(
                    f"evidence/m276-release-test/process-{pair:02d}-{side}.log",
                    {"exit_code": 0},
                )
                process_logs.append(self._reference(log_path, log_raw))

        summary: dict[str, object] = {
            "schema_version": 1,
            "stage": 10,
            "workstream": "P3",
            "milestone": "M276",
            "candidate_id": CANDIDATE_ID,
            "run_id": RUN_ID,
            "status": "completed_measurement_no_performance_acceptance",
            "profile": "release",
            "protocol": PROTOCOL,
            "source_manifest_sha256": manifest_hash,
            "probe_source_sha256": probe_hash,
            "acceptance_thresholds": {
                "defined": False,
                "finding": "No numeric performance acceptance thresholds are defined.",
            },
            "builds": build_refs,
            "processes": {
                "count": 10,
                "exit_codes": [0] * 10,
                "logs": process_logs,
            },
            "raw_outputs": output_refs,
            "records": [
                self._reference(run_index, run_index_raw),
                self._reference(provenance, provenance_raw),
                self._reference(comparison, comparison_raw),
                *script_refs,
            ],
            "results": {
                "median_case_ratio": 1.1,
                "median_pair_ratio": 1.1,
                "median_within_case_pair_spread": 0.0,
                "worst_median_case": {
                    "kind": "dense",
                    "len": 16,
                    "median_candidate_over_baseline": 1.1,
                    "pair_ratio_min": 1.1,
                    "pair_ratio_max": 1.1,
                },
                "interpretation": "Synthetic data; acceptance is not assessed.",
            },
            "workload": {
                "pairs": 5,
                "case_count": 40,
                "paired_ratio_count": 200,
                "kinds": list(KINDS),
                "lengths": list(LENGTHS),
                "differential_oracle_checks": "passed all paired case outputs",
            },
            "cleanup": {
                "gui_app_cleanup_applicable": False,
                "reason": "Synthetic command-line probe fixture.",
            },
            "scope": "Synthetic fixture; no performance acceptance is implied.",
        }
        return summary

    def _write_summary(self) -> None:
        self.summary_path.write_bytes(json_bytes(self.summary))

    def invoke(self) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        process = subprocess.run(
            [sys.executable, str(SCRIPT), str(self.summary_path), "--root", str(self.root)],
            capture_output=True,
            text=True,
            check=False,
        )
        return process, json.loads(process.stdout)

    def _record_by_name(self, suffix: str) -> dict[str, object]:
        return next(
            item for item in self.summary["records"]  # type: ignore[index]
            if str(item["path"]).endswith(suffix)
        )

    def test_valid_summary_reports_measurement_without_acceptance(self) -> None:
        process, report = self.invoke()

        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(report["measurement_status"], "valid")
        self.assertEqual(report["acceptance_status"], "not_assessed")
        self.assertEqual(report["performance_gate"], "not_run")
        self.assertEqual(report["promotion_gate"], "not_run")

    def test_rejects_non_integer_mixed_batch_counts(self) -> None:
        spec = importlib.util.spec_from_file_location("quad_sort_performance_validator", SCRIPT)
        assert spec is not None and spec.loader is not None
        validator = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(validator)
        output = self.summary["raw_outputs"][0]  # type: ignore[index]
        data = json.loads(Path(str(output["path"])).read_bytes())

        for value in ("1", True):
            with self.subTest(value=value):
                changed = json.loads(json.dumps(data))
                changed["mixed_batch_audit"]["batches"] = value
                changed["mixed_batch_audit"]["oracle_batches"] = value
                with self.assertRaisesRegex(validator.ValidationError, "mixed-batch oracle failed"):
                    validator._check_raw_run(
                        json_bytes(changed), "synthetic raw output",
                        changed["probe_source_fnv1a64"],
                    )

    def test_rejects_non_integer_raw_schema_version(self) -> None:
        spec = importlib.util.spec_from_file_location("quad_sort_performance_validator", SCRIPT)
        assert spec is not None and spec.loader is not None
        validator = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(validator)
        output = self.summary["raw_outputs"][0]  # type: ignore[index]
        data = json.loads(Path(str(output["path"])).read_bytes())
        data["schema_version"] = 2.0

        with self.assertRaisesRegex(validator.ValidationError, "unsupported raw schema"):
            validator._check_raw_run(
                json_bytes(data), "synthetic raw output", data["probe_source_fnv1a64"]
            )

    def test_rejects_boolean_cold_iteration_count(self) -> None:
        spec = importlib.util.spec_from_file_location("quad_sort_performance_validator", SCRIPT)
        assert spec is not None and spec.loader is not None
        validator = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(validator)
        output = self.summary["raw_outputs"][0]  # type: ignore[index]
        data = json.loads(Path(str(output["path"])).read_bytes())
        case = next(case for case in data["cases"] if case["len"] == 16384)
        case["cold"]["iterations"] = True

        with self.assertRaisesRegex(validator.ValidationError, "invalid cold timing"):
            validator._check_raw_run(
                json_bytes(data), "synthetic raw output", data["probe_source_fnv1a64"]
            )

    def test_malformed_nested_summary_returns_invalid_report(self) -> None:
        self.summary["acceptance_thresholds"] = []
        self._write_summary()

        process, report = self.invoke()

        self.assertNotEqual(process.returncode, 0)
        self.assertEqual(report["measurement_status"], "invalid")
        self.assertTrue(report["errors"])

    def test_rejects_boolean_schema_version(self) -> None:
        self.summary["schema_version"] = True
        self._write_summary()

        process, report = self.invoke()

        self.assertNotEqual(process.returncode, 0)
        self.assertEqual(report["measurement_status"], "invalid")
        self.assertTrue(
            any("summary milestone identity mismatch" in error for error in report["errors"])
        )

    def test_rejects_boolean_process_exit_code(self) -> None:
        self.summary["processes"]["exit_codes"][0] = False  # type: ignore[index]
        self._write_summary()

        process, report = self.invoke()

        self.assertNotEqual(process.returncode, 0)
        self.assertEqual(report["measurement_status"], "invalid")
        self.assertTrue(
            any("summary process status" in error for error in report["errors"])
        )

    def test_rejects_boolean_summary_build_exit_code(self) -> None:
        self.summary["builds"][0]["exit_code"] = False  # type: ignore[index]
        self._write_summary()

        process, report = self.invoke()

        self.assertNotEqual(process.returncode, 0)
        self.assertEqual(report["measurement_status"], "invalid")
        self.assertTrue(
            any("summary build status" in error for error in report["errors"])
        )

    def test_rejects_boolean_provenance_pair(self) -> None:
        provenance_ref = self._record_by_name("provenance-verification.json")
        provenance_path = Path(str(provenance_ref["path"]))
        provenance = json.loads(provenance_path.read_text())
        provenance["verified_rows"][0]["pair"] = False
        provenance_raw = json_bytes(provenance)
        provenance_path.write_bytes(provenance_raw)
        provenance_ref["sha256"] = digest(provenance_raw)
        provenance_ref["bytes"] = len(provenance_raw)
        self._write_summary()

        process, report = self.invoke()

        self.assertNotEqual(process.returncode, 0)
        self.assertEqual(report["measurement_status"], "invalid")
        self.assertTrue(
            any("provenance must contain ten verified rows" in error
                for error in report["errors"])
        )

    def test_rejects_boolean_run_index_schema_version(self) -> None:
        run_index_ref = self._record_by_name("run-index.json")
        run_index_path = Path(str(run_index_ref["path"]))
        run_index = json.loads(run_index_path.read_text())
        run_index["schema_version"] = True
        run_index_raw = json_bytes(run_index)
        run_index_path.write_bytes(run_index_raw)
        run_index_ref["sha256"] = digest(run_index_raw)
        run_index_ref["bytes"] = len(run_index_raw)

        for filename, field in (
            ("provenance-verification.json", "run_index_sha256"),
            ("comparison.json", "run_index_sha256"),
        ):
            artifact = self._record_by_name(filename)
            path = Path(str(artifact["path"]))
            value = json.loads(path.read_text())
            value[field] = digest(run_index_raw)
            raw = json_bytes(value)
            path.write_bytes(raw)
            artifact["sha256"] = digest(raw)
            artifact["bytes"] = len(raw)

        self._write_summary()
        process, report = self.invoke()

        self.assertNotEqual(process.returncode, 0)
        self.assertEqual(report["measurement_status"], "invalid")
        self.assertTrue(
            any("run-index identity mismatch" in error for error in report["errors"])
        )

    def test_rejects_path_escape_and_symlinked_artifacts(self) -> None:
        artifact = self._record_by_name("run-index.json")
        original_path = str(artifact["path"])
        outside = self.root.parent / "outside-run-index.json"
        outside.write_bytes(Path(original_path).read_bytes())
        self.addCleanup(lambda: outside.unlink(missing_ok=True))

        artifact["path"] = str(outside.resolve())
        self._write_summary()
        process, report = self.invoke()
        self.assertNotEqual(process.returncode, 0)
        self.assertEqual(report["measurement_status"], "invalid")

        alias = self.evidence / "run-index-alias.json"
        alias.symlink_to(Path(original_path))
        artifact["path"] = str(alias.resolve(strict=False))
        # Keep the alias itself in the record; resolve(strict=False) would follow it.
        artifact["path"] = str(alias.absolute())
        self._write_summary()
        process, report = self.invoke()
        self.assertNotEqual(process.returncode, 0)
        self.assertEqual(report["measurement_status"], "invalid")

    def test_rejects_artifact_digest_drift(self) -> None:
        artifact = self._record_by_name("run-index.json")
        Path(str(artifact["path"])).write_bytes(b"changed after summary\n")

        process, report = self.invoke()

        self.assertNotEqual(process.returncode, 0)
        self.assertEqual(report["measurement_status"], "invalid")

    def test_rejects_candidate_transplant(self) -> None:
        self.summary["candidate_id"] = "another-candidate"
        self._write_summary()

        process, report = self.invoke()

        self.assertNotEqual(process.returncode, 0)
        self.assertEqual(report["measurement_status"], "invalid")

    def test_rejects_missing_or_duplicate_pair_artifacts(self) -> None:
        original = list(self.summary["raw_outputs"])  # type: ignore[arg-type]
        for case, outputs in (
            ("missing", original[:-1]),
            ("duplicate", [*original[:-1], original[0]]),
        ):
            with self.subTest(case=case):
                self.summary["raw_outputs"] = outputs
                self._write_summary()
                process, report = self.invoke()
                self.assertNotEqual(process.returncode, 0)
                self.assertEqual(report["measurement_status"], "invalid")
                self.assertEqual(report["acceptance_status"], "not_assessed")
                self.assertEqual(report["performance_gate"], "not_run")
        self.summary["raw_outputs"] = original

    def test_rejects_invalid_timing_value(self) -> None:
        output = self.summary["raw_outputs"][0]  # type: ignore[index]
        output_path = Path(str(output["path"]))
        raw = json.loads(output_path.read_text())
        raw["cases"][0]["trials"][0]["samples"][0]["ns_per_iteration"] = float("nan")
        changed = json_bytes(raw)
        output_path.write_bytes(changed)
        output["sha256"] = digest(changed)
        output["bytes"] = len(changed)

        run_index_ref = self._record_by_name("run-index.json")
        run_index_path = Path(str(run_index_ref["path"]))
        run_index = json.loads(run_index_path.read_text())
        for row in run_index["runs"]:
            if row["output"] == str(output_path.resolve()):
                row["output_sha256"] = digest(changed)
        run_index_raw = json_bytes(run_index)
        run_index_path.write_bytes(run_index_raw)
        run_index_ref["sha256"] = digest(run_index_raw)
        run_index_ref["bytes"] = len(run_index_raw)

        provenance_ref = self._record_by_name("provenance-verification.json")
        provenance_path = Path(str(provenance_ref["path"]))
        provenance = json.loads(provenance_path.read_text())
        provenance["run_index_sha256"] = digest(run_index_raw)
        for row in provenance["verified_rows"]:
            if row["output"] == str(output_path.resolve()):
                row["output_sha256"] = digest(changed)
        provenance_raw = json_bytes(provenance)
        provenance_path.write_bytes(provenance_raw)
        provenance_ref["sha256"] = digest(provenance_raw)
        provenance_ref["bytes"] = len(provenance_raw)

        comparison_ref = self._record_by_name("comparison.json")
        comparison_path = Path(str(comparison_ref["path"]))
        comparison = json.loads(comparison_path.read_text())
        comparison["run_index_sha256"] = digest(run_index_raw)
        for side in ("baseline", "candidate"):
            for row in comparison[f"{side}_runs"]:
                if row["path"] == str(output_path.resolve()):
                    row["sha256"] = digest(changed)
        comparison_raw = json_bytes(comparison)
        comparison_path.write_bytes(comparison_raw)
        comparison_ref["sha256"] = digest(comparison_raw)
        comparison_ref["bytes"] = len(comparison_raw)
        self._write_summary()

        process, report = self.invoke()

        self.assertNotEqual(process.returncode, 0)
        self.assertEqual(report["measurement_status"], "invalid")
        self.assertEqual(report["acceptance_status"], "not_assessed")
        self.assertEqual(report["performance_gate"], "not_run")

    def test_rejects_out_of_range_integer_timing_value(self) -> None:
        spec = importlib.util.spec_from_file_location("quad_sort_performance_validator", SCRIPT)
        assert spec is not None and spec.loader is not None
        validator = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(validator)
        with self.assertRaisesRegex(validator.ValidationError, "out-of-range"):
            validator._same_number(10**1000, 1.0, "timing")


if __name__ == "__main__":
    unittest.main()
