"""Tests for the read-only candidate patch-plan manifest validator."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "validate_patch_plan.py"


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
            any("duplicate patch path: patches/alias.patch" in error for error in report["errors"])
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
