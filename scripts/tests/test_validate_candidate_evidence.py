"""Focused tests for the read-only candidate evidence validator."""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch


SCRIPT = Path(__file__).resolve().parents[1] / "validate_candidate_evidence.py"
sys.path.insert(0, str(SCRIPT.parent))
import validate_candidate_evidence  # noqa: E402


class ValidateCandidateEvidenceTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.log_bytes = b"candidate check completed\n"
        self.evidence_bytes = b'{"observation":"bounded"}\n'
        (self.root / "logs").mkdir()
        (self.root / "evidence").mkdir()
        (self.root / "logs" / "check.log").write_bytes(self.log_bytes)
        (self.root / "evidence" / "check.json").write_bytes(self.evidence_bytes)
        self.record = {
            "manifest_sha256": "a" * 64,
            "command": ["cargo", "check", "--locked", "--offline"],
            "exit_status": 0,
            "log": {"path": "logs/check.log", "sha256": hashlib.sha256(self.log_bytes).hexdigest()},
            "evidence": [{"path": "evidence/check.json", "sha256": hashlib.sha256(self.evidence_bytes).hexdigest()}],
            "provenance": {"source_revision": "unknown"},
            "gates": {"build": "passed", "correctness": "not_run", "gui": "not_run", "performance": "not_run"},
        }

    def invoke(self, value: dict | str) -> subprocess.CompletedProcess[str]:
        path = self.root / "event.json"
        path.write_text(value if isinstance(value, str) else json.dumps(value), encoding="utf-8")
        return subprocess.run(
            [sys.executable, str(SCRIPT), str(path), "--root", str(self.root)],
            capture_output=True,
            text=True,
            check=False,
        )

    def test_valid_record_reports_identity_and_gate_states(self) -> None:
        result = self.invoke(self.record)
        report = json.loads(result.stdout)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["status"], "valid")
        self.assertEqual(report["manifest_sha256"], "a" * 64)
        self.assertEqual(report["gate_states"], self.record["gates"])

    def test_pass_requires_zero_exit_log_and_evidence(self) -> None:
        record = json.loads(json.dumps(self.record))
        record["exit_status"] = 1
        result = self.invoke(record)
        report = json.loads(result.stdout)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("exit_status is 0" in error for error in report["errors"]))

        record = json.loads(json.dumps(self.record))
        record["evidence"] = []
        report = json.loads(self.invoke(record).stdout)
        self.assertTrue(any("without at least one valid evidence artifact" in error for error in report["errors"]))

        record = json.loads(json.dumps(self.record))
        record["log"] = {"path": "logs/missing.log", "sha256": "b" * 64}
        report = json.loads(self.invoke(record).stdout)
        self.assertTrue(any("without a valid log artifact" in error for error in report["errors"]))

    def test_exit_status_must_fit_the_process_status_range(self) -> None:
        record = json.loads(json.dumps(self.record))
        record["gates"] = {name: "not_run" for name in validate_candidate_evidence.GATE_NAMES}
        record["exit_status"] = 255
        report = json.loads(self.invoke(record).stdout)
        self.assertEqual(report["status"], "valid")

        record["exit_status"] = 256
        report = json.loads(self.invoke(record).stdout)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("integer from 0 to 255" in error for error in report["errors"]))

    def test_rejects_bad_digest_mismatch_and_unsafe_paths(self) -> None:
        for path in ("../outside", "/tmp/outside", "C:/outside", "bad\x00path", "logs/./check.log"):
            with self.subTest(path=path):
                record = json.loads(json.dumps(self.record))
                record["evidence"][0]["path"] = path
                report = json.loads(self.invoke(record).stdout)
                self.assertEqual(report["status"], "invalid")

        record = json.loads(json.dumps(self.record))
        record["evidence"][0]["sha256"] = "0" * 64
        report = json.loads(self.invoke(record).stdout)
        self.assertTrue(any("does not match" in error for error in report["errors"]))

    def test_log_and_supporting_evidence_must_be_distinct_files(self) -> None:
        duplicate_contents = self.root / "evidence" / "same-contents.json"
        duplicate_contents.write_bytes(self.log_bytes)
        record = json.loads(json.dumps(self.record))
        record["evidence"][0] = {
            "path": "evidence/same-contents.json",
            "sha256": hashlib.sha256(self.log_bytes).hexdigest(),
        }
        report = json.loads(self.invoke(record).stdout)
        self.assertEqual(report["status"], "valid")

        record = json.loads(json.dumps(self.record))
        record["evidence"][0] = {
            "path": "logs/check.log",
            "sha256": hashlib.sha256(self.log_bytes).hexdigest(),
        }
        report = json.loads(self.invoke(record).stdout)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("distinct regular file" in error for error in report["errors"]))
        alias = self.root / "evidence" / "check-hardlink.json"
        alias.hardlink_to(self.root / "evidence" / "check.json")
        record = json.loads(json.dumps(self.record))
        record["evidence"].append({
            "path": "evidence/check-hardlink.json",
            "sha256": hashlib.sha256(self.evidence_bytes).hexdigest(),
        })
        report = json.loads(self.invoke(record).stdout)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("multiple hard links" in error for error in report["errors"]))

    def test_rejects_artifact_hard_linked_outside_candidate_root(self) -> None:
        outside = self.root.parent / f"{self.root.name}-outside.log"
        self.addCleanup(lambda: outside.unlink(missing_ok=True))
        outside.hardlink_to(self.root / "logs" / "check.log")

        report = json.loads(self.invoke(self.record).stdout)

        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("multiple hard links" in error for error in report["errors"]))

    def test_rejects_symlink_escape_and_manifest_record_escape(self) -> None:
        outside = self.root.parent / f"{self.root.name}-outside.log"
        outside.write_bytes(self.log_bytes)
        self.addCleanup(lambda: outside.unlink(missing_ok=True))
        (self.root / "logs" / "outside.log").symlink_to(outside)
        record = json.loads(json.dumps(self.record))
        record["log"]["path"] = "logs/outside.log"
        report = json.loads(self.invoke(record).stdout)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("symlink" in error for error in report["errors"]))

        (self.root / "logs" / "inside.log").symlink_to(self.root / "logs" / "check.log")
        record = json.loads(json.dumps(self.record))
        record["log"]["path"] = "logs/inside.log"
        report = json.loads(self.invoke(record).stdout)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("symlink" in error for error in report["errors"]))

        outside_record = self.root.parent / f"{self.root.name}-record.json"
        outside_record.write_text(json.dumps(self.record), encoding="utf-8")
        self.addCleanup(lambda: outside_record.unlink(missing_ok=True))
        result = subprocess.run(
            [sys.executable, str(SCRIPT), str(outside_record), "--root", str(self.root)],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode, 1)
        self.assertIn("cannot read evidence record", json.loads(result.stdout)["errors"][0])

    def test_rejects_symlinked_record_even_when_target_is_inside_root(self) -> None:
        record = self.root / "event.json"
        record.write_text(json.dumps(self.record), encoding="utf-8")
        alias = self.root / "event-alias.json"
        alias.symlink_to(record)

        result = subprocess.run(
            [sys.executable, str(SCRIPT), str(alias), "--root", str(self.root)],
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(result.returncode, 1)
        report = json.loads(result.stdout)
        self.assertEqual(report["status"], "invalid")
        self.assertIn("symlink", report["errors"][0])

    def test_rejects_record_hard_link_created_while_it_is_read(self) -> None:
        record = self.root / "event.json"
        record.write_text(json.dumps(self.record), encoding="utf-8")
        outside_alias = self.root.parent / f"{self.root.name}-event-alias.json"
        self.addCleanup(lambda: outside_alias.unlink(missing_ok=True))
        original_open = validate_candidate_evidence._open_regular_file

        class LinkingRead:
            def __init__(self, stream):
                self.stream = stream
                self.linked = False

            def __enter__(self):
                self.stream.__enter__()
                return self

            def __exit__(self, *args):
                return self.stream.__exit__(*args)

            def fileno(self) -> int:
                return self.stream.fileno()

            def read(self, size: int = -1) -> bytes:
                if not self.linked:
                    outside_alias.hardlink_to(record)
                    self.linked = True
                return self.stream.read(size)

        def link_during_record_read(root, relative, field):
            stream = original_open(root, relative, field)
            return LinkingRead(stream) if field == "evidence record" else stream

        with patch.object(validate_candidate_evidence, "_open_regular_file",
                          side_effect=link_during_record_read):
            with self.assertRaisesRegex(validate_candidate_evidence.EvidenceError,
                                        "gained a hard link while being read"):
                validate_candidate_evidence._load(self.root.resolve(), Path("event.json"))

    def test_rejects_root_directory_as_record_with_json_report(self) -> None:
        result = subprocess.run(
            [sys.executable, str(SCRIPT), str(self.root), "--root", str(self.root)],
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(result.returncode, 1)
        self.assertEqual(result.stderr, "")
        report = json.loads(result.stdout)
        self.assertEqual(report["status"], "invalid")
        self.assertIn("record path must identify a regular file", report["errors"][0])

    def test_inherited_candidate_id_uses_pipeline_safe_identifier_contract(self) -> None:
        candidate_parent = self.root / "target/headstart/candidates"
        candidate_parent.mkdir(parents=True)
        directory_flags = os.O_RDONLY | getattr(os, "O_DIRECTORY", 0)
        repo_fd = os.open(self.root, directory_flags)
        parent_fd = os.open(candidate_parent, directory_flags)
        self.addCleanup(os.close, repo_fd)
        self.addCleanup(os.close, parent_fd)
        for candidate_id in ("candidate-a", "nested/one", "bad id", "x..y", "x" * 65):
            with self.subTest(candidate_id=candidate_id):
                candidate = candidate_parent / candidate_id
                candidate.mkdir(parents=True)
                (candidate / "logs").mkdir()
                (candidate / "evidence").mkdir()
                (candidate / "logs/check.log").write_bytes(self.log_bytes)
                (candidate / "evidence/check.json").write_bytes(self.evidence_bytes)
                (candidate / "event.json").write_text(json.dumps(self.record), encoding="utf-8")
                candidate_fd = os.open(candidate, directory_flags)
                try:
                    env = os.environ.copy()
                    env.update({
                        "RUSTUI_P3_REPO_ROOT_FD": str(repo_fd),
                        "RUSTUI_P3_CANDIDATE_PARENT_FD": str(parent_fd),
                        "RUSTUI_P3_CANDIDATE_ROOT_FD": str(candidate_fd),
                        "RUSTUI_P3_CANDIDATE_ID": candidate_id,
                    })
                    relative = Path("target/headstart/candidates") / candidate_id
                    result = subprocess.run(
                        [sys.executable, str(SCRIPT), "event.json", "--root", str(relative)],
                        cwd=self.root, env=env, pass_fds=(repo_fd, parent_fd, candidate_fd),
                        capture_output=True, text=True, check=False,
                    )
                finally:
                    os.close(candidate_fd)
                self.assertEqual(result.stderr, "")
                report = json.loads(result.stdout)
                if candidate_id == "candidate-a":
                    self.assertEqual(result.returncode, 0, report)
                    self.assertEqual(report["status"], "valid")
                else:
                    self.assertEqual(result.returncode, 1, report)
                    self.assertEqual(report["status"], "invalid")
                    self.assertIn("safe candidate identifier", report["errors"][0])

    def test_rejects_inherited_candidate_parent_outside_repository(self) -> None:
        candidate_parent = self.root / "target/headstart/candidates"
        candidate = candidate_parent / "candidate-a"
        candidate.mkdir(parents=True)
        (candidate / "event.json").write_text(json.dumps(self.record), encoding="utf-8")

        decoy_parent = self.root / "decoy-candidates"
        decoy_parent.mkdir()
        decoy_candidate = decoy_parent / "candidate-a"
        decoy_candidate.mkdir()

        directory_flags = os.O_RDONLY | getattr(os, "O_DIRECTORY", 0)
        repo_fd = os.open(self.root, directory_flags)
        parent_fd = os.open(decoy_parent, directory_flags)
        candidate_fd = os.open(decoy_candidate, directory_flags)
        try:
            env = os.environ.copy()
            env.update({
                "RUSTUI_P3_REPO_ROOT_FD": str(repo_fd),
                "RUSTUI_P3_CANDIDATE_PARENT_FD": str(parent_fd),
                "RUSTUI_P3_CANDIDATE_ROOT_FD": str(candidate_fd),
                "RUSTUI_P3_CANDIDATE_ID": "candidate-a",
            })
            result = subprocess.run(
                [sys.executable, str(SCRIPT), "target/headstart/candidates/candidate-a/event.json",
                 "--root", "target/headstart/candidates/candidate-a"],
                cwd=self.root,
                env=env,
                pass_fds=(repo_fd, parent_fd, candidate_fd),
                capture_output=True,
                text=True,
                check=False,
            )
        finally:
            os.close(candidate_fd)
            os.close(parent_fd)
            os.close(repo_fd)

        self.assertEqual(result.returncode, 1)
        report = json.loads(result.stdout)
        self.assertEqual(report["status"], "invalid")
        self.assertIn("inherited candidate parent does not match repository root",
                      report["errors"][0])

    def test_rejects_symlink_swap_after_parent_directory_is_opened(self) -> None:
        outside = self.root.parent / f"{self.root.name}-swapped.log"
        outside.write_bytes(self.log_bytes)
        self.addCleanup(lambda: outside.unlink(missing_ok=True))
        target = self.root / "logs" / "check.log"
        backup = self.root / "logs" / "check.log.saved"
        original_open_child = validate_candidate_evidence._open_child
        swapped = False

        def swap_before_open(parent_fd, name, flags, field, relative):
            nonlocal swapped
            if name == "check.log" and not swapped:
                target.rename(backup)
                target.symlink_to(outside)
                swapped = True
            return original_open_child(parent_fd, name, flags, field, relative)

        with patch.object(validate_candidate_evidence, "_open_child", side_effect=swap_before_open):
            report = validate_candidate_evidence.validate(self.record, self.root)

        self.assertTrue(swapped)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("symlink" in error for error in report["errors"]))

    def test_directory_component_open_closes_temporary_pinned_owner(self) -> None:
        opened_children = []
        original_open_child = validate_candidate_evidence.PinnedDir.open_child

        def track_open_child(parent, relative):
            child = original_open_child(parent, relative)
            opened_children.append(child)
            return child

        with patch.object(validate_candidate_evidence.PinnedDir, "open_child",
                          side_effect=track_open_child):
            with validate_candidate_evidence._open_regular_file(
                self.root.resolve(), Path("logs/check.log"), "test.path"
            ) as stream:
                self.assertEqual(stream.read(), self.log_bytes)

        self.assertTrue(opened_children)
        self.assertTrue(all(not child._owned for child in opened_children))

    def test_rejects_artifact_path_replaced_while_digest_is_read(self) -> None:
        evidence_path = self.root / "evidence" / "check.json"
        replacement = self.root / "evidence" / "replacement.json"
        replacement.write_bytes(b'{"observation":"changed"}\n')
        original_open = validate_candidate_evidence._open_regular_file
        replaced = False

        class ReplacingRead:
            def __init__(self, stream):
                self.stream = stream

            def __enter__(self):
                self.stream.__enter__()
                return self

            def __exit__(self, *args):
                return self.stream.__exit__(*args)

            def fileno(self) -> int:
                return self.stream.fileno()

            def read(self, size: int = -1) -> bytes:
                nonlocal replaced
                chunk = self.stream.read(size)
                if not replaced:
                    evidence_path.unlink()
                    replacement.rename(evidence_path)
                    replaced = True
                return chunk

        def replace_after_open(root: Path, relative: Path, field: str):
            stream = original_open(root, relative, field)
            if field == "evidence[0].path":
                return ReplacingRead(stream)
            return stream

        with patch.object(validate_candidate_evidence, "_open_regular_file",
                          side_effect=replace_after_open):
            report = validate_candidate_evidence.validate(self.record, self.root)

        self.assertTrue(replaced)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("changed while its digest was being checked" in error
                            for error in report["errors"]))

    def test_rejects_duplicate_keys_unknown_fields_and_invalid_states(self) -> None:
        result = self.invoke('{"manifest_sha256":"' + "a" * 64 + '","manifest_sha256":"' + "a" * 64 + '"}')
        self.assertEqual(result.returncode, 1)
        self.assertIn("duplicate JSON key", json.loads(result.stdout)["errors"][0])

        record = json.loads(json.dumps(self.record))
        record["extra"] = True
        report = json.loads(self.invoke(record).stdout)
        self.assertTrue(any("exactly manifest_sha256" in error for error in report["errors"]))

        record = json.loads(json.dumps(self.record))
        record["gates"]["gui"] = {"state": "passed"}
        report = json.loads(self.invoke(record).stdout)
        self.assertTrue(any("gates.gui must be one of" in error for error in report["errors"]))

    def test_rejects_json_numbers_outside_finite_float_range(self) -> None:
        raw = json.dumps(self.record).replace(
            '"source_revision": "unknown"',
            '"source_revision": "unknown", "measurement": 1e9999',
        )

        result = self.invoke(raw)

        self.assertEqual(result.returncode, 1)
        report = json.loads(result.stdout)
        self.assertEqual(report["status"], "invalid")
        self.assertIn("outside the finite float range", report["errors"][0])

    def test_rejects_oversized_evidence_record_without_reading_it_all(self) -> None:
        record = self.root / "event.json"
        with record.open("wb") as stream:
            stream.write(b" " * validate_candidate_evidence.MAX_RECORD_BYTES)
            stream.write(b" ")

        result = subprocess.run(
            [sys.executable, str(SCRIPT), str(record), "--root", str(self.root)],
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(result.returncode, 1)
        report = json.loads(result.stdout)
        self.assertEqual(report["status"], "invalid")
        self.assertIn("exceeds 1048576 bytes", report["errors"][0])

    def test_rejects_artifacts_over_aggregate_byte_budget(self) -> None:
        with patch.object(validate_candidate_evidence, "MAX_ARTIFACT_BYTES", 2):
            report = validate_candidate_evidence.validate(self.record, self.root)

        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("aggregate artifact byte limit" in error for error in report["errors"]))

    def test_artifacts_at_aggregate_byte_budget_are_accepted(self) -> None:
        total_size = len(self.log_bytes) + len(self.evidence_bytes)
        with patch.object(validate_candidate_evidence, "MAX_ARTIFACT_BYTES", total_size):
            report = validate_candidate_evidence.validate(self.record, self.root)
        self.assertEqual(report["status"], "valid", report["errors"])

        with patch.object(validate_candidate_evidence, "MAX_ARTIFACT_BYTES", total_size - 1):
            report = validate_candidate_evidence.validate(self.record, self.root)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("aggregate artifact byte limit" in error for error in report["errors"]))

    def test_rejects_file_growth_after_size_check(self) -> None:
        log_bytes = b"x"
        evidence_bytes = b"abcde"
        (self.root / "logs" / "check.log").write_bytes(log_bytes)
        evidence_path = self.root / "evidence" / "check.json"
        evidence_path.write_bytes(evidence_bytes)
        self.record["log"]["sha256"] = hashlib.sha256(log_bytes).hexdigest()
        self.record["evidence"][0]["sha256"] = hashlib.sha256(evidence_bytes).hexdigest()

        original_fstat = os.fstat
        identity = evidence_path.stat()
        grew = False

        def grow_after_stat(fd: int):
            nonlocal grew
            file_stat = original_fstat(fd)
            if (not grew and file_stat.st_dev == identity.st_dev
                    and file_stat.st_ino == identity.st_ino):
                with evidence_path.open("ab") as stream:
                    stream.write(b"xy")
                grew = True
            return file_stat

        with (
            patch.object(validate_candidate_evidence, "MAX_ARTIFACT_BYTES", 6),
            patch.object(validate_candidate_evidence.os, "fstat", side_effect=grow_after_stat),
        ):
            report = validate_candidate_evidence.validate(self.record, self.root)

        self.assertTrue(grew)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("aggregate artifact byte limit" in error for error in report["errors"]))

    def test_reports_artifact_read_io_error_as_invalid_evidence(self) -> None:
        class FailingRead:
            def __init__(self, stream):
                self.stream = stream

            def __enter__(self):
                self.stream.__enter__()
                return self

            def __exit__(self, *args):
                return self.stream.__exit__(*args)

            def fileno(self) -> int:
                return self.stream.fileno()

            def read(self, size: int = -1) -> bytes:
                raise OSError("simulated artifact read failure")

        original_open = validate_candidate_evidence._open_regular_file

        def open_with_failing_read(root: Path, relative: Path, field: str):
            if field == "log.path":
                return FailingRead(original_open(root, relative, field))
            return original_open(root, relative, field)

        with patch.object(validate_candidate_evidence, "_open_regular_file", side_effect=open_with_failing_read):
            report = validate_candidate_evidence.validate(self.record, self.root)

        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("log.path cannot be read" in error for error in report["errors"]))


if __name__ == "__main__":
    unittest.main()
