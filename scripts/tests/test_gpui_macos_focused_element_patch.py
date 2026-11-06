#!/usr/bin/env python3
"""Isolated GPUI macOS focused-element lifecycle and provenance regressions."""
from pathlib import Path
import contextlib
import importlib.util
import io
import json
import os
import shutil
import tempfile
import unittest
from unittest.mock import patch

BUNDLE = Path(__file__).resolve().parents[2]
SOURCE_ROOT = Path(os.environ.get("RUSTUI_PATCH_TEST_SOURCE_ROOT", BUNDLE)).resolve()
HELPER_PATH = BUNDLE / "scripts/apply_gpui_macos_focused_element_patch.py"
SPEC = importlib.util.spec_from_file_location("gpui_macos_focused_element_patch", HELPER_PATH)
HELPER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(HELPER)
MANIFEST = HELPER.MANIFEST


class FocusedElementPatch(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        temporary = Path(self.temp.name)
        self.root = temporary / "workspace"
        self.artifacts = temporary / "artifacts"
        self.empty_cargo_home = temporary / "empty-cargo-home"
        self.root.mkdir()
        self.empty_cargo_home.mkdir()

        shutil.copy2(SOURCE_ROOT / "Cargo.toml", self.root / "Cargo.toml")
        shutil.copy2(SOURCE_ROOT / "Cargo.lock", self.root / "Cargo.lock")
        for package in ("gpui-pre-macos-0.3.9", "accesskit_macos-0.26.3"):
            shutil.copytree(SOURCE_ROOT / "vendor" / package, self.root / "vendor" / package)

        metadata = json.loads((BUNDLE / MANIFEST).read_text())
        files = {MANIFEST, Path(metadata["patch"])}
        files.update(Path(item["path"]) for item in metadata["accesskit_macos"]["license_files"])
        for relative in files:
            source = BUNDLE / relative
            destination = self.artifacts / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, destination)
        self.metadata = metadata
        source = self.root / metadata["source"]
        image = HELPER.hash_file(source)
        if image == metadata["post"]:
            self.run_mode("--reverse")
            image = HELPER.hash_file(source)
        if image != metadata["pre"]:
            raise ValueError("fixture source is neither the exact PRE nor POST image")

    def run_mode(self, mode):
        with patch.dict(os.environ, {"CARGO_HOME": str(self.empty_cargo_home)}):
            with contextlib.redirect_stdout(io.StringIO()):
                HELPER.run(self.root, self.artifacts, mode)

    def source(self):
        return self.root / self.metadata["source"]

    def test_pre_apply_verify_reverse_lifecycle(self):
        self.run_mode("--check")
        with self.assertRaisesRegex(ValueError, "exact POST required"):
            self.run_mode("--verify")
        self.run_mode("--apply")
        self.assertEqual(HELPER.hash_file(self.source()), self.metadata["post"])
        self.run_mode("--verify")
        self.run_mode("--reverse")
        self.assertEqual(HELPER.hash_file(self.source()), self.metadata["pre"])
        self.run_mode("--check")

    def test_unknown_source_rejected(self):
        self.source().write_bytes(self.source().read_bytes() + b"\n// foreign source\n")
        with self.assertRaisesRegex(ValueError, "unknown or partial GPUI macOS source"):
            self.run_mode("--check")

    def test_registry_source_selection_rejected(self):
        cargo = self.root / "Cargo.toml"
        content = cargo.read_text()
        selected = f'gpui-pre-macos = {{ path = "{self.metadata["gpui"]["cargo_path"]}" }}'
        self.assertIn(selected, content)
        cargo.write_text(content.replace(selected, 'gpui-pre-macos = { git = "https://example.invalid/zed" }'))
        with self.assertRaisesRegex(ValueError, "must select the pinned local GPUI macOS package"):
            self.run_mode("--check")

    def test_tampered_license_rejected(self):
        license_path = self.artifacts / self.metadata["accesskit_macos"]["license_files"][0]["path"]
        license_path.write_bytes(license_path.read_bytes() + b"\n")
        with self.assertRaisesRegex(ValueError, "license provenance mismatch"):
            self.run_mode("--check")

    def test_tampered_vendored_api_rejected(self):
        api = self.root / self.metadata["accesskit_macos"]["vendored_source"] / "src/patch.rs"
        api.write_bytes(api.read_bytes() + b"\n// foreign source\n")
        with self.assertRaisesRegex(ValueError, "checksum mismatch|implementation changed"):
            self.run_mode("--check")

    def test_rollback_after_post_replace_failure(self):
        before = self.source().read_bytes()
        original_replace = HELPER.os.replace
        calls = 0

        def replace_then_fail(source, destination):
            nonlocal calls
            calls += 1
            original_replace(source, destination)
            if calls == 1:
                raise OSError("injected failure after atomic replace")

        with patch.object(HELPER.os, "replace", side_effect=replace_then_fail):
            with self.assertRaisesRegex(OSError, "injected failure"):
                self.run_mode("--apply")
        self.assertEqual(self.source().read_bytes(), before)
        self.assertEqual(calls, 2)
        self.assertFalse(list(self.source().parent.glob(".gpui-focused-element-*")))


if __name__ == "__main__":
    unittest.main()
