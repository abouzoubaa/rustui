"""Tests for the read-only candidate patch-plan manifest validator."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from io import StringIO
from unittest.mock import patch


SCRIPT = Path(__file__).resolve().parents[1] / "validate_patch_plan.py"
sys.path.insert(0, str(SCRIPT.parent))
SPEC = importlib.util.spec_from_file_location("validate_patch_plan", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
VALIDATOR = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(VALIDATOR)


class ValidatePatchPlanTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.patch_bytes = b"diff --git a/a b/a\n"
        self.checksum_bytes = b'{"files":{}}\n'
        (self.root / "patches").mkdir()
        (self.root / "patches" / "0001-example.patch").write_bytes(self.patch_bytes)
        package_dir = self.root / "vendor" / "example"
        package_dir.mkdir(parents=True)
        (package_dir / ".cargo-checksum.json").write_bytes(self.checksum_bytes)
        (package_dir / "LICENSE-MIT").write_text("MIT license text\n", encoding="utf-8")
        self.manifest = {
            "schema_version": 1,
            "candidate": {
                "id": "candidate-1",
                "snapshot_sha256": "a" * 64,
                "parent_manifest_sha256": "b" * 64,
            },
            "source": {
                "repository": "https://example.invalid/gpui.git",
                "revision": "1" * 40,
                "packages": [
                    {
                        "name": "gpui-example",
                        "version": "0.1.0",
                        "checksum_file": "vendor/example/.cargo-checksum.json",
                        "checksum_sha256": hashlib.sha256(self.checksum_bytes).hexdigest(),
                        "license_expression": "MIT",
                        "license_files": ["vendor/example/LICENSE-MIT"],
                    }
                ],
            },
            "patches": [
                {
                    "path": "patches/0001-example.patch",
                    "sha256": hashlib.sha256(self.patch_bytes).hexdigest(),
                }
            ],
            "gates": {},
        }

    def run_validator(self, manifest: dict | str) -> subprocess.CompletedProcess[str]:
        manifest_path = self.root / "manifest.json"
        if isinstance(manifest, str):
            manifest_path.write_text(manifest, encoding="utf-8")
        else:
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
        return subprocess.run(
            [sys.executable, str(SCRIPT), str(manifest_path), "--root", str(self.root)],
            capture_output=True,
            check=False,
            text=True,
        )

    def report(self, manifest: dict) -> dict:
        process = self.run_validator(manifest)
        return json.loads(process.stdout)

    def test_valid_manifest_reports_pinned_identity_and_not_run_gates(self) -> None:
        process = self.run_validator(self.manifest)
        report = json.loads(process.stdout)
        self.assertEqual(process.returncode, 0)
        self.assertEqual(report["status"], "valid")
        self.assertEqual(report["candidate_id"], "candidate-1")
        self.assertEqual(report["source_revision"], "1" * 40)
        self.assertEqual(report["patch_count"], 1)
        self.assertEqual(
            report["gates"],
            {"build": "not_run", "correctness": "not_run", "gui": "not_run", "performance": "not_run"},
        )

        without_gate_input = dict(self.manifest)
        without_gate_input.pop("gates")
        report = self.report(without_gate_input)
        self.assertEqual(report["status"], "valid")
        self.assertTrue(all(state == "not_run" for state in report["gates"].values()))

    def test_candidate_id_matches_cache_workspace_identifier_rules(self) -> None:
        accepted = dict(self.manifest)
        accepted["candidate"] = dict(self.manifest["candidate"], id="Release_2026.10")
        self.assertEqual(self.report(accepted)["status"], "valid")

        for candidate_id in ("a..b", "a" * 65, "_invalid"):
            with self.subTest(candidate_id=candidate_id):
                changed = json.loads(json.dumps(self.manifest))
                changed["candidate"]["id"] = candidate_id
                self.assertEqual(self.report(changed)["status"], "invalid")

    def test_hashing_reads_files_in_bounded_chunks(self) -> None:
        payload = b"x" * (VALIDATOR._HASH_READ_CHUNK_SIZE * 2 + 3)
        target = self.root / "large.patch"
        target.write_bytes(payload)

        class ReadSpy:
            def __init__(self, stream) -> None:
                self.stream = stream
                self.read_sizes: list[int] = []

            def read(self, size: int = -1) -> bytes:
                self.read_sizes.append(size)
                return self.stream.read(size)

            def fileno(self) -> int:
                return self.stream.fileno()

            def __enter__(self):
                return self

            def __exit__(self, *args):
                return self.stream.__exit__(*args)

        streams = []

        def open_file(root, relative):
            stream = ReadSpy((Path(root) / relative).open("rb"))
            streams.append(stream)
            return stream

        with patch.object(VALIDATOR, "_open_contained_file", side_effect=open_file):
            actual = VALIDATOR._hash_contained_file(self.root, Path("large.patch"))

        self.assertEqual(actual, hashlib.sha256(payload).hexdigest())
        self.assertEqual(
            streams[0].read_sizes,
            [VALIDATOR._HASH_READ_CHUNK_SIZE] * 4,
        )

    def test_patch_and_checksum_hashes_use_exact_file_bytes(self) -> None:
        changed = dict(self.manifest)
        changed["patches"] = [dict(self.manifest["patches"][0], sha256="0" * 64)]
        report = self.report(changed)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("patches[0].sha256" in error for error in report["errors"]))

        changed = json.loads(json.dumps(self.manifest))
        changed["source"]["packages"][0]["checksum_sha256"] = "0" * 64
        report = self.report(changed)
        self.assertTrue(any("checksum_sha256" in error for error in report["errors"]))

    def test_rejects_patch_replaced_while_its_digest_is_read(self) -> None:
        patch_path = self.root / self.manifest["patches"][0]["path"]
        original_open = VALIDATOR._open_contained_file
        replaced = False

        class ReplacingStream:
            def __init__(self, stream):
                self.stream = stream

            def __enter__(self):
                return self

            def __exit__(self, *args):
                return self.stream.__exit__(*args)

            def fileno(self):
                return self.stream.fileno()

            def read(self, size=-1):
                nonlocal replaced
                data = self.stream.read(size)
                if not data and not replaced:
                    patch_path.write_bytes(b"replaced during digest read\n")
                    replaced = True
                return data

        def open_file(root, relative):
            stream = original_open(root, relative)
            if relative.as_posix() == self.manifest["patches"][0]["path"]:
                return ReplacingStream(stream)
            return stream

        with patch.object(VALIDATOR, "_open_contained_file", side_effect=open_file):
            report = VALIDATOR.validate(self.manifest, self.root)

        self.assertTrue(replaced)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("changed while hashing" in error for error in report["errors"]))

    def test_rejects_traversal_absolute_and_nul_paths(self) -> None:
        for path in ("../outside.patch", "/tmp/outside.patch", "C:/outside.patch", "bad\x00path"):
            with self.subTest(path=path):
                changed = json.loads(json.dumps(self.manifest))
                changed["patches"][0]["path"] = path
                report = self.report(changed)
                self.assertEqual(report["status"], "invalid")

    def test_rejects_symlink_escape(self) -> None:
        outside = Path(self.temp.name).parent / (Path(self.temp.name).name + "-outside.patch")
        outside.write_bytes(self.patch_bytes)
        self.addCleanup(lambda: outside.unlink(missing_ok=True))
        (self.root / "patches" / "escape.patch").symlink_to(outside)
        changed = json.loads(json.dumps(self.manifest))
        changed["patches"][0]["path"] = "patches/escape.patch"
        changed["patches"][0]["sha256"] = hashlib.sha256(self.patch_bytes).hexdigest()
        report = self.report(changed)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("escapes root" in error for error in report["errors"]))

    def test_source_root_path_swap_does_not_redirect_reads(self) -> None:
        original_root = self.root.with_name(f"{self.root.name}-pinned")
        original_contained_file = VALIDATOR._contained_file
        swapped = False

        def swap_after_check(
            root: object, relative: Path, field: str, errors: list[str]
        ) -> Path | None:
            nonlocal swapped
            resolved = original_contained_file(root, relative, field, errors)
            if field == "source.packages[0].checksum_file" and resolved is not None:
                self.root.rename(original_root)
                (self.root / "patches").mkdir(parents=True)
                (self.root / "patches" / "0001-example.patch").write_bytes(b"decoy patch\n")
                decoy_package = self.root / "vendor" / "example"
                decoy_package.mkdir(parents=True)
                (decoy_package / ".cargo-checksum.json").write_bytes(b"decoy checksum\n")
                (decoy_package / "LICENSE-MIT").write_text("decoy license\n", encoding="utf-8")
                swapped = True
            return resolved

        try:
            with patch.object(VALIDATOR, "_contained_file", side_effect=swap_after_check):
                report = VALIDATOR.validate(self.manifest, self.root)
            self.assertTrue(swapped)
            self.assertEqual(report["status"], "valid", report["errors"])
            self.assertEqual(
                (self.root / "patches" / "0001-example.patch").read_bytes(), b"decoy patch\n"
            )
        finally:
            if self.root.exists():
                shutil.rmtree(self.root)
            if original_root.exists():
                original_root.rename(self.root)

    def test_inherited_source_root_fd_survives_repo_path_swap_before_manifest_read(self) -> None:
        repo_temp = tempfile.TemporaryDirectory()
        self.addCleanup(repo_temp.cleanup)
        repo = Path(repo_temp.name)
        candidate_relative = Path("target/headstart/candidates/candidate-1")
        candidate_root = repo / candidate_relative
        candidate_root.mkdir(parents=True)
        shutil.copytree(self.root / "patches", candidate_root / "patches")
        shutil.copytree(self.root / "vendor", candidate_root / "vendor")
        plan_path = candidate_root / "plan.json"
        plan_path.write_text(json.dumps(self.manifest), encoding="utf-8")

        repo_fd = os.open(repo, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
        source_fd = os.open(candidate_root, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
        parked_root = candidate_root.with_name("candidate-1-pinned")
        original_load_json = VALIDATOR._load_json
        swapped = False

        def swap_after_identity_check(root: object, relative: Path) -> dict:
            nonlocal swapped
            candidate_root.rename(parked_root)
            candidate_root.mkdir()
            (candidate_root / "plan.json").write_text("{}", encoding="utf-8")
            swapped = True
            return original_load_json(root, relative)

        argv = [
            str(SCRIPT),
            (candidate_relative / "plan.json").as_posix(),
            "--root",
            candidate_relative.as_posix(),
        ]
        try:
            with (
                patch.object(VALIDATOR.sys, "argv", argv),
                patch.dict(
                    VALIDATOR.os.environ,
                    {
                        "RUSTUI_P3_SOURCE_ROOT_FD": str(source_fd),
                        "RUSTUI_P3_REPO_ROOT_FD": str(repo_fd),
                    },
                ),
                patch.object(VALIDATOR, "_load_json", side_effect=swap_after_identity_check),
                redirect_stdout(StringIO()) as output,
            ):
                result = VALIDATOR.main()

            self.assertTrue(swapped)
            self.assertEqual(result, 0, output.getvalue())
            self.assertEqual(json.loads(output.getvalue())["status"], "valid")
        finally:
            os.close(source_fd)
            os.close(repo_fd)
            if candidate_root.exists():
                shutil.rmtree(candidate_root)
            if parked_root.exists():
                parked_root.rename(candidate_root)

    def test_rejects_duplicate_patch_paths_and_missing_license_metadata(self) -> None:
        changed = json.loads(json.dumps(self.manifest))
        changed["patches"].append(dict(changed["patches"][0]))
        changed["source"]["packages"][0]["license_files"] = []
        report = self.report(changed)
        self.assertTrue(any("duplicate patch path" in error for error in report["errors"]))
        self.assertTrue(any("license_files must be a nonempty array" in error for error in report["errors"]))

        changed = json.loads(json.dumps(self.manifest))
        changed["source"]["packages"][0]["checksum_file"] = "vendor/missing/.cargo-checksum.json"
        changed["source"]["packages"][0]["license_files"] = ["vendor/example/MISSING-LICENSE"]
        report = self.report(changed)
        self.assertTrue(any("checksum_file does not exist" in error for error in report["errors"]))
        self.assertTrue(any("license_files[0] does not exist" in error for error in report["errors"]))

    def test_rejects_patch_symlink_alias_to_same_resolved_file(self) -> None:
        (self.root / "patches" / "alias.patch").symlink_to("0001-example.patch")
        changed = json.loads(json.dumps(self.manifest))
        changed["patches"].append(
            {
                "path": "patches/alias.patch",
                "sha256": hashlib.sha256(self.patch_bytes).hexdigest(),
            }
        )

        report = self.report(changed)

        self.assertEqual(report["status"], "invalid")
        self.assertTrue(
            any("patches[1].path escapes root" in error for error in report["errors"])
        )

    def test_rejects_patch_hardlink_alias_to_same_file(self) -> None:
        original = self.root / "patches" / "0001-example.patch"
        alias = self.root / "patches" / "hardlink.patch"
        alias.hardlink_to(original)
        changed = json.loads(json.dumps(self.manifest))
        changed["patches"].append(
            {
                "path": "patches/hardlink.patch",
                "sha256": hashlib.sha256(self.patch_bytes).hexdigest(),
            }
        )

        report = self.report(changed)

        self.assertEqual(report["status"], "invalid")
        self.assertTrue(
            any("duplicate patch file identity: patches/hardlink.patch" in error
                for error in report["errors"])
        )

    def test_rejects_gate_promotion_and_duplicate_json_keys(self) -> None:
        changed = json.loads(json.dumps(self.manifest))
        changed["gates"] = {"build": "passed"}
        report = self.report(changed)
        self.assertTrue(any("must remain not_run" in error for error in report["errors"]))

        process = self.run_validator('{"schema_version":1,"schema_version":1}')
        self.assertEqual(process.returncode, 1)
        report = json.loads(process.stdout)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("duplicate JSON key" in error for error in report["errors"]))

        for constant in ("NaN", "Infinity", "-Infinity"):
            with self.subTest(constant=constant):
                process = self.run_validator(f'{{"schema_version":{constant}}}')
                self.assertEqual(process.returncode, 1)
                report = json.loads(process.stdout)
                self.assertEqual(report["status"], "invalid")
                self.assertTrue(
                    any("nonstandard JSON constant" in error for error in report["errors"])
                )

        changed = json.loads(json.dumps(self.manifest))
        changed["misspelled_field"] = True
        report = self.report(changed)
        self.assertTrue(any("only gates is optional" in error for error in report["errors"]))

    def test_manifest_file_must_resolve_inside_root(self) -> None:
        outside_manifest = self.root.parent / f"{self.root.name}-outside.json"
        outside_manifest.write_text(json.dumps(self.manifest), encoding="utf-8")
        self.addCleanup(lambda: outside_manifest.unlink(missing_ok=True))
        process = subprocess.run(
            [sys.executable, str(SCRIPT), str(outside_manifest), "--root", str(self.root)],
            capture_output=True,
            check=False,
            text=True,
        )
        report = json.loads(process.stdout)
        self.assertEqual(process.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertIn("manifest file must resolve inside root", report["errors"][0])


if __name__ == "__main__":
    unittest.main()
