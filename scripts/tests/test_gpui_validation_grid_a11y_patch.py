#!/usr/bin/env python3
"""Verify exact replay and rejection of partial modal focus vendor images."""
import contextlib
import importlib.util
import io
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "validation_grid_a11y_patch", ROOT / "scripts/apply_gpui_validation_grid_a11y_patch.py"
)
HELPER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(HELPER)


class ValidationGridA11yPatchTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name) / "source"
        self.artifacts = Path(self.temporary.name) / "artifacts"
        self.meta = json.loads((ROOT / HELPER.MANIFEST).read_text())
        for relative in self.meta["files"]:
            source = ROOT / relative
            target = self.root / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, target)
        for relative in (HELPER.MANIFEST, Path(self.meta["patch"])):
            target = self.artifacts / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / relative, target)
        subprocess.run([sys.executable, str(ROOT / "scripts/apply_r15_test_tree_disabled_v1_patch.py"), "--root", str(self.root), "--artifacts", str(ROOT), "--reverse"], check=True, capture_output=True)
        subprocess.run([sys.executable, str(ROOT / "scripts/apply_gpui_primary_mousedown_click_v2_patch.py"), "--root", str(self.root), "--artifacts", str(ROOT), "--reverse"], check=True, capture_output=True)
        if HELPER.digest((self.root / next(iter(self.meta["files"]))).read_bytes()) == next(iter(self.meta["files"].values()))["post"]:
            self.run_mode("--reverse")
        self.pre = {name: (self.root / name).read_bytes() for name in self.meta["files"]}
        for package in ("gpui-pre-0.3.9",):
            (self.root / "vendor" / package / ".cargo-checksum.json").write_bytes(b"original checksum metadata")

    def run_mode(self, mode):
        with contextlib.redirect_stdout(io.StringIO()):
            HELPER.run(self.root, self.artifacts, mode)

    def test_check_apply_verify_reverse_round_trip_preserves_checksums(self):
        self.run_mode("--check")
        with self.assertRaisesRegex(ValueError, "POST required"):
            self.run_mode("--verify")
        self.run_mode("--apply")
        self.run_mode("--verify")
        self.run_mode("--check")
        self.run_mode("--reverse")
        self.run_mode("--check")
        for package in ("gpui-pre-0.3.9",):
            self.assertEqual((self.root / "vendor" / package / ".cargo-checksum.json").read_bytes(), b"original checksum metadata")

    def test_rejects_unknown_or_partial_images(self):
        self.run_mode("--apply")
        relative, images = next(iter(self.meta["files"].items()))
        source = self.root / relative
        source.write_bytes(source.read_bytes() + b"drift")
        with self.assertRaisesRegex(ValueError, "unknown source image"):
            self.run_mode("--check")
        source.write_bytes(self.pre[relative])
        with self.assertRaisesRegex(ValueError, "partial validation grid accessibility patch state"):
            self.run_mode("--check")

    def test_rejects_modified_patch(self):
        path = self.artifacts / self.meta["patch"]
        path.write_bytes(path.read_bytes() + b"drift")
        with self.assertRaisesRegex(ValueError, "patch hash mismatch"):
            self.run_mode("--check")

    def test_rolls_back_when_install_fails(self):
        before = {name: (self.root / name).read_bytes() for name in self.meta["files"]}
        original = HELPER.install
        calls = 0

        def fail_once(path, data):
            nonlocal calls
            calls += 1
            if calls == 2:
                raise OSError("injected installation failure")
            original(path, data)

        with patch.object(HELPER, "install", side_effect=fail_once):
            with self.assertRaisesRegex(OSError, "injected"):
                self.run_mode("--apply")
        self.assertEqual(before, {name: (self.root / name).read_bytes() for name in before})


if __name__ == "__main__":
    unittest.main()
