from __future__ import annotations

import hashlib
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from scripts.apply_gpui_macos_popup_child_teardown_patch import apply_patch, verify_applied


ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "patches/approved/gpui-macos-popup-child-teardown-v1.json"


class PopupChildTeardownPatchTests(unittest.TestCase):
    def test_patch_applies_and_reverse_restores_owned_and_vendor_sources(self) -> None:
        manifest = json.loads(MANIFEST.read_text())
        with tempfile.TemporaryDirectory() as temporary:
            fixture = Path(temporary)
            for relative in manifest["files"]:
                target = fixture / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(ROOT / relative, target)

            self.assertEqual(apply_patch(fixture, reverse=True), "reversed")
            self.assertEqual(apply_patch(fixture, check_only=True), "verified")
            self.assertEqual(apply_patch(fixture), "applied")
            self.assertEqual(verify_applied(fixture), "verified")
            self.assertEqual(apply_patch(fixture, reverse=True, check_only=True), "verified")
            self.assertEqual(apply_patch(fixture, reverse=True), "reversed")

            for relative, hashes in manifest["files"].items():
                digest = hashlib.sha256((fixture / relative).read_bytes()).hexdigest()
                self.assertEqual(digest, hashes["pre_sha256"], relative)


if __name__ == "__main__":
    unittest.main()
