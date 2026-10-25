"""Focused synthetic tests for read-only P3L/P3M evidence aggregation."""

from __future__ import annotations

import hashlib
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import struct
import sys
import tempfile
import unittest
from unittest.mock import patch
import zlib


SCRIPT = Path(__file__).resolve().parents[1] / "aggregate_candidate_evidence.py"
sys.path.insert(0, str(SCRIPT.parent))
import aggregate_candidate_evidence as AGGREGATOR


def digest(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def png_fixture() -> bytes:
    signature = b"\x89PNG\r\n\x1a\n"

    def chunk(kind: bytes, data: bytes) -> bytes:
        body = kind + data
        return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body))

    return (signature + chunk(b"IHDR", struct.pack(">IIBBBBB", 1, 1, 8, 6, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(b"\0\0\0\0\xff")) + chunk(b"IEND", b""))


class AggregateCandidateEvidenceTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.evidence = self.root / "evidence"
        self.evidence.mkdir()

        self.source_hash = "a" * 64
        self.candidate_id = "candidate-one"
        scene_relative = Path("path-sources/gpui-pre-0.3.9/src/scene.rs")
        snapshots = {}
        source_hash_snapshots = {}
        self.snapshot_tree_hashes = {}
        self.scene_identity = {}
        for side in ("baseline", "candidate"):
            source_root = self.root / side / "source"
            scene_path = source_root / scene_relative
            scene_path.parent.mkdir(parents=True)
            scene_raw = f"{side} scene fixture\n".encode()
            scene_path.write_bytes(scene_raw)
            scene_hash = digest(scene_raw)
            snapshots[side] = {
                "source": str(source_root),
                "files": {scene_relative.as_posix(): scene_hash},
            }
            files = {scene_relative.as_posix(): scene_hash}
            tree_hash = digest(json.dumps(files, sort_keys=True).encode())
            self.snapshot_tree_hashes[side] = tree_hash
            source_hash_snapshots[side] = {
                "source": f"{side}/source", "files": files, "tree_sha256": tree_hash,
            }
            self.scene_identity[f"{side}_scene_path"] = (
                scene_path.relative_to(self.root).as_posix()
            )
            self.scene_identity[f"{side}_scene_sha256"] = scene_hash
        self.manifest_raw = json.dumps({
            "schema_version": 1, "candidate_id": self.candidate_id,
            "snapshots": snapshots,
        }, sort_keys=True).encode()
        (self.root / "manifest.json").write_bytes(self.manifest_raw)
        self.manifest_hash = digest(self.manifest_raw)

        self.source_hash_manifest_raw = json.dumps({
            "schema_version": 1,
            "candidate_id": self.candidate_id,
            "manifest_sha256": self.manifest_hash,
            "source_snapshot_manifest_sha256": self.source_hash,
            "snapshots": source_hash_snapshots,
        }, sort_keys=True).encode()
        (self.root / "source-hash-manifest.json").write_bytes(self.source_hash_manifest_raw)
        self.source_hash_manifest_hash = digest(self.source_hash_manifest_raw)

        intake = {
            "schema_version": 1,
            "approved": True,
            "approval_reference": "synthetic-only",
            "candidate_id": self.candidate_id,
            "source_candidate_id": "source-one",
            "source_manifest": {"path": "not-opened.json", "sha256": self.source_hash},
            "patch_plan": None,
            "purpose": "synthetic validator fixture",
        }
        self.intake_raw = (json.dumps(intake, indent=2, sort_keys=True) + "\n").encode()
        (self.root / "approved-intake.json").write_bytes(self.intake_raw)

        primary_log = b"synthetic P3F input log\n"
        (self.evidence / "01-input.log").write_bytes(primary_log)
        self.gates = {
            "build": "not_run", "correctness": "not_run",
            "gui": "not_run", "performance": "not_run",
        }
        self.p3f = {
            "manifest_sha256": self.manifest_hash,
            "command": ["python", "pipeline"],
            "exit_status": 0,
            "log": {"path": "evidence/01-input.log", "sha256": digest(primary_log)},
            "evidence": [
                {"path": "approved-intake.json", "sha256": digest(self.intake_raw)},
                {"path": "manifest.json", "sha256": self.manifest_hash},
                {"path": "source-hash-manifest.json", "sha256": self.source_hash_manifest_hash},
            ],
            "provenance": {
                "approved_intake_sha256": digest(self.intake_raw),
                "source_candidate_id": "source-one",
            },
            "gates": dict(self.gates),
        }
        self.write_p3f()

        p3f_stdout = json.dumps({
            "schema_version": 1,
            "status": "valid",
            "manifest_sha256": self.manifest_hash,
            "gate_states": self.gates,
            "errors": [],
        }, sort_keys=True).encode() + b"\n"
        (self.evidence / "08-P3F.log").write_bytes(p3f_stdout)
        event = {
            "schema_version": 1,
            "step": "P3F-evidence-record",
            "status": "passed",
            "exit_status": 0,
            "command": ["python", str(SCRIPT.parent / "validate_candidate_evidence.py")],
            "log": {"path": "08-P3F.log", "sha256": digest(p3f_stdout)},
            "manifest_sha256": self.manifest_hash,
            "recorded_at": "2026-10-20T00:00:00+00:00",
        }
        event_raw = (json.dumps(event, indent=2, sort_keys=True) + "\n").encode()
        (self.evidence / "event-01-P3F.json").write_bytes(event_raw)
        self.event = {
            **event,
            "event_record": {"path": "event-01-P3F.json", "sha256": digest(event_raw)},
        }
        self.result = {
            "schema_version": 1,
            "candidate_id": self.candidate_id,
            "manifest_sha256": self.manifest_hash,
            "status": "partial",
            "scope": "synthetic fixture",
            "events": [self.event],
            "gates": {**self.gates, "promotion": "not_run"},
            "source_hash_manifest_sha256": self.source_hash_manifest_hash,
        }
        self.result_path = self.evidence / "pipeline-result.json"
        self.write_result()

    def write_p3f(self) -> None:
        (self.evidence / "P3F-record.json").write_text(
            json.dumps(self.p3f, indent=2, sort_keys=True) + "\n", encoding="utf-8",
        )

    def write_result(self) -> None:
        self.result_path.write_text(json.dumps(self.result, indent=2, sort_keys=True) + "\n",
                                    encoding="utf-8")

    def invoke(self, p3f_path: Path | None = None,
               followups: list[Path] | None = None) -> tuple[subprocess.CompletedProcess[str], dict]:
        command = [sys.executable, str(SCRIPT), str(self.result_path), "--root", str(self.root)]
        if p3f_path is not None:
            command.extend(("--p3f-record", str(p3f_path)))
        for followup in followups or []:
            command.extend(("--followup-summary", str(followup)))
        process = subprocess.run(
            command,
            capture_output=True, text=True, check=False,
        )
        return process, json.loads(process.stdout)

    def write_followups(self) -> tuple[Path, Path]:
        event_dir = self.evidence / "followup-events"
        log_dir = self.evidence / "followup-logs"
        event_dir.mkdir(exist_ok=True)
        log_dir.mkdir(exist_ok=True)
        m274_events = {}
        event_steps = {
            "baseline_check": "P3 baseline workspace all-targets check",
            "baseline_build": "P3 baseline workspace all-targets build",
            "candidate_check": "candidate-workspace-all-targets-check",
            "candidate_build": "candidate-workspace-all-targets-build",
        }
        for name in ("baseline_check", "baseline_build", "candidate_check", "candidate_build"):
            log_path = log_dir / f"{name}.log"
            log_raw = f"{name} passed\n".encode()
            log_path.write_bytes(log_raw)
            event_path = event_dir / f"{name}.json"
            event = {
                "schema_version": 1,
                "status": "passed",
                "exit_status": 0,
                "step": event_steps[name],
                "command": ["./scripts/cargo-headstart", "check" if name.endswith("check") else "build",
                            "--workspace", "--all-targets", "--locked", "--offline"],
                "log": {"path": log_path.relative_to(self.root).as_posix(),
                        "sha256": digest(log_raw)},
            }
            if name.startswith("baseline_"):
                event["candidate_id"] = self.candidate_id
            else:
                event["manifest_sha256"] = self.manifest_hash
            event_raw = (json.dumps(event, sort_keys=True) + "\n").encode()
            event_path.write_bytes(event_raw)
            m274_events[name] = {
                "event_path": event_path.relative_to(self.root).as_posix(),
                "event_sha256": digest(event_raw),
                "log_path": log_path.relative_to(self.root).as_posix(),
                "log_sha256": digest(log_raw),
                "exit_status": 0,
            }
        m274 = {
            "schema_version": 1,
            "milestone": "M274",
            "candidate_id": self.candidate_id,
            "manifest_sha256": self.manifest_hash,
            "gate": "build",
            "status": "passed",
            "events": m274_events,
            "gates": {"build": "passed", "correctness": "not_run", "gui": "not_run",
                      "performance": "not_run", "promotion": "not_run"},
        }
        m274_path = self.evidence / "m274-build-gate-summary.json"
        m274_raw = (json.dumps(m274, sort_keys=True) + "\n").encode()
        m274_path.write_bytes(m274_raw)

        m276_log = log_dir / "m276-correctness.log"
        m276_log_raw = b"10 passed; 0 failed\n"
        m276_log.write_bytes(m276_log_raw)
        m276_event_path = event_dir / "m276-correctness.json"
        m276_event = {
            "milestone": "M276",
            "status": "passed",
            "candidate_id": self.candidate_id,
            "manifest_sha256": self.manifest_hash,
            "exit_status": 0,
            "sha256": {"raw_log": digest(m276_log_raw)},
        }
        m276_event_raw = (json.dumps(m276_event, sort_keys=True) + "\n").encode()
        m276_event_path.write_bytes(m276_event_raw)
        m276 = {
            "schema_version": 1,
            "milestone": "M276",
            "candidate_id": self.candidate_id,
            "manifest_sha256": self.manifest_hash,
            "gate": "correctness",
            "status": "passed",
            "build_summary_path": m274_path.relative_to(self.root).as_posix(),
            "build_summary_sha256": digest(m274_raw),
            "success_event_path": m276_event_path.relative_to(self.root).as_posix(),
            "success_event_sha256": digest(m276_event_raw),
            "success_log_path": m276_log.relative_to(self.root).as_posix(),
            "success_log_sha256": digest(m276_log_raw),
            "source_identity": self.scene_identity,
            "gates": {"build": "passed", "correctness": "passed", "gui": "not_run",
                      "performance": "not_run", "promotion": "not_run"},
        }
        m276_path = self.evidence / "m276-build-correctness-gate-summary.json"
        m276_path.write_text(json.dumps(m276, sort_keys=True) + "\n", encoding="utf-8")
        return m274_path, m276_path

    def write_gui_followup(self, gate_states: dict[str, str] | None = None) -> Path:
        m274_path, _ = self.write_followups()
        snapshots = {}
        for side in ("baseline", "candidate"):
            screenshot_path = self.evidence / f"{side}-window.png"
            screenshot_raw = png_fixture()
            screenshot_path.write_bytes(screenshot_raw)
            accessibility_path = self.evidence / f"{side}-ax.json"
            accessibility_raw = json.dumps({"side": side, "role": "canvas"}).encode() + b"\n"
            accessibility_path.write_bytes(accessibility_raw)
            snapshots[side] = {
                "source_tree_sha256": self.snapshot_tree_hashes[side],
                "screenshot_state": "passed",
                "screenshot": {"path": screenshot_path.relative_to(self.root).as_posix(),
                               "sha256": digest(screenshot_raw)},
                "accessibility_state": "passed",
                "accessibility_tree": {
                    "path": accessibility_path.relative_to(self.root).as_posix(),
                    "sha256": digest(accessibility_raw),
                },
            }
        summary = {
            "schema_version": 1,
            "milestone": "M287",
            "candidate_id": self.candidate_id,
            "manifest_sha256": self.manifest_hash,
            "source_hash_manifest_sha256": self.source_hash_manifest_hash,
            "build_summary_path": m274_path.relative_to(self.root).as_posix(),
            "build_summary_sha256": digest(m274_path.read_bytes()),
            "gate_states": gate_states or {
                "visual": "passed", "accessibility": "passed", "gui": "passed",
            },
            "snapshots": snapshots,
        }
        summary_path = self.evidence / "m287-gui-gate-summary.json"
        summary_path.write_text(json.dumps(summary, sort_keys=True) + "\n", encoding="utf-8")
        return summary_path

    def rewrite_m274_event(self, summary_path: Path, slot: str, **changes: str | int | None) -> None:
        summary = json.loads(summary_path.read_text())
        record = summary["events"][slot]
        event_path = self.root / record["event_path"]
        event = json.loads(event_path.read_text())
        for field, value in changes.items():
            if value is None:
                event.pop(field, None)
            else:
                event[field] = value
        event_raw = (json.dumps(event, sort_keys=True) + "\n").encode()
        event_path.write_bytes(event_raw)
        record["event_sha256"] = digest(event_raw)
        summary_path.write_text(json.dumps(summary, sort_keys=True) + "\n", encoding="utf-8")

    def test_m276_source_identity_must_match_snapshot_and_file(self) -> None:
        m274, m276 = self.write_followups()
        candidate_scene = self.root / self.scene_identity["candidate_scene_path"]
        candidate_scene.write_text("changed after correctness run\n", encoding="utf-8")
        process, report = self.invoke(followups=[m274, m276])
        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("candidate scene source SHA-256 does not match file" in error
                            for error in report["errors"]))
        self.assertTrue(all(value == "not_run" for value in report["gate_states"].values()))

    def test_m276_source_identity_rejects_path_outside_snapshot_source(self) -> None:
        m274, m276 = self.write_followups()
        summary = json.loads(m276.read_text(encoding="utf-8"))
        summary["source_identity"]["candidate_scene_path"] = "candidate/scene.rs"
        m276.write_text(json.dumps(summary) + "\n", encoding="utf-8")
        process, report = self.invoke(followups=[m274, m276])
        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("does not belong to its manifest source root" in error
                            for error in report["errors"]))
        self.assertTrue(all(value == "not_run" for value in report["gate_states"].values()))

    def test_pipeline_candidate_id_must_match_safe_identifier_contract(self) -> None:
        for candidate_id in ("../outside", "candidate/one", "bad id", "x" * 65):
            with self.subTest(candidate_id=candidate_id):
                self.result["candidate_id"] = candidate_id
                self.write_result()

                process, report = self.invoke()

                self.assertEqual(process.returncode, 1)
                self.assertEqual(report["status"], "invalid")
                self.assertTrue(any("pipeline result candidate_id is invalid" in error
                                    for error in report["errors"]))

    def test_pipeline_scope_must_be_a_string(self) -> None:
        self.result["scope"] = {"untrusted": "object"}
        self.write_result()

        process, report = self.invoke()

        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("pipeline result scope must be a string" in error
                            for error in report["errors"]))

    def test_consistent_records_report_only_observed_not_run_gates(self) -> None:
        process, report = self.invoke()
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(report["status"], "consistent")
        self.assertEqual(report["gate_states"], {**self.gates, "promotion": "not_run"})
        self.assertEqual(report["event_count"], 1)
        self.assertNotIn("accepted", report)
        self.assertNotIn("promoted", report)

    def test_pipeline_event_rejects_float_overflow(self) -> None:
        event = {key: value for key, value in self.event.items() if key != "event_record"}
        event["recorded_at"] = "NONFINITE_FLOAT_SENTINEL"
        event_text = json.dumps(event, indent=2, sort_keys=True) + "\n"
        event_raw = event_text.replace('"NONFINITE_FLOAT_SENTINEL"', "1e999").encode()
        (self.evidence / "event-01-P3F.json").write_bytes(event_raw)
        self.result["events"] = [{
            **event,
            "event_record": {"path": "event-01-P3F.json", "sha256": digest(event_raw)},
        }]
        result_text = json.dumps(self.result, indent=2, sort_keys=True) + "\n"
        self.result_path.write_text(
            result_text.replace('"NONFINITE_FLOAT_SENTINEL"', "1e999"), encoding="utf-8",
        )

        report = AGGREGATOR.aggregate(self.result_path, self.root)

        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("outside the finite float range" in error
                            for error in report["errors"]), report["errors"])

    def test_command_must_match_event_step_validator(self) -> None:
        event = {key: value for key, value in self.event.items() if key != "event_record"}
        event["command"] = ["python", "/tmp/other/validate_candidate_evidence.py"]
        event_raw = (json.dumps(event, indent=2, sort_keys=True) + "\n").encode()
        (self.evidence / "event-01-P3F.json").write_bytes(event_raw)
        self.result["events"][0] = {
            **event,
            "event_record": {
                "path": "event-01-P3F.json",
                "sha256": digest(event_raw),
            },
        }
        self.write_result()

        process, report = self.invoke()

        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("command does not identify the validator" in error
                            for error in report["errors"]))

    def test_pipeline_result_record_size_is_bounded(self) -> None:
        self.result_path.write_bytes(b" " * (1024 * 1024 + 1))

        process, report = self.invoke()

        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("pipeline result exceeds the 1048576-byte record limit" in error
                            for error in report["errors"]))

    def test_pipeline_event_record_size_is_bounded(self) -> None:
        event = dict(self.event)
        event["padding"] = "x" * (1024 * 1024)
        event_raw = (json.dumps(event, indent=2) + "\n").encode()
        (self.evidence / "event-01-P3F.json").write_bytes(event_raw)
        self.result["events"][0]["event_record"]["sha256"] = digest(event_raw)
        self.write_result()

        process, report = self.invoke()

        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("events[0].event_record exceeds the 1048576-byte record limit" in error
                            for error in report["errors"]), report["errors"])

    def test_p3f_validator_output_record_size_is_bounded(self) -> None:
        log_path = self.evidence / "08-P3F.log"
        output = json.loads(log_path.read_text())
        output["padding"] = "x" * (1024 * 1024)
        log_raw = (json.dumps(output, sort_keys=True) + "\n").encode()
        log_path.write_bytes(log_raw)

        event = self.result["events"][0]
        event["log"]["sha256"] = digest(log_raw)
        event_raw = (json.dumps(
            {key: value for key, value in event.items() if key != "event_record"},
            indent=2, sort_keys=True,
        ) + "\n").encode()
        (self.evidence / "event-01-P3F.json").write_bytes(event_raw)
        event["event_record"]["sha256"] = digest(event_raw)
        self.write_result()

        process, report = self.invoke()

        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("events[0].log exceeds the 1048576-byte record limit" in error
                            for error in report["errors"]), report["errors"])

    def test_command_event_log_size_is_bounded(self) -> None:
        log_raw = b"x" * 17
        log_path = self.evidence / "oversized-command.log"
        log_path.write_bytes(log_raw)
        event = {
            "schema_version": 1,
            "step": "P3E-patch-plan",
            "status": "not_run",
            "exit_status": None,
            "command": ["python", "validate_patch_plan.py"],
            "log": {"path": log_path.name, "sha256": digest(log_raw)},
            "manifest_sha256": self.source_hash,
            "recorded_at": "2026-10-23T00:00:00+00:00",
        }
        event_raw = (json.dumps(event, sort_keys=True) + "\n").encode()
        event_path = self.evidence / "event-02-P3E.json"
        event_path.write_bytes(event_raw)
        self.result["events"].append({
            **event,
            "event_record": {"path": event_path.name, "sha256": digest(event_raw)},
        })
        self.write_result()

        with patch.object(AGGREGATOR, "MAX_COMMAND_LOG_BYTES", 16):
            report = AGGREGATOR.aggregate(self.result_path, self.root)

        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any(
            "events[1].log exceeds the 16-byte record limit" in error
            for error in report["errors"]
        ), report["errors"])

    def test_p3f_artifact_reread_enforces_aggregate_byte_budget(self) -> None:
        log_path = self.evidence / "01-input.log"
        grown = b"x" * 5

        def grow_after_validation(record: dict, root: Path) -> dict:
            log_path.write_bytes(grown)
            return {"status": "valid"}

        with (patch.object(AGGREGATOR, "validate_p3f", side_effect=grow_after_validation),
              patch.object(AGGREGATOR, "MAX_P3F_ARTIFACT_BYTES", 4)):
            report = AGGREGATOR.aggregate(self.result_path, self.root)

        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any(
            "P3F artifact[0] exceeds the 4-byte record limit" in error
            for error in report["errors"]
        ), report["errors"])

    def test_gate_disagreement_fails_closed(self) -> None:
        self.p3f["gates"]["build"] = "passed"
        self.write_p3f()
        process, report = self.invoke()
        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("gate build conflicts" in error for error in report["errors"]))
        self.assertTrue(all(value == "not_run" for value in report["gate_states"].values()))

    def test_pipeline_status_must_match_p3f_record_exit_status(self) -> None:
        self.p3f["exit_status"] = 1
        self.write_p3f()

        process, report = self.invoke()

        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("exit_status conflicts with pipeline status" in error
                            for error in report["errors"]))
        self.assertTrue(all(value == "not_run" for value in report["gate_states"].values()))

    def test_failed_pipeline_accepts_matching_p3f_record_exit_status(self) -> None:
        self.p3f["exit_status"] = 1
        self.write_p3f()
        log_raw = b"snapshot rejected\n"
        (self.evidence / "failed-snapshot.log").write_bytes(log_raw)
        event = {
            "schema_version": 1,
            "step": "P3G-snapshot",
            "status": "failed",
            "exit_status": 1,
            "command": ["python", "validate_candidate_snapshot.py"],
            "log": {"path": "failed-snapshot.log", "sha256": digest(log_raw)},
            "manifest_sha256": self.manifest_hash,
        }
        event_raw = (json.dumps(event, sort_keys=True) + "\n").encode()
        (self.evidence / "failed-snapshot.json").write_bytes(event_raw)
        self.result["status"] = "failed"
        self.result["events"].append({
            **event,
            "event_record": {
                "path": "failed-snapshot.json",
                "sha256": digest(event_raw),
            },
        })
        self.write_result()

        process, report = self.invoke()

        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(report["status"], "consistent")
        self.assertEqual(report["gate_states"], {**self.gates, "promotion": "not_run"})

    def test_failed_command_event_requires_nonzero_integer_exit_status(self) -> None:
        self.p3f["exit_status"] = 1
        self.write_p3f()
        log_raw = b"snapshot rejected\n"
        (self.evidence / "failed-snapshot.log").write_bytes(log_raw)
        event = {
            "schema_version": 1,
            "step": "P3G-snapshot",
            "status": "failed",
            "exit_status": None,
            "command": ["python", "validate_candidate_snapshot.py"],
            "log": {"path": "failed-snapshot.log", "sha256": digest(log_raw)},
            "manifest_sha256": self.manifest_hash,
        }
        event_raw = (json.dumps(event, sort_keys=True) + "\n").encode()
        (self.evidence / "failed-snapshot.json").write_bytes(event_raw)
        self.result["status"] = "failed"
        self.result["events"].append({
            **event,
            "event_record": {
                "path": "failed-snapshot.json",
                "sha256": digest(event_raw),
            },
        })
        self.write_result()

        process, report = self.invoke()

        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("failed command must have a nonzero integer exit_status" in error
                            for error in report["errors"]))

    def test_promotion_cannot_be_claimed_by_pipeline_record(self) -> None:
        self.result["gates"]["promotion"] = "passed"
        self.write_result()
        process, report = self.invoke()
        self.assertEqual(process.returncode, 1)
        self.assertTrue(any("promotion must remain not_run" in error for error in report["errors"]))

    def test_event_record_hash_and_embedded_content_must_match(self) -> None:
        (self.evidence / "event-01-P3F.json").write_text("{}\n", encoding="utf-8")
        process, report = self.invoke()
        self.assertEqual(process.returncode, 1)
        self.assertTrue(any("sha256 does not match" in error for error in report["errors"]))

    def test_failed_command_event_requires_hash_bound_log(self) -> None:
        failed = {
            "schema_version": 1,
            "step": "build-command",
            "status": "failed",
            "exit_status": 1,
            "command": ["cargo", "check"],
            "manifest_sha256": self.manifest_hash,
            "recorded_at": "2026-10-21T00:00:00Z",
        }
        event_raw = (json.dumps(failed, sort_keys=True) + "\n").encode()
        (self.evidence / "failed-build.json").write_bytes(event_raw)
        self.result["status"] = "failed"
        self.result["events"].append({
            **failed,
            "event_record": {
                "path": "failed-build.json",
                "sha256": digest(event_raw),
            },
        })
        self.write_result()

        process, report = self.invoke()
        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("command event has no hash-bound log" in error
                            for error in report["errors"]))
        self.assertTrue(all(value == "not_run" for value in report["gate_states"].values()))

    def test_symlinked_event_artifact_is_rejected(self) -> None:
        target = self.evidence / "event-01-P3F.json"
        real = self.evidence / "event-real.json"
        target.rename(real)
        target.symlink_to(real.name)
        process, report = self.invoke()
        self.assertEqual(process.returncode, 1)
        self.assertTrue(any("symlink rejected" in error for error in report["errors"]))

    def test_symlinked_event_parent_directory_is_rejected(self) -> None:
        alias = self.evidence / "event-alias"
        alias.symlink_to(".", target_is_directory=True)
        self.result["events"][0]["event_record"]["path"] = (
            "event-alias/event-01-P3F.json"
        )
        self.write_result()
        process, report = self.invoke()
        self.assertEqual(process.returncode, 1)
        self.assertTrue(any("symlink rejected" in error for error in report["errors"]))

    def test_external_symlink_alias_to_pipeline_result_is_rejected(self) -> None:
        alias = self.root.parent / f"{self.root.name}-pipeline-result-alias.json"
        self.addCleanup(alias.unlink, missing_ok=True)
        alias.symlink_to(self.result_path)
        process = subprocess.run(
            [sys.executable, str(SCRIPT), str(alias), "--root", str(self.root)],
            capture_output=True, text=True, check=False,
        )
        report = json.loads(process.stdout)
        self.assertEqual(process.returncode, 1)
        self.assertTrue(any("symlink rejected" in error for error in report["errors"]))

    def test_external_symlink_alias_to_p3f_record_is_rejected(self) -> None:
        alias = self.root.parent / f"{self.root.name}-P3F-record-alias.json"
        self.addCleanup(alias.unlink, missing_ok=True)
        p3f_path = self.evidence / "P3F-record.json"
        alias.symlink_to(p3f_path)
        process, report = self.invoke(alias)
        self.assertEqual(process.returncode, 1)
        self.assertTrue(any("symlink rejected" in error for error in report["errors"]))

    def test_event_json_boolean_does_not_equal_integer(self) -> None:
        disk_event = dict(self.event)
        disk_event.pop("event_record")
        disk_event["schema_version"] = True
        event_raw = json.dumps(disk_event, indent=2, sort_keys=True).encode() + b"\n"
        event_path = self.evidence / "event-01-P3F.json"
        event_path.write_bytes(event_raw)
        self.result["events"][0]["event_record"]["sha256"] = digest(event_raw)
        self.write_result()
        process, report = self.invoke()
        self.assertEqual(process.returncode, 1)
        self.assertTrue(any("event file conflicts" in error for error in report["errors"]))

    def test_missing_p3f_validation_event_is_not_inferred_as_success(self) -> None:
        self.result["events"] = []
        self.write_result()
        process, report = self.invoke()
        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(all(value == "not_run" for value in report["gate_states"].values()))

    def test_linked_build_and_correctness_followups_reconcile_independently(self) -> None:
        m274, m276 = self.write_followups()
        process, report = self.invoke(followups=[m274, m276])
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(report["status"], "consistent")
        self.assertEqual(report["gate_states"], {
            "build": "passed", "correctness": "passed", "gui": "not_run",
            "performance": "not_run", "promotion": "not_run",
        })
        self.assertEqual([record["milestone"] for record in report["followup_summaries"]],
                         ["M274", "M276"])

    def test_m287_gui_reconciles_and_retains_paired_artifact_digests(self) -> None:
        m274, _ = self.write_followups()
        m287 = self.write_gui_followup()

        process, report = self.invoke(followups=[m274, m287])

        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(report["gate_states"]["gui"], "passed")
        gui_record = next(record for record in report["followup_summaries"]
                          if record["milestone"] == "M287")
        self.assertEqual(gui_record["source_hash_manifest_sha256"],
                         self.source_hash_manifest_hash)
        self.assertEqual(gui_record["gate_states"], {
            "visual": "passed", "accessibility": "passed", "gui": "passed",
        })
        for side in ("baseline", "candidate"):
            self.assertEqual(gui_record["snapshots"][side]["source_tree_sha256"],
                             self.snapshot_tree_hashes[side])
            for artifact in ("screenshot", "accessibility_tree"):
                ref = gui_record["snapshots"][side][artifact]
                self.assertEqual(ref["sha256"], digest((self.root / ref["path"]).read_bytes()))

    def test_m287_rejects_non_png_screenshot_even_when_hash_matches(self) -> None:
        m274, _ = self.write_followups()
        m287 = self.write_gui_followup()
        summary = json.loads(m287.read_text())
        screenshot_ref = summary["snapshots"]["baseline"]["screenshot"]
        screenshot = self.root / screenshot_ref["path"]
        screenshot_raw = b"plain text mislabeled as a screenshot\n"
        screenshot.write_bytes(screenshot_raw)
        screenshot_ref["sha256"] = digest(screenshot_raw)
        m287.write_text(json.dumps(summary) + "\n", encoding="utf-8")

        process, report = self.invoke(followups=[m274, m287])

        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("M287 baseline screenshot is not a PNG image" in error
                            for error in report["errors"]))

    def test_m287_rejects_signature_only_png_screenshot(self) -> None:
        m274, _ = self.write_followups()
        m287 = self.write_gui_followup()
        summary = json.loads(m287.read_text())
        screenshot_ref = summary["snapshots"]["baseline"]["screenshot"]
        screenshot = self.root / screenshot_ref["path"]
        screenshot_raw = b"\x89PNG\r\n\x1a\n"
        screenshot.write_bytes(screenshot_raw)
        screenshot_ref["sha256"] = digest(screenshot_raw)
        m287.write_text(json.dumps(summary) + "\n", encoding="utf-8")

        process, report = self.invoke(followups=[m274, m287])

        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("M287 baseline screenshot is not a PNG image" in error
                            for error in report["errors"]))

    def test_m287_rejects_png_with_only_empty_idat_data(self) -> None:
        m274, _ = self.write_followups()
        m287 = self.write_gui_followup()
        summary = json.loads(m287.read_text())
        screenshot_ref = summary["snapshots"]["baseline"]["screenshot"]
        screenshot = self.root / screenshot_ref["path"]

        def chunk(kind: bytes, data: bytes) -> bytes:
            body = kind + data
            return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body))

        screenshot_raw = (
            b"\x89PNG\r\n\x1a\n"
            + chunk(b"IHDR", struct.pack(">IIBBBBB", 1, 1, 8, 6, 0, 0, 0))
            + chunk(b"IDAT", b"")
            + chunk(b"IEND", b"")
        )
        screenshot.write_bytes(screenshot_raw)
        screenshot_ref["sha256"] = digest(screenshot_raw)
        m287.write_text(json.dumps(summary) + "\n", encoding="utf-8")

        process, report = self.invoke(followups=[m274, m287])

        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertEqual(report["gate_states"]["gui"], "not_run")

    def test_m287_artifact_read_is_bounded(self) -> None:
        m274, _ = self.write_followups()
        m287 = self.write_gui_followup()
        summary = json.loads(m287.read_text(encoding="utf-8"))
        screenshot_ref = summary["snapshots"]["baseline"]["screenshot"]
        screenshot = self.root / screenshot_ref["path"]
        screenshot_raw = b"oversized"
        screenshot.write_bytes(screenshot_raw)
        screenshot_ref["sha256"] = digest(screenshot_raw)
        m287.write_text(json.dumps(summary) + "\n", encoding="utf-8")

        with patch.object(AGGREGATOR, "MAX_GUI_ARTIFACT_BYTES", 4):
            report = AGGREGATOR.aggregate(
                self.result_path, self.root, followup_paths=[m274, m287],
                path_root=self.root,
            )

        self.assertEqual(report["status"], "invalid")
        self.assertEqual(report["gate_states"]["gui"], "not_run")
        self.assertTrue(any(
            "M287 baseline screenshot exceeds the 4-byte record limit" in error
            for error in report["errors"]
        ), report["errors"])

    def test_m287_rejects_reused_baseline_candidate_artifact_paths(self) -> None:
        for artifact_key in ("screenshot", "accessibility_tree"):
            with self.subTest(artifact=artifact_key):
                m274, _ = self.write_followups()
                m287 = self.write_gui_followup()
                summary = json.loads(m287.read_text())
                baseline_ref = summary["snapshots"]["baseline"][artifact_key]
                candidate_ref = summary["snapshots"]["candidate"][artifact_key]
                candidate_ref["path"] = baseline_ref["path"]
                candidate_ref["sha256"] = digest(
                    (self.root / baseline_ref["path"]).read_bytes()
                )
                m287.write_text(json.dumps(summary) + "\n", encoding="utf-8")

                process, report = self.invoke(followups=[m274, m287])

                self.assertEqual(process.returncode, 1)
                self.assertEqual(report["status"], "invalid")
                label = "screenshot" if artifact_key == "screenshot" else "accessibility tree"
                self.assertTrue(
                    any(
                        f"baseline and candidate {label} paths must differ" in error
                        for error in report["errors"]
                    )
                )

    def _assert_m287_rejects_hard_linked_artifact(self, artifact_key: str) -> None:
        m274, _ = self.write_followups()
        m287 = self.write_gui_followup()
        summary = json.loads(m287.read_text())
        baseline_ref = summary["snapshots"]["baseline"][artifact_key]
        candidate_ref = summary["snapshots"]["candidate"][artifact_key]
        baseline_path = self.root / baseline_ref["path"]
        candidate_path = self.root / candidate_ref["path"]
        candidate_path.unlink()
        os.link(baseline_path, candidate_path)
        candidate_ref["sha256"] = digest(candidate_path.read_bytes())
        m287.write_text(json.dumps(summary) + "\n", encoding="utf-8")

        process, report = self.invoke(followups=[m274, m287])

        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")

    def test_m287_rejects_hard_linked_screenshot_artifacts(self) -> None:
        self._assert_m287_rejects_hard_linked_artifact("screenshot")

    def test_m287_rejects_screenshot_hard_linked_outside_candidate_root(self) -> None:
        m274, _ = self.write_followups()
        m287 = self.write_gui_followup()
        summary = json.loads(m287.read_text())
        screenshot_ref = summary["snapshots"]["baseline"]["screenshot"]
        screenshot = self.root / screenshot_ref["path"]
        external = self.root.parent / f"{self.root.name}-external-screenshot.png"
        self.addCleanup(external.unlink, missing_ok=True)
        external.write_bytes(b"external screenshot\n")
        screenshot.unlink()
        os.link(external, screenshot)
        screenshot_ref["sha256"] = digest(screenshot.read_bytes())
        m287.write_text(json.dumps(summary) + "\n", encoding="utf-8")

        process, report = self.invoke(followups=[m274, m287])

        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("M287 baseline screenshot must not have multiple hard links" in error
                            for error in report["errors"]), report["errors"])

    def test_m287_rejects_screenshot_path_replaced_after_open(self) -> None:
        m274, _ = self.write_followups()
        m287 = self.write_gui_followup()
        screenshot = self.root / "evidence" / "baseline-window.png"
        original_open_child = AGGREGATOR._open_child

        def replace_after_open(parent_fd: int, name: str, flags: int, label: str,
                               relative: Path) -> int:
            descriptor = original_open_child(parent_fd, name, flags, label, relative)
            if label == "M287 baseline screenshot" and name == screenshot.name:
                replacement = screenshot.with_suffix(".replacement")
                replacement.write_bytes(b"replacement screenshot\n")
                replacement.replace(screenshot)
            return descriptor

        with patch.object(AGGREGATOR, "_open_child", side_effect=replace_after_open):
            report = AGGREGATOR.aggregate(
                self.result_path, self.root, followup_paths=[m274, m287]
            )

        self.assertEqual(report["status"], "invalid")
        self.assertTrue(all(value == "not_run" for value in report["gate_states"].values()))

    def test_m287_rejects_hard_linked_accessibility_tree_artifacts(self) -> None:
        self._assert_m287_rejects_hard_linked_artifact("accessibility_tree")

    def _assert_m287_rejects_cross_role_artifact_reuse(
        self,
        first_side: str,
        first_key: str,
        second_side: str,
        second_key: str,
        *,
        hard_link: bool = False,
    ) -> None:
        m274, _ = self.write_followups()
        m287 = self.write_gui_followup()
        summary = json.loads(m287.read_text())
        first_ref = summary["snapshots"][first_side][first_key]
        second_ref = summary["snapshots"][second_side][second_key]
        first_path = self.root / first_ref["path"]
        second_path = self.root / second_ref["path"]

        if hard_link:
            second_path.unlink()
            os.link(first_path, second_path)
        else:
            second_ref["path"] = first_ref["path"]
        second_ref["sha256"] = digest(first_path.read_bytes())
        m287.write_text(json.dumps(summary) + "\n", encoding="utf-8")

        process, report = self.invoke(followups=[m274, m287])

        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")

    def test_m287_rejects_same_side_cross_role_artifact_path_reuse(self) -> None:
        self._assert_m287_rejects_cross_role_artifact_reuse(
            "baseline", "screenshot", "baseline", "accessibility_tree"
        )

    def test_m287_rejects_cross_side_cross_role_hard_link_reuse(self) -> None:
        self._assert_m287_rejects_cross_role_artifact_reuse(
            "baseline", "screenshot", "candidate", "accessibility_tree", hard_link=True
        )

    def test_m287_accepts_same_bytes_at_distinct_baseline_candidate_paths(self) -> None:
        m274, _ = self.write_followups()
        m287 = self.write_gui_followup()
        summary = json.loads(m287.read_text())
        for artifact_key in ("screenshot", "accessibility_tree"):
            baseline_ref = summary["snapshots"]["baseline"][artifact_key]
            candidate_ref = summary["snapshots"]["candidate"][artifact_key]
            baseline_path = self.root / baseline_ref["path"]
            candidate_path = self.root / candidate_ref["path"]
            candidate_path.write_bytes(baseline_path.read_bytes())
            candidate_ref["sha256"] = digest(candidate_path.read_bytes())
        m287.write_text(json.dumps(summary) + "\n", encoding="utf-8")

        process, report = self.invoke(followups=[m274, m287])

        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(report["status"], "consistent")
        self.assertEqual(report["gate_states"]["gui"], "passed")

    def test_m287_rejects_candidate_and_source_manifest_transplants(self) -> None:
        cases = (
            ("candidate_id", "another-candidate", "candidate or manifest identity"),
            ("manifest_sha256", "b" * 64, "candidate or manifest identity"),
            ("source_hash_manifest_sha256", "b" * 64, "source hash manifest digest"),
        )
        for field, value, message in cases:
            with self.subTest(field=field):
                m274, _ = self.write_followups()
                m287 = self.write_gui_followup()
                summary = json.loads(m287.read_text())
                summary[field] = value
                m287.write_text(json.dumps(summary) + "\n")

                process, report = self.invoke(followups=[m274, m287])

                self.assertEqual(process.returncode, 1)
                self.assertEqual(report["status"], "invalid")
                self.assertTrue(any(message in error for error in report["errors"]))
                self.assertTrue(all(state == "not_run"
                                    for state in report["gate_states"].values()))

    def test_m287_rejects_source_tree_transplant_and_artifact_drift(self) -> None:
        m274, _ = self.write_followups()
        m287 = self.write_gui_followup()
        summary = json.loads(m287.read_text())
        summary["snapshots"]["candidate"]["source_tree_sha256"] = "b" * 64
        m287.write_text(json.dumps(summary) + "\n")
        process, report = self.invoke(followups=[m274, m287])
        self.assertEqual(process.returncode, 1)
        self.assertTrue(any("candidate source tree digest conflicts" in error
                            for error in report["errors"]))

        m274, _ = self.write_followups()
        m287 = self.write_gui_followup()
        summary = json.loads(m287.read_text())
        screenshot = self.root / summary["snapshots"]["baseline"]["screenshot"]["path"]
        screenshot.write_bytes(b"changed screenshot\n")
        process, report = self.invoke(followups=[m274, m287])
        self.assertEqual(process.returncode, 1)
        self.assertTrue(any("screenshot.sha256 does not match" in error
                            for error in report["errors"]))

    def test_m287_preserves_explicit_not_run_states(self) -> None:
        m274, _ = self.write_followups()
        m287 = self.write_gui_followup({
            "visual": "not_run", "accessibility": "not_run", "gui": "not_run",
        })
        process, report = self.invoke(followups=[m274, m287])
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(report["gate_states"]["gui"], "not_run")
        gui_record = next(record for record in report["followup_summaries"]
                          if record["milestone"] == "M287")
        self.assertEqual(gui_record["gate_states"]["gui"], "not_run")

    def test_m287_cannot_claim_visual_or_gui_pass_without_required_captures(self) -> None:
        cases = (
            ("screenshot_state", "not_run", "visual pass requires both screenshot captures"),
            ("screenshot", None, "screenshot is required for a passed state"),
            ("accessibility_state", "not_run",
             "accessibility pass requires both AX captures"),
            ("accessibility_tree", None, "accessibility_tree is required for a passed state"),
        )
        for artifact_field, value, message in cases:
            with self.subTest(artifact_field=artifact_field):
                m274, _ = self.write_followups()
                m287 = self.write_gui_followup()
                summary = json.loads(m287.read_text())
                summary["snapshots"]["candidate"][artifact_field] = value
                m287.write_text(json.dumps(summary) + "\n")
                process, report = self.invoke(followups=[m274, m287])
                self.assertEqual(process.returncode, 1)
                self.assertTrue(any(message in error for error in report["errors"]))

    def test_m287_requires_linked_m274_build_summary(self) -> None:
        _, _ = self.write_followups()
        m287 = self.write_gui_followup()
        process, report = self.invoke(followups=[m287])
        self.assertEqual(process.returncode, 1)
        self.assertTrue(any("M274 build summary" in error for error in report["errors"]))

    def test_followup_candidate_mismatch_fails_closed(self) -> None:
        m274, m276 = self.write_followups()
        summary = json.loads(m276.read_text())
        summary["candidate_id"] = "another-candidate"
        m276.write_text(json.dumps(summary) + "\n")
        process, report = self.invoke(followups=[m274, m276])
        self.assertEqual(process.returncode, 1)
        self.assertTrue(any("candidate or manifest identity" in error
                            for error in report["errors"]))
        self.assertTrue(all(value == "not_run" for value in report["gate_states"].values()))

    def test_m274_event_step_must_match_summary_slot(self) -> None:
        m274, _ = self.write_followups()
        summary = json.loads(m274.read_text())
        repeated_event = summary["events"]["candidate_build"].copy()
        summary["events"] = {name: repeated_event.copy() for name in summary["events"]}
        m274.write_text(json.dumps(summary) + "\n")

        process, report = self.invoke(followups=[m274])

        self.assertEqual(process.returncode, 1)
        self.assertTrue(any("wrong step" in error for error in report["errors"]))
        self.assertTrue(all(value == "not_run" for value in report["gate_states"].values()))

    def test_m274_event_rejects_unsupported_schema_version(self) -> None:
        m274, _ = self.write_followups()
        self.rewrite_m274_event(m274, "candidate_build", schema_version=2)

        process, report = self.invoke(followups=[m274])

        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("event schema_version must be 1" in error
                            for error in report["errors"]))

    def test_m274_event_record_size_is_bounded(self) -> None:
        m274, _ = self.write_followups()
        self.rewrite_m274_event(m274, "candidate_build", padding="x" * (1024 * 1024))

        process, report = self.invoke(followups=[m274])

        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("M274 candidate_build event exceeds the 1048576-byte record limit" in error
                            for error in report["errors"]))
        self.assertTrue(all(value == "not_run" for value in report["gate_states"].values()))

    def test_m274_summary_exit_status_must_match_successful_event(self) -> None:
        m274, _ = self.write_followups()
        summary = json.loads(m274.read_text())
        summary["events"]["candidate_build"]["exit_status"] = 1
        m274.write_text(json.dumps(summary) + "\n")

        process, report = self.invoke(followups=[m274])

        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("summary exit_status" in error for error in report["errors"]))
        self.assertTrue(all(value == "not_run" for value in report["gate_states"].values()))

    def test_m274_event_command_must_match_its_build_step(self) -> None:
        m274, _ = self.write_followups()
        summary = json.loads(m274.read_text())
        slot = summary["events"]["candidate_build"]
        event_path = self.root / slot["event_path"]
        event = json.loads(event_path.read_text())
        event["command"] = ["./scripts/cargo-headstart", "check", "--workspace",
                             "--all-targets", "--locked", "--offline"]
        event_raw = (json.dumps(event, sort_keys=True) + "\n").encode()
        event_path.write_bytes(event_raw)
        slot["event_sha256"] = digest(event_raw)
        m274.write_text(json.dumps(summary) + "\n")

        process, report = self.invoke(followups=[m274])

        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("command does not match its locked all-targets build" in error
                            for error in report["errors"]))
        self.assertTrue(all(value == "not_run" for value in report["gate_states"].values()))

    def test_m274_event_rejects_executable_name_with_cargo_headstart_suffix(self) -> None:
        m274, _ = self.write_followups()
        summary = json.loads(m274.read_text())
        slot = summary["events"]["candidate_build"]
        event_path = self.root / slot["event_path"]
        event = json.loads(event_path.read_text())
        event["command"][0] = "./scripts/fake-cargo-headstart"
        event_raw = (json.dumps(event, sort_keys=True) + "\n").encode()
        event_path.write_bytes(event_raw)
        slot["event_sha256"] = digest(event_raw)
        m274.write_text(json.dumps(summary) + "\n")

        process, report = self.invoke(followups=[m274])

        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("command does not match its locked all-targets build" in error
                            for error in report["errors"]))
        self.assertTrue(all(value == "not_run" for value in report["gate_states"].values()))

    def test_m274_event_requires_explicit_pass_status(self) -> None:
        m274, _ = self.write_followups()
        self.rewrite_m274_event(m274, "candidate_build", status=None)

        process, report = self.invoke(followups=[m274])

        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertEqual(len(report["errors"]), 1)
        self.assertIn("not successful", report["errors"][0])
        self.assertTrue(all(value == "not_run" for value in report["gate_states"].values()))

    def test_m274_event_requires_side_specific_identity(self) -> None:
        cases = (("baseline_check", "candidate_id"),
                 ("candidate_check", "manifest_sha256"))
        for slot, identity_field in cases:
            with self.subTest(slot=slot):
                m274, _ = self.write_followups()
                self.rewrite_m274_event(m274, slot, **{identity_field: None})
                process, report = self.invoke(followups=[m274])
                self.assertEqual(process.returncode, 1)
                self.assertEqual(report["status"], "invalid")
                self.assertTrue(any(identity_field + " identity is missing or conflicts" in error
                                    for error in report["errors"]))
                self.assertTrue(all(value == "not_run" for value in report["gate_states"].values()))

    def test_m274_event_rejects_transplanted_side_specific_identity(self) -> None:
        cases = (("baseline_check", "candidate_id", "another-candidate"),
                 ("candidate_check", "manifest_sha256", "b" * 64))
        for slot, identity_field, transplanted_value in cases:
            with self.subTest(slot=slot):
                m274, _ = self.write_followups()
                self.rewrite_m274_event(m274, slot, **{identity_field: transplanted_value})
                process, report = self.invoke(followups=[m274])
                self.assertEqual(process.returncode, 1)
                self.assertEqual(report["status"], "invalid")
                self.assertTrue(any(identity_field + " identity is missing or conflicts" in error
                                    for error in report["errors"]))
                self.assertTrue(all(value == "not_run" for value in report["gate_states"].values()))

    def test_followup_manifest_mismatch_fails_closed(self) -> None:
        m274, m276 = self.write_followups()
        summary = json.loads(m276.read_text())
        summary["manifest_sha256"] = "b" * 64
        m276.write_text(json.dumps(summary) + "\n")
        process, report = self.invoke(followups=[m274, m276])
        self.assertEqual(process.returncode, 1)
        self.assertTrue(any("candidate or manifest identity" in error
                            for error in report["errors"]))
        self.assertTrue(all(value == "not_run" for value in report["gate_states"].values()))

    def test_m276_build_summary_digest_must_match_supplied_m274(self) -> None:
        m274, m276 = self.write_followups()
        summary = json.loads(m276.read_text())
        summary["build_summary_sha256"] = "b" * 64
        m276.write_text(json.dumps(summary) + "\n")
        process, report = self.invoke(followups=[m274, m276])
        self.assertEqual(process.returncode, 1)
        self.assertTrue(any("M276 build summary path or digest" in error
                            for error in report["errors"]))
        self.assertTrue(all(value == "not_run" for value in report["gate_states"].values()))

    def test_m276_success_event_rejects_explicit_nonpassed_status(self) -> None:
        for status in ("failed", "not_run", None):
            with self.subTest(status=status):
                m274, m276 = self.write_followups()
                summary = json.loads(m276.read_text())
                event_path = self.root / summary["success_event_path"]
                event = json.loads(event_path.read_text())
                event["status"] = status
                event_raw = (json.dumps(event, sort_keys=True) + "\n").encode()
                event_path.write_bytes(event_raw)
                summary["success_event_sha256"] = digest(event_raw)
                m276.write_text(json.dumps(summary) + "\n")

                process, report = self.invoke(followups=[m274, m276])

                self.assertEqual(process.returncode, 1)
                self.assertEqual(report["status"], "invalid")
                self.assertTrue(any("not a successful M276 event" in error
                                    for error in report["errors"]))
                self.assertTrue(all(value == "not_run"
                                    for value in report["gate_states"].values()))

    def test_m276_success_event_requires_explicit_status(self) -> None:
        m274, m276 = self.write_followups()
        summary = json.loads(m276.read_text())
        event_path = self.root / summary["success_event_path"]
        event = json.loads(event_path.read_text())
        event.pop("status")
        event_raw = (json.dumps(event, sort_keys=True) + "\n").encode()
        event_path.write_bytes(event_raw)
        summary["success_event_sha256"] = digest(event_raw)
        m276.write_text(json.dumps(summary) + "\n")

        process, report = self.invoke(followups=[m274, m276])

        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("not a successful M276 event" in error
                            for error in report["errors"]))
        self.assertTrue(all(value == "not_run" for value in report["gate_states"].values()))

    def test_m276_success_event_record_size_is_bounded(self) -> None:
        m274, m276 = self.write_followups()
        summary = json.loads(m276.read_text(encoding="utf-8"))
        event_path = self.root / summary["success_event_path"]
        event = json.loads(event_path.read_text(encoding="utf-8"))
        event["padding"] = "x" * (1024 * 1024)
        event_raw = (json.dumps(event, sort_keys=True) + "\n").encode()
        event_path.write_bytes(event_raw)
        summary["success_event_sha256"] = digest(event_raw)
        m276.write_text(json.dumps(summary, sort_keys=True) + "\n", encoding="utf-8")

        process, report = self.invoke(followups=[m274, m276])

        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("M276 correctness event exceeds the 1048576-byte record limit" in error
                            for error in report["errors"]))
        self.assertTrue(all(value == "not_run" for value in report["gate_states"].values()))

    def test_m276_success_event_rejects_transplanted_identity(self) -> None:
        cases = (("candidate_id", "another-candidate"),
                 ("manifest_sha256", "b" * 64))
        for field, value in cases:
            with self.subTest(field=field):
                m274, m276 = self.write_followups()
                summary = json.loads(m276.read_text(encoding="utf-8"))
                event_path = self.root / summary["success_event_path"]
                event = json.loads(event_path.read_text(encoding="utf-8"))
                event[field] = value
                event_raw = (json.dumps(event, sort_keys=True) + "\n").encode()
                event_path.write_bytes(event_raw)
                summary["success_event_sha256"] = digest(event_raw)
                m276.write_text(json.dumps(summary, sort_keys=True) + "\n", encoding="utf-8")

                process, report = self.invoke(followups=[m274, m276])

                self.assertEqual(process.returncode, 1)
                self.assertEqual(report["status"], "invalid")
                self.assertTrue(any(f"{field} identity" in error
                                    for error in report["errors"]))

    def test_m276_success_event_requires_identity_fields(self) -> None:
        for field in ("candidate_id", "manifest_sha256"):
            with self.subTest(field=field):
                m274, m276 = self.write_followups()
                summary = json.loads(m276.read_text(encoding="utf-8"))
                event_path = self.root / summary["success_event_path"]
                event = json.loads(event_path.read_text(encoding="utf-8"))
                event.pop(field)
                event_raw = (json.dumps(event, sort_keys=True) + "\n").encode()
                event_path.write_bytes(event_raw)
                summary["success_event_sha256"] = digest(event_raw)
                m276.write_text(json.dumps(summary, sort_keys=True) + "\n", encoding="utf-8")

                process, report = self.invoke(followups=[m274, m276])

                self.assertEqual(process.returncode, 1)
                self.assertEqual(report["status"], "invalid")
                self.assertTrue(any(
                    f"{field} identity is missing" in error for error in report["errors"]
                ))
                self.assertTrue(all(value == "not_run"
                                    for value in report["gate_states"].values()))

    def test_followup_event_hash_drift_fails_closed(self) -> None:
        m274, m276 = self.write_followups()
        summary = json.loads(m274.read_text())
        event = summary["events"]["candidate_build"]
        (self.root / event["event_path"]).write_text("{}\n")
        process, report = self.invoke(followups=[m274])
        self.assertEqual(process.returncode, 1)
        self.assertTrue(any("event.sha256 does not match" in error
                            for error in report["errors"]))
        self.assertTrue(all(value == "not_run" for value in report["gate_states"].values()))

    def test_followup_log_hash_drift_fails_closed(self) -> None:
        m274, _ = self.write_followups()
        summary = json.loads(m274.read_text())
        log = summary["events"]["candidate_build"]["log_path"]
        (self.root / log).write_bytes(b"changed\n")
        process, report = self.invoke(followups=[m274])
        self.assertEqual(process.returncode, 1)
        self.assertTrue(any("log.sha256 does not match" in error
                            for error in report["errors"]))
        self.assertTrue(all(value == "not_run" for value in report["gate_states"].values()))

    def test_m276_requires_explicit_m274_summary(self) -> None:
        _, m276 = self.write_followups()
        process, report = self.invoke(followups=[m276])
        self.assertEqual(process.returncode, 1)
        self.assertTrue(any("requires its linked M274" in error
                            for error in report["errors"]))
        self.assertTrue(all(value == "not_run" for value in report["gate_states"].values()))


    def test_candidate_manifest_size_uses_p3g_bound(self) -> None:
        with (self.root / "manifest.json").open("wb") as stream:
            stream.truncate(32 * 1024 * 1024 + 1)

        process, report = self.invoke()

        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("candidate manifest exceeds the 33554432-byte record limit" in error for error in report["errors"]), report["errors"])

    def test_source_hash_manifest_size_uses_p3g_bound(self) -> None:
        source_manifest = self.root / "source-hash-manifest.json"
        with source_manifest.open("wb") as stream:
            stream.truncate(32 * 1024 * 1024 + 1)
        self.result["source_hash_manifest_sha256"] = digest(source_manifest.read_bytes())
        self.write_result()

        process, report = self.invoke()

        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("source hash manifest exceeds the 33554432-byte record limit" in error
                            for error in report["errors"]), report["errors"])

    def test_source_hash_manifest_snapshot_files_match_candidate_manifest(self) -> None:
        source_manifest_path = self.root / "source-hash-manifest.json"
        source_manifest = json.loads(source_manifest_path.read_text())
        source_manifest["snapshots"]["candidate"]["files"]["unexpected.rs"] = "0" * 64
        source_raw = json.dumps(source_manifest, sort_keys=True).encode()
        source_manifest_path.write_bytes(source_raw)
        source_hash = digest(source_raw)
        self.result["source_hash_manifest_sha256"] = source_hash
        for artifact in self.p3f["evidence"]:
            if artifact["path"] == "source-hash-manifest.json":
                artifact["sha256"] = source_hash
        self.write_p3f()
        self.write_result()

        process, report = self.invoke()

        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("source hash manifest candidate snapshot files conflict" in error
                            for error in report["errors"]), report["errors"])

    def test_source_hash_manifest_requires_integer_schema_version(self) -> None:
        source_manifest_path = self.root / "source-hash-manifest.json"
        source_manifest = json.loads(source_manifest_path.read_text())
        source_manifest["schema_version"] = True
        source_raw = json.dumps(source_manifest, sort_keys=True).encode()
        source_manifest_path.write_bytes(source_raw)
        source_hash = digest(source_raw)
        self.result["source_hash_manifest_sha256"] = source_hash
        for artifact in self.p3f["evidence"]:
            if artifact["path"] == "source-hash-manifest.json":
                artifact["sha256"] = source_hash
        self.write_p3f()
        self.write_result()

        process, report = self.invoke()

        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("source hash manifest schema_version must be 1" in error
                            for error in report["errors"]), report["errors"])

    def test_snapshot_file_map_comparison_distinguishes_boolean_from_integer(self) -> None:
        self.assertFalse(
            AGGREGATOR._snapshot_file_maps_match({"file.rs": True}, {"file.rs": 1})
        )

    def test_source_hash_manifest_tree_hash_matches_snapshot_files(self) -> None:
        source_manifest_path = self.root / "source-hash-manifest.json"
        source_manifest = json.loads(source_manifest_path.read_text())
        source_manifest["snapshots"]["candidate"]["tree_sha256"] = "0" * 64
        source_raw = json.dumps(source_manifest, sort_keys=True).encode()
        source_manifest_path.write_bytes(source_raw)
        source_hash = digest(source_raw)
        self.result["source_hash_manifest_sha256"] = source_hash
        for artifact in self.p3f["evidence"]:
            if artifact["path"] == "source-hash-manifest.json":
                artifact["sha256"] = source_hash
        self.write_p3f()
        self.write_result()

        process, report = self.invoke()

        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("source hash manifest candidate snapshot tree hash is invalid" in error
                            for error in report["errors"]), report["errors"])

    def test_inherited_candidate_fd_survives_named_path_swap(self) -> None:
        repo = self.root / "repo"
        candidate_prefix = Path("target/headstart/candidates") / self.candidate_id
        candidate_path = repo / candidate_prefix
        candidate_path.parent.mkdir(parents=True)
        shutil.copytree(self.root, candidate_path, ignore=shutil.ignore_patterns("repo"))
        from p3_candidate_fs import PinnedDir

        repo_root = PinnedDir.open_root(repo)
        candidate_parent = PinnedDir.open_child(repo_root, "target/headstart/candidates")
        candidate_root = PinnedDir.open_child(candidate_parent, self.candidate_id)
        self.addCleanup(candidate_root.close)
        self.addCleanup(candidate_parent.close)
        self.addCleanup(repo_root.close)
        output_relative = candidate_prefix / "evidence/aggregate-report.json"
        argv = [
            str(SCRIPT),
            (candidate_prefix / "evidence/pipeline-result.json").as_posix(),
            "--root", candidate_prefix.as_posix(),
            "--output", output_relative.as_posix(),
        ]
        env = {
            "RUSTUI_P3_REPO_ROOT_FD": str(repo_root.fd),
            "RUSTUI_P3_CANDIDATE_PARENT_FD": str(candidate_parent.fd),
            "RUSTUI_P3_CANDIDATE_ROOT_FD": str(candidate_root.fd),
            "RUSTUI_P3_CANDIDATE_ID": self.candidate_id,
        }
        original_aggregate = AGGREGATOR.aggregate
        moved_path = candidate_path.with_name("candidate-one-pinned")
        captured = io.StringIO()

        def aggregate_while_swapped(*args, **kwargs):
            candidate_path.rename(moved_path)
            candidate_path.mkdir()
            (candidate_path / "evidence").mkdir()
            (candidate_path / "evidence/pipeline-result.json").write_text("{}\n")
            try:
                return original_aggregate(*args, **kwargs)
            finally:
                shutil.rmtree(candidate_path)
                moved_path.rename(candidate_path)

        with patch.dict(os.environ, env), patch("sys.argv", argv), \
                patch.object(AGGREGATOR, "aggregate", side_effect=aggregate_while_swapped), \
                patch("sys.stdout", captured):
            status = AGGREGATOR.main()

        self.assertEqual(status, 0, captured.getvalue())
        report = json.loads(captured.getvalue())
        self.assertEqual(report["status"], "consistent")
        self.assertTrue((candidate_path / "evidence/aggregate-report.json").is_file())
        self.assertEqual(
            json.loads((candidate_path / "evidence/aggregate-report.json").read_text()),
            report,
        )


if __name__ == "__main__":
    unittest.main()
