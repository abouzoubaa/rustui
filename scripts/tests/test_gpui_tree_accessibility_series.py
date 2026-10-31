"""Exact three-source approved Tree lifecycle; disposable fixtures, no Cargo/apps."""
from __future__ import annotations

import hashlib
from pathlib import Path
import runpy
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[2]
SERIES_SCRIPT = Path("scripts/apply_gpui_tree_accessibility_series.py")
BASELINE_SCRIPT = Path("scripts/apply_gpui_tree_aria_disabled_patch.py")
LEVEL_SCRIPT = Path("scripts/apply_gpui_tree_aria_level_patch.py")
SERIES = runpy.run_path(str(ROOT / SERIES_SCRIPT))


class TreeAccessibilitySeriesLifecycle(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="rustui-tree-series-test-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        support = set(SERIES["FILES"]) | set(SERIES["PROVENANCE"]) | set(SERIES["DISABLED_FILES"]) | set(SERIES["IDENTITY"]) | set(SERIES["PATCH_HASHES"])
        support |= {Path("Cargo.toml"), Path("Cargo.lock"), SERIES_SCRIPT, BASELINE_SCRIPT, LEVEL_SCRIPT}
        for relative in support:
            relative = Path(relative)
            target = self.root / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / relative, target)
        current = self.sources()
        hashes = {path: SERIES["digest"](data) for path, data in current.items()}
        if hashes == SERIES["POST"]:
            current = SERIES["transform"](self.root, current, reverse=True)
            for relative, contents in current.items():
                (self.root / relative).write_bytes(contents)
        elif hashes != SERIES["PRE"]:
            raise AssertionError("test fixture requires exact PRE or POST three-source images")
        self.preimages = self.sources()
        self.provenance = {Path(path): (self.root / path).read_bytes() for path in SERIES["PROVENANCE"]}

    def sources(self):
        return {path: (self.root / path).read_bytes() for path in SERIES["FILES"]}

    def command(self, mode, script=SERIES_SCRIPT, accepted=True):
        result = subprocess.run([sys.executable, str(self.root / script), mode, "--root", str(self.root)], capture_output=True, text=True, check=False)
        if accepted:
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        return result

    def assert_provenance_preserved(self):
        for relative, data in self.provenance.items():
            self.assertEqual((self.root / relative).read_bytes(), data, str(relative))

    def test_pre_check_apply_post_verify_composed_baseline_reverse(self):
        self.command("--check")
        self.command("--verify", accepted=False)
        self.assertEqual(self.sources(), self.preimages)
        self.command("--apply")
        self.assertEqual({path: SERIES["digest"](data) for path, data in self.sources().items()}, SERIES["POST"])
        postimages = self.sources()
        self.command("--verify")
        self.command("--check", accepted=False)
        self.assertEqual(self.sources(), postimages, "POST --verify must be read-only")
        self.command("--check", BASELINE_SCRIPT)
        self.command("--reverse", BASELINE_SCRIPT, accepted=False)
        self.assertEqual(self.sources(), postimages, "composed baseline check must be read-only")
        self.command("--reverse")
        self.assertEqual(self.sources(), self.preimages)
        self.command("--check", BASELINE_SCRIPT)
        self.assert_provenance_preserved()

    def test_legacy_level_entry_point_is_read_only_for_pre_and_post(self):
        self.command("--check", LEVEL_SCRIPT)
        self.assertEqual(self.sources(), self.preimages)
        self.command("--apply", LEVEL_SCRIPT, accepted=False)
        self.command("--reverse", LEVEL_SCRIPT, accepted=False)
        self.assertEqual(self.sources(), self.preimages)
        self.command("--apply")
        postimages = self.sources()
        self.command("--check", LEVEL_SCRIPT)
        self.assertEqual(self.sources(), postimages)
        self.assert_provenance_preserved()

    def test_unknown_preimage_rejects_check_and_apply_without_writes(self):
        (self.root / SERIES["COMPONENT"]).write_bytes(self.preimages[SERIES["COMPONENT"]] + b"\n// unknown\n")
        unknown = self.sources()
        self.command("--check", accepted=False)
        self.command("--apply", accepted=False)
        self.assertEqual(self.sources(), unknown)
        self.assert_provenance_preserved()

    def test_unknown_postimage_rejects_verify_and_reverse_without_writes(self):
        self.command("--apply")
        path = self.root / SERIES["TEST_WINDOW"]
        path.write_bytes(path.read_bytes() + b"\n// unknown\n")
        unknown = self.sources()
        self.command("--verify", accepted=False)
        self.command("--reverse", accepted=False)
        self.assertEqual(self.sources(), unknown)
        self.assert_provenance_preserved()

    def test_baseline_verifier_rejects_partial_composition(self):
        self.command("--apply")
        (self.root / SERIES["COMPONENT"]).write_bytes(self.preimages[SERIES["COMPONENT"]])
        partial = self.sources()
        self.command("--check", BASELINE_SCRIPT, accepted=False)
        self.assertEqual(self.sources(), partial)

    def test_patch_artifact_change_rejected_before_source_writes(self):
        path = self.root / SERIES["ARIA_LEVEL_PATCH"]
        path.write_bytes(path.read_bytes() + b"\n")
        self.command("--apply", accepted=False)
        self.assertEqual(self.sources(), self.preimages)

    def test_three_source_install_rolls_back_after_partial_replace_failure(self):
        updated = SERIES["transform"](self.root, self.preimages, reverse=False)
        original_replace = SERIES["os"].replace
        calls = 0

        def fail_second_replace(source, target):
            nonlocal calls
            calls += 1
            if calls == 2:
                raise OSError("fixture injected second source replacement failure")
            return original_replace(source, target)

        with mock.patch.object(SERIES["os"], "replace", side_effect=fail_second_replace):
            with self.assertRaises(OSError):
                SERIES["install"](self.root, updated)
        self.assertEqual(self.sources(), self.preimages)
        self.assertFalse(list(self.root.rglob("*.tree-a11y-tmp")))
        self.assertFalse(list(self.root.rglob("*.tree-a11y-rollback")))
        self.assert_provenance_preserved()


if __name__ == "__main__":
    unittest.main()
