import importlib.util
import hashlib
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock


SCRIPT = Path(__file__).resolve().parents[1] / "validate_p3d_visual_evidence.py"
SPEC = importlib.util.spec_from_file_location("validate_p3d_visual_evidence", SCRIPT)
validator = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(validator)


def write_provenance(root: Path):
    (root / "cache_comparison").write_bytes(b"binary")
    binary_hash = hashlib.sha256(b"binary").hexdigest()
    build = {"binary": {"path": "cache_comparison", "sha256": binary_hash}}
    build_bytes = json.dumps(build).encode()
    (root / "build.json").write_bytes(build_bytes)
    build_ref = {"path": "build.json", "sha256": hashlib.sha256(build_bytes).hexdigest()}
    plan = {
        "build_record": build_ref,
        "binary": build["binary"],
        "workload": {"name": "32-leaf", "leaves": 32, "text_lines_per_leaf": 12},
    }
    plan_bytes = json.dumps(plan).encode()
    (root / "plan.json").write_bytes(plan_bytes)
    plan_ref = {"path": "plan.json", "sha256": hashlib.sha256(plan_bytes).hexdigest()}
    return build_ref, plan_ref, binary_hash


class IncompleteCaptureObservationTests(unittest.TestCase):
    def test_attempted_states_without_screenshots_require_observations(self):
        for status in ("failed", "inconclusive"):
            with self.subTest(status=status), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary).resolve()
                previous_root = validator.ROOT
                validator.ROOT = root
                try:
                    build_ref, plan_ref, binary_hash = write_provenance(root)
                    doc = {
                        "schema_version": 1,
                        "kind": "rustui_p3d_visual_evidence",
                        "workload": {"name": "32-leaf", "leaves": 32},
                        "provenance": {
                            "build_record": build_ref,
                            "plan": plan_ref,
                            "binary_sha256": binary_hash,
                            "platform": "test",
                            "appearance": "light",
                            "display": "main",
                            "scale_factor": 2.0,
                            "capture_layout": {
                                "mode": "native_wide_8x4",
                                "columns": 8,
                                "window_logical_points": [1700, 1000],
                                "leaf_logical_points": [210, 220],
                                "lines_per_leaf": 12,
                            },
                        },
                        "states": {state: {"status": "not_run"}
                                   for state in validator.STATES},
                    }
                    attempted = validator.STATES[0]
                    doc["states"][attempted] = {"status": status, "screenshot": None}
                    manifest = root / "evidence.json"
                    manifest.write_text(json.dumps(doc))

                    with self.assertRaisesRegex(validator.InvalidEvidence, "missing observation"):
                        validator.validate(manifest)

                    doc["states"][attempted]["observation"] = {
                        "all_32_values_visible": False,
                        "expected_state_visible": False,
                        "ax_active": False,
                        "native_drag_active": False,
                        "reviewer": "reviewer",
                        "notes": "capture failed before a screenshot was written",
                    }
                    manifest.write_text(json.dumps(doc))
                    self.assertEqual(set(validator.validate(manifest)), set(validator.STATES))
                finally:
                    validator.ROOT = previous_root


class WindowsPathValidationTests(unittest.TestCase):
    def test_rejects_windows_drive_and_backslash_paths(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            paths = {
                "C:/capture.png": root / "C:" / "capture.png",
                "capture\\image.png": root / "capture\\image.png",
            }
            for relative, local_path in paths.items():
                with self.subTest(path=relative):
                    local_path.parent.mkdir(parents=True, exist_ok=True)
                    raw = b"synthetic screenshot\n"
                    local_path.write_bytes(raw)
                    reference = {
                        "path": relative,
                        "sha256": hashlib.sha256(raw).hexdigest(),
                    }
                    with self.assertRaises(validator.InvalidEvidence):
                        validator.artifact(root, reference, "screenshot")


class ArtifactReadBoundsTests(unittest.TestCase):
    def test_artifact_hashing_uses_bounded_reads_and_json_records_are_limited(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            contents = b"synthetic screenshot"
            (root / "capture.png").write_bytes(contents)
            reference = {
                "path": "capture.png",
                "sha256": hashlib.sha256(contents).hexdigest(),
            }
            real_fdopen = os.fdopen
            read_sizes = []

            class ReadTracker:
                def __init__(self, stream):
                    self.stream = stream

                def __enter__(self):
                    self.stream.__enter__()
                    return self

                def __exit__(self, *args):
                    return self.stream.__exit__(*args)

                def fileno(self):
                    return self.stream.fileno()

                def read(self, size=-1):
                    read_sizes.append(size)
                    return self.stream.read(size)

            with mock.patch.object(validator, "READ_CHUNK_BYTES", 3), \
                    mock.patch.object(os, "fdopen", side_effect=lambda fd, mode: ReadTracker(
                        real_fdopen(fd, mode)
                    )):
                validator.artifact(root, reference, "screenshot")
                with validator.PinnedDir.open_root(root) as pinned:
                    with self.assertRaisesRegex(validator.InvalidEvidence, "record limit"):
                        validator._read_file(pinned, "capture.png", "record", max_bytes=5)

            self.assertGreater(len(read_sizes), 1)
            self.assertTrue(all(size == 3 for size in read_sizes))


class ArtifactStabilityTests(unittest.TestCase):
    def test_rejects_screenshot_with_hard_link_outside_evidence_root(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            parent = Path(temporary)
            root = parent / "evidence"
            root.mkdir()
            outside = parent / "external-capture.png"
            screenshot = root / "capture.png"
            original = b"captured screenshot\n"
            outside.write_bytes(original)
            os.link(outside, screenshot)
            reference = {
                "path": "capture.png",
                "sha256": hashlib.sha256(original).hexdigest(),
            }

            with self.assertRaisesRegex(validator.InvalidEvidence, "hard-link"):
                validator.artifact(root, reference, "screenshot")

    def test_rejects_screenshot_path_replaced_after_open(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            screenshot = root / "capture.png"
            original = b"captured screenshot\n"
            screenshot.write_bytes(original)
            reference = {
                "path": screenshot.name,
                "sha256": hashlib.sha256(original).hexdigest(),
            }
            original_info = screenshot.stat()
            original_identity = (original_info.st_dev, original_info.st_ino)
            pinned = validator.PinnedDir.open_root(root)
            original_open_file = validator.PinnedDir.open_file
            replaced = False

            def replace_after_open(directory, relative, flags, mode=0o600):
                nonlocal replaced
                fd = original_open_file(directory, relative, flags, mode)
                if directory.fd == pinned.fd and relative == "capture.png" and not replaced:
                    replacement = root / "replacement.png"
                    replacement.write_bytes(b"replacement screenshot\n")
                    replacement.replace(screenshot)
                    replaced = True
                return fd

            try:
                with mock.patch.object(validator.PinnedDir, "open_file", autospec=True,
                                       side_effect=replace_after_open):
                    with self.assertRaisesRegex(
                            validator.InvalidEvidence, "path changed while its digest"):
                        validator.artifact(pinned, reference, "screenshot")
                self.assertTrue(replaced)
                self.assertNotEqual(
                    (screenshot.stat().st_dev, screenshot.stat().st_ino), original_identity)
            finally:
                pinned.close()


class ScaleFactorValidationTests(unittest.TestCase):
    def test_rejects_boolean_schema_version(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            previous_root = validator.ROOT
            validator.ROOT = root
            try:
                build_ref, plan_ref, binary_hash = write_provenance(root)
                doc = {
                    "schema_version": True,
                    "kind": "rustui_p3d_visual_evidence",
                    "workload": {"name": "32-leaf", "leaves": 32},
                    "provenance": {
                        "build_record": build_ref,
                        "plan": plan_ref,
                        "binary_sha256": binary_hash,
                        "platform": "test",
                        "appearance": "light",
                        "display": "main",
                        "scale_factor": 2.0,
                        "capture_layout": {
                            "mode": "native_wide_8x4",
                            "columns": 8,
                            "window_logical_points": [1700, 1000],
                            "leaf_logical_points": [210, 220],
                            "lines_per_leaf": 12,
                        },
                    },
                    "states": {state: {"status": "not_run"} for state in validator.STATES},
                }
                manifest = root / "evidence.json"
                manifest.write_text(json.dumps(doc))

                with self.assertRaisesRegex(validator.InvalidEvidence, "schema"):
                    validator.validate(manifest)
            finally:
                validator.ROOT = previous_root

    def test_rejects_nonpositive_and_nonfinite_scale_factors(self):
        for value in (0, -1, float("nan"), float("inf"), -float("inf"), False):
            with self.subTest(value=value), self.assertRaises(validator.InvalidEvidence):
                validator.require_scale_factor(value)

    def test_rejects_numeric_strings_as_scale_factors(self):
        with self.assertRaises(validator.InvalidEvidence):
            validator.require_scale_factor("2.0")

    def test_plan_must_reference_the_bound_build_record(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            previous_root = validator.ROOT
            validator.ROOT = root
            try:
                binary_hash = "a" * 64
                build = {"binary": {"path": "cache_comparison", "sha256": binary_hash}}
                build_bytes = json.dumps(build).encode()
                (root / "build.json").write_bytes(build_bytes)
                plan = {
                    "build_record": {
                        "path": "other-build.json",
                        "sha256": hashlib.sha256(build_bytes).hexdigest(),
                    },
                    "binary": {"path": "cache_comparison", "sha256": binary_hash},
                }
                plan_bytes = json.dumps(plan).encode()
                (root / "plan.json").write_bytes(plan_bytes)
                build_ref = {
                    "path": "build.json",
                    "sha256": hashlib.sha256(build_bytes).hexdigest(),
                }
                doc = {
                    "schema_version": 1,
                    "kind": "rustui_p3d_visual_evidence",
                    "workload": {"name": "32-leaf", "leaves": 32},
                    "provenance": {
                        "build_record": build_ref,
                        "plan": {
                            "path": "plan.json",
                            "sha256": hashlib.sha256(plan_bytes).hexdigest(),
                        },
                        "binary_sha256": binary_hash,
                        "platform": "test",
                        "appearance": "light",
                        "display": "main",
                        "scale_factor": 2.0,
                        "capture_layout": {
                            "mode": "native_wide_8x4",
                            "columns": 8,
                            "window_logical_points": [1700, 1000],
                            "leaf_logical_points": [210, 220],
                            "lines_per_leaf": 12,
                        },
                    },
                    "states": {state: {"status": "not_run"} for state in validator.STATES},
                }
                manifest = root / "evidence.json"
                manifest.write_text(json.dumps(doc))
                with self.assertRaisesRegex(validator.InvalidEvidence, "does not reference"):
                    validator.validate(manifest)
            finally:
                validator.ROOT = previous_root

    def test_plan_workload_must_match_visual_evidence(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            previous_root = validator.ROOT
            validator.ROOT = root
            try:
                build_ref, plan_ref, binary_hash = write_provenance(root)
                plan_path = root / plan_ref["path"]
                plan = json.loads(plan_path.read_text())
                plan["workload"] = {"name": "8-leaf", "leaves": 8, "text_lines_per_leaf": 12}
                plan_bytes = json.dumps(plan).encode()
                plan_path.write_bytes(plan_bytes)
                plan_ref = {"path": plan_ref["path"], "sha256": hashlib.sha256(plan_bytes).hexdigest()}
                doc = {
                    "schema_version": 1,
                    "kind": "rustui_p3d_visual_evidence",
                    "workload": {"name": "32-leaf", "leaves": 32},
                    "provenance": {
                        "build_record": build_ref,
                        "plan": plan_ref,
                        "binary_sha256": binary_hash,
                        "platform": "test",
                        "appearance": "light",
                        "display": "main",
                        "scale_factor": 2.0,
                        "capture_layout": {
                            "mode": "native_wide_8x4",
                            "columns": 8,
                            "window_logical_points": [1700, 1000],
                            "leaf_logical_points": [210, 220],
                            "lines_per_leaf": 12,
                        },
                    },
                    "states": {state: {"status": "not_run"} for state in validator.STATES},
                }
                evidence = root / "evidence.json"
                evidence.write_text(json.dumps(doc))

                with self.assertRaisesRegex(validator.InvalidEvidence, "plan workload"):
                    validator.validate(evidence)
            finally:
                validator.ROOT = previous_root

    def test_rejects_binary_drift_after_build_and_plan_records(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            previous_root = validator.ROOT
            validator.ROOT = root
            try:
                build_ref, plan_ref, binary_hash = write_provenance(root)
                (root / "cache_comparison").write_bytes(b"replaced binary")
                doc = {
                    "schema_version": 1,
                    "kind": "rustui_p3d_visual_evidence",
                    "workload": {"name": "32-leaf", "leaves": 32},
                    "provenance": {
                        "build_record": build_ref,
                        "plan": plan_ref,
                        "binary_sha256": binary_hash,
                        "platform": "test",
                        "appearance": "light",
                        "display": "main",
                        "scale_factor": 2.0,
                        "capture_layout": {
                            "mode": "native_wide_8x4",
                            "columns": 8,
                            "window_logical_points": [1700, 1000],
                            "leaf_logical_points": [210, 220],
                            "lines_per_leaf": 12,
                        },
                    },
                    "states": {state: {"status": "not_run"} for state in validator.STATES},
                }
                manifest = root / "evidence.json"
                manifest.write_text(json.dumps(doc))
                with self.assertRaisesRegex(validator.InvalidEvidence, "provenance.binary hash mismatch"):
                    validator.validate(manifest)
            finally:
                validator.ROOT = previous_root


    def test_rejects_non_string_platform_provenance(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            previous_root = validator.ROOT
            validator.ROOT = root
            try:
                build_ref, plan_ref, binary_hash = write_provenance(root)
                doc = {
                    "schema_version": 1,
                    "kind": "rustui_p3d_visual_evidence",
                    "workload": {"name": "32-leaf", "leaves": 32},
                    "provenance": {
                        "build_record": build_ref,
                        "plan": plan_ref,
                        "binary_sha256": binary_hash,
                        "platform": 7,
                        "appearance": "light",
                        "display": "main",
                        "scale_factor": 2.0,
                        "capture_layout": {
                            "mode": "native_wide_8x4",
                            "columns": 8,
                            "window_logical_points": [1700, 1000],
                            "leaf_logical_points": [210, 220],
                            "lines_per_leaf": 12,
                        },
                    },
                    "states": {state: {"status": "not_run"} for state in validator.STATES},
                }
                manifest = root / "evidence.json"
                manifest.write_text(json.dumps(doc))
                with self.assertRaisesRegex(validator.InvalidEvidence, "provenance.platform"):
                    validator.validate(manifest)
            finally:
                validator.ROOT = previous_root


class ExactShapeValidationTests(unittest.TestCase):
    def test_rejects_duplicate_keys_in_visual_and_provenance_json(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            previous_root = validator.ROOT
            validator.ROOT = root
            try:
                (root / "cache_comparison").write_bytes(b"binary")
                binary_hash = hashlib.sha256(b"binary").hexdigest()
                build_raw = json.dumps({"binary": {"path": "cache_comparison", "sha256": binary_hash}})
                duplicate_build = (
                    build_raw[:-1] + "," + build_raw[1:-1] + "}"
                ).encode()
                (root / "build.json").write_bytes(duplicate_build)
                build_ref = {
                    "path": "build.json",
                    "sha256": hashlib.sha256(duplicate_build).hexdigest(),
                }

                with self.assertRaisesRegex(validator.InvalidEvidence, "duplicate JSON key: binary"):
                    validator.artifact_json(root, build_ref, "provenance.build_record")

                manifest = root / "evidence.json"
                manifest.write_text('{"schema_version": 1, "schema_version": 1}')
                with self.assertRaisesRegex(validator.InvalidEvidence, "duplicate JSON key: schema_version"):
                    validator.validate(manifest)
            finally:
                validator.ROOT = previous_root

    def test_rejects_non_utf8_provenance_artifact_as_invalid_evidence(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            previous_root = validator.ROOT
            validator.ROOT = root
            try:
                raw = b'{"record": "invalid utf-8: \xff"}'
                (root / "build.json").write_bytes(raw)
                with self.assertRaisesRegex(
                    validator.InvalidEvidence,
                    "provenance.build_record must contain valid UTF-8",
                ):
                    validator.artifact_json(
                        root,
                        {"path": "build.json", "sha256": hashlib.sha256(raw).hexdigest()},
                        "provenance.build_record",
                    )
            finally:
                validator.ROOT = previous_root

    def test_rejects_nonstandard_json_constants_in_visual_and_provenance(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            manifest = root / "evidence.json"
            manifest.write_text('{"schema_version": 1, "extra": NaN}')
            with self.assertRaisesRegex(
                validator.InvalidEvidence, "non-standard JSON numeric constant: NaN"
            ):
                validator.validate(manifest)

            provenance = root / "build.json"
            raw = b'{"binary": Infinity}'
            provenance.write_bytes(raw)
            reference = {
                "path": provenance.name,
                "sha256": hashlib.sha256(raw).hexdigest(),
            }
            with self.assertRaisesRegex(
                validator.InvalidEvidence, "non-standard JSON numeric constant: Infinity"
            ):
                validator.artifact_json(root, reference, "provenance.build_record")

    def test_rejects_float_overflow_numbers_in_visual_and_provenance(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            manifest = root / "evidence.json"
            manifest.write_text('{"schema_version": 1, "extra": 1e10000}')
            with self.assertRaisesRegex(
                validator.InvalidEvidence, "JSON number is outside the finite float range"
            ):
                validator.validate(manifest)

            provenance = root / "build.json"
            raw = b'{"binary": 1e10000}'
            provenance.write_bytes(raw)
            reference = {
                "path": provenance.name,
                "sha256": hashlib.sha256(raw).hexdigest(),
            }
            with self.assertRaisesRegex(
                validator.InvalidEvidence, "JSON number is outside the finite float range"
            ):
                validator.artifact_json(root, reference, "provenance.build_record")

    def test_rejects_non_utf8_manifest_as_invalid_evidence(self):
        with tempfile.TemporaryDirectory() as temporary:
            evidence = Path(temporary) / "evidence.json"
            evidence.write_bytes(b"{\"schema_version\": 1, \"invalid\": \xff}")

            with self.assertRaisesRegex(
                validator.InvalidEvidence, "P3D visual evidence must be valid UTF-8"
            ):
                validator.validate(evidence)

    def test_rejects_integral_floats_in_fixed_integer_fields(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            previous_root = validator.ROOT
            validator.ROOT = root
            try:
                build_ref, plan_ref, binary_hash = write_provenance(root)
                doc = {
                    "schema_version": 1,
                    "kind": "rustui_p3d_visual_evidence",
                    "workload": {"name": "32-leaf", "leaves": 32},
                    "provenance": {
                        "build_record": build_ref,
                        "plan": plan_ref,
                        "binary_sha256": binary_hash,
                        "platform": "test",
                        "appearance": "light",
                        "display": "main",
                        "scale_factor": 2.0,
                        "capture_layout": {
                            "mode": "native_wide_8x4",
                            "columns": 8,
                            "window_logical_points": [1700, 1000],
                            "leaf_logical_points": [210, 220],
                            "lines_per_leaf": 12,
                        },
                    },
                    "states": {
                        state: {"status": "not_run"} for state in validator.STATES
                    },
                }
                evidence = root / "evidence.json"
                cases = (
                    ("workload.leaves", lambda value: value["workload"].update(leaves=32.0)),
                    ("capture_layout.columns", lambda value: value["provenance"]["capture_layout"].update(columns=8.0)),
                )
                for field, make_float in cases:
                    with self.subTest(field=field):
                        candidate = json.loads(json.dumps(doc))
                        make_float(candidate)
                        evidence.write_text(json.dumps(candidate))
                        with self.assertRaises(validator.InvalidEvidence):
                            validator.validate(evidence)
            finally:
                validator.ROOT = previous_root

    def test_rejects_non_object_document_root(self):
        with tempfile.TemporaryDirectory() as temporary:
            evidence = Path(temporary) / "evidence.json"
            evidence.write_text("[]")

            with self.assertRaisesRegex(
                validator.InvalidEvidence, "P3D visual evidence must be a JSON object"
            ):
                validator.validate(evidence)

    def test_rejects_non_string_state_status_without_type_error(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            previous_root = validator.ROOT
            validator.ROOT = root
            try:
                build_ref, plan_ref, binary_hash = write_provenance(root)
                doc = {
                    "schema_version": 1,
                    "kind": "rustui_p3d_visual_evidence",
                    "workload": {"name": "32-leaf", "leaves": 32},
                    "provenance": {
                        "build_record": build_ref,
                        "plan": plan_ref,
                        "binary_sha256": binary_hash,
                        "platform": "test",
                        "appearance": "light",
                        "display": "main",
                        "scale_factor": 2.0,
                        "capture_layout": {
                            "mode": "native_wide_8x4",
                            "columns": 8,
                            "window_logical_points": [1700, 1000],
                            "leaf_logical_points": [210, 220],
                            "lines_per_leaf": 12,
                        },
                    },
                    "states": {
                        state: {"status": "not_run"} for state in validator.STATES
                    },
                }
                doc["states"][validator.STATES[0]]["status"] = []
                evidence = root / "evidence.json"
                evidence.write_text(json.dumps(doc))

                with self.assertRaisesRegex(
                    validator.InvalidEvidence, f"invalid status for {validator.STATES[0]}"
                ):
                    validator.validate(evidence)
            finally:
                validator.ROOT = previous_root


class ArtifactPathValidationTests(unittest.TestCase):
    def test_rejects_embedded_nul_path_without_value_error(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            artifact = json.loads('{"path":"\\u0000","sha256":"' + "0" * 64 + '"}')

            with self.assertRaisesRegex(
                validator.InvalidEvidence, "artifact path cannot be resolved"
            ):
                validator.artifact(root, artifact, "artifact")


class ScreenshotDistinctnessTests(unittest.TestCase):
    def test_rejects_screenshot_for_not_run_state(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            previous_root = validator.ROOT
            validator.ROOT = root
            try:
                build_ref, plan_ref, binary_hash = write_provenance(root)
                (root / "attempt.png").write_bytes(b"image")
                screenshot_bytes = b"image"
                doc = {
                    "schema_version": 1,
                    "kind": "rustui_p3d_visual_evidence",
                    "workload": {"name": "32-leaf", "leaves": 32},
                    "provenance": {
                        "build_record": build_ref,
                        "plan": plan_ref,
                        "binary_sha256": binary_hash,
                        "platform": "test",
                        "appearance": "light",
                        "display": "main",
                        "scale_factor": 2.0,
                        "capture_layout": {
                            "mode": "native_wide_8x4",
                            "columns": 8,
                            "window_logical_points": [1700, 1000],
                            "leaf_logical_points": [210, 220],
                            "lines_per_leaf": 12,
                        },
                    },
                    "states": {state: {"status": "not_run"} for state in validator.STATES},
                }
                doc["states"][validator.STATES[0]]["screenshot"] = {
                    "path": "attempt.png",
                    "sha256": hashlib.sha256(screenshot_bytes).hexdigest(),
                }
                manifest = root / "evidence.json"
                manifest.write_text(json.dumps(doc))
                with self.assertRaisesRegex(validator.InvalidEvidence, "not_run state"):
                    validator.validate(manifest)

                state = doc["states"][validator.STATES[0]]
                state["status"] = "captured"
                state["screenshot"] = build_ref
                state["observation"] = {
                    "all_32_values_visible": True,
                    "expected_state_visible": True,
                    "ax_active": False,
                    "native_drag_active": False,
                    "reviewer": "reviewer",
                    "notes": "capture review",
                }
                manifest.write_text(json.dumps(doc))
                with self.assertRaisesRegex(
                    validator.InvalidEvidence, "screenshot must be distinct from provenance artifacts"
                ):
                    validator.validate(manifest)
            finally:
                validator.ROOT = previous_root


class InheritedCandidateDescriptorTests(unittest.TestCase):
    def test_candidate_path_swap_uses_inherited_candidate_root(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo_path = Path(temporary)
            candidate_id = "run-a"
            candidates = repo_path / "target/headstart/candidates"
            candidate_path = candidates / candidate_id
            evidence_dir = candidate_path / "evidence"
            evidence_dir.mkdir(parents=True)

            binary = b"candidate binary"
            binary_hash = hashlib.sha256(binary).hexdigest()
            binary_ref = f"target/headstart/candidates/{candidate_id}/evidence/cache_comparison"
            (evidence_dir / "cache_comparison").write_bytes(binary)
            build = {"binary": {"path": binary_ref, "sha256": binary_hash}}
            build_raw = json.dumps(build).encode()
            (evidence_dir / "build.json").write_bytes(build_raw)
            build_ref = {
                "path": f"target/headstart/candidates/{candidate_id}/evidence/build.json",
                "sha256": hashlib.sha256(build_raw).hexdigest(),
            }
            plan = {
                "build_record": build_ref,
                "binary": build["binary"],
                "workload": {"name": "32-leaf", "leaves": 32,
                             "text_lines_per_leaf": 12},
            }
            plan_raw = json.dumps(plan).encode()
            (evidence_dir / "plan.json").write_bytes(plan_raw)
            plan_ref = {
                "path": f"target/headstart/candidates/{candidate_id}/evidence/plan.json",
                "sha256": hashlib.sha256(plan_raw).hexdigest(),
            }
            doc = {
                "schema_version": 1,
                "kind": "rustui_p3d_visual_evidence",
                "workload": {"name": "32-leaf", "leaves": 32},
                "provenance": {
                    "build_record": build_ref,
                    "plan": plan_ref,
                    "binary_sha256": binary_hash,
                    "platform": "test",
                    "appearance": "light",
                    "display": "main",
                    "scale_factor": 2.0,
                    "capture_layout": {
                        "mode": "native_wide_8x4", "columns": 8,
                        "window_logical_points": [1700, 1000],
                        "leaf_logical_points": [210, 220], "lines_per_leaf": 12,
                    },
                },
                "states": {state: {"status": "not_run"} for state in validator.STATES},
            }
            (evidence_dir / "visual.json").write_text(json.dumps(doc))

            repo = validator.PinnedDir.open_root(repo_path)
            parent = validator.PinnedDir.open_child(repo, "target/headstart/candidates")
            candidate = validator.PinnedDir.open_child(parent, candidate_id)
            try:
                original_open_file = validator.PinnedDir.open_file
                candidate_info = os.fstat(candidate.fd)
                swapped = False

                def swap_after_manifest_open(directory, relative, flags, mode=0o600):
                    nonlocal swapped
                    fd = original_open_file(directory, relative, flags, mode)
                    directory_info = os.fstat(directory.fd)
                    if ((directory_info.st_dev, directory_info.st_ino)
                            == (candidate_info.st_dev, candidate_info.st_ino)
                            and relative == "evidence/visual.json" and not swapped):
                        detached = candidates / "detached-run-a"
                        candidate_path.rename(detached)
                        (candidate_path / "evidence").mkdir(parents=True)
                        (candidate_path / "evidence/visual.json").write_text("{}")
                        swapped = True
                    return fd

                env = {
                    "RUSTUI_P3_REPO_ROOT_FD": str(repo.fd),
                    "RUSTUI_P3_CANDIDATE_PARENT_FD": str(parent.fd),
                    "RUSTUI_P3_CANDIDATE_ROOT_FD": str(candidate.fd),
                    "RUSTUI_P3_CANDIDATE_ID": candidate_id,
                }
                with mock.patch.object(validator.PinnedDir, "open_file", autospec=True,
                                       side_effect=swap_after_manifest_open), \
                     mock.patch.dict(os.environ, env), \
                     mock.patch.object(sys, "argv", [str(SCRIPT),
                         f"target/headstart/candidates/{candidate_id}/evidence/visual.json"]):
                    self.assertEqual(validator.main(), 0)
                self.assertTrue(swapped)
                self.assertTrue((candidates / "detached-run-a/evidence/visual.json").is_file())
            finally:
                candidate.close()
                parent.close()
                repo.close()

    def test_rejects_same_screenshot_for_two_incomplete_states(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            previous_root = validator.ROOT
            validator.ROOT = root
            try:
                build_ref, plan_ref, binary_hash = write_provenance(root)
                (root / "attempt.png").write_bytes(b"image")
                screenshot = {
                    "path": "attempt.png",
                    "sha256": hashlib.sha256(b"image").hexdigest(),
                }
                doc = {
                    "schema_version": 1,
                    "kind": "rustui_p3d_visual_evidence",
                    "workload": {"name": "32-leaf", "leaves": 32},
                    "provenance": {
                        "build_record": build_ref,
                        "plan": plan_ref,
                        "binary_sha256": binary_hash,
                        "platform": "test",
                        "appearance": "light",
                        "display": "main",
                        "scale_factor": 2.0,
                        "capture_layout": {
                            "mode": "native_wide_8x4",
                            "columns": 8,
                            "window_logical_points": [1700, 1000],
                            "leaf_logical_points": [210, 220],
                            "lines_per_leaf": 12,
                        },
                    },
                    "states": {state: {"status": "not_run"} for state in validator.STATES},
                }
                doc["states"][validator.STATES[0]] = {"status": "failed", "screenshot": screenshot}
                doc["states"][validator.STATES[1]] = {"status": "inconclusive", "screenshot": screenshot}
                manifest = root / "evidence.json"
                manifest.write_text(json.dumps(doc))
                with self.assertRaisesRegex(validator.InvalidEvidence, "missing observation"):
                    validator.validate(manifest)
                observation = {
                    "all_32_values_visible": False,
                    "expected_state_visible": False,
                    "ax_active": False,
                    "native_drag_active": False,
                    "reviewer": "reviewer",
                    "notes": "capture attempt",
                }
                doc["states"][validator.STATES[0]]["observation"] = observation
                doc["states"][validator.STATES[1]]["observation"] = observation
                manifest.write_text(json.dumps(doc))
                with self.assertRaisesRegex(validator.InvalidEvidence, "distinct screenshot"):
                    validator.validate(manifest)
            finally:
                validator.ROOT = previous_root


if __name__ == "__main__":
    unittest.main()
