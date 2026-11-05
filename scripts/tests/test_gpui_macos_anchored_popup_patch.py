from __future__ import annotations

import shutil
import tempfile
import unittest
from pathlib import Path

from scripts.apply_gpui_macos_anchored_popup_patch import (
    PATCH,
    POSTIMAGES,
    PREIMAGES,
    PatchError,
    apply_patch,
    sha256,
)


ROOT = Path(__file__).resolve().parents[2]


class AnchoredPopupPatchTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory(prefix="rustui-popup-patch-test-")
        self.root = Path(self.temp.name)
        for relative in PREIMAGES:
            destination = self.root / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / relative, destination)

    def tearDown(self) -> None:
        self.temp.cleanup()

    def test_exact_patch_applies_and_is_idempotent(self) -> None:
        self.assertEqual(apply_patch(self.root), "applied")
        self.assertEqual(
            {relative: sha256(self.root / relative) for relative in PREIMAGES},
            POSTIMAGES,
        )
        self.assertEqual(apply_patch(self.root), "already-applied")

    def test_check_mode_does_not_change_sources(self) -> None:
        before = {relative: sha256(self.root / relative) for relative in PREIMAGES}
        self.assertEqual(apply_patch(self.root, check_only=True), "verified")
        self.assertEqual(before, {relative: sha256(self.root / relative) for relative in PREIMAGES})

    def test_screen_conversion_uses_anchor_views_native_window(self) -> None:
        patch = PATCH.read_text(encoding="utf-8")
        self.assertIn("let anchor_window: id = msg_send![parent_view, window];", patch)
        self.assertIn("msg_send![anchor_window, convertRectToScreen: anchor_in_window]", patch)

    def test_reverse_restores_exact_preimages(self) -> None:
        apply_patch(self.root)
        self.assertEqual(apply_patch(self.root, reverse=True), "reversed")
        self.assertEqual(
            {relative: sha256(self.root / relative) for relative in PREIMAGES},
            PREIMAGES,
        )

    def test_unknown_source_is_preserved_and_rejected(self) -> None:
        source = self.root / next(iter(PREIMAGES))
        source.write_bytes(source.read_bytes() + b"\nlocal change\n")
        before = source.read_bytes()
        with self.assertRaises(PatchError):
            apply_patch(self.root)
        self.assertEqual(source.read_bytes(), before)

    def test_approved_patch_is_present(self) -> None:
        self.assertTrue(PATCH.is_file())


if __name__ == "__main__":
    unittest.main()
