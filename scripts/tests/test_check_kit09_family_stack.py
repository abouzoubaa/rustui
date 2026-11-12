from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))
SPEC = importlib.util.spec_from_file_location(
    "check_kit09_family_stack", ROOT / "scripts/check_kit09_family_stack.py"
)
assert SPEC is not None and SPEC.loader is not None
STACK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(STACK)


class Kit09FamilyStackTests(unittest.TestCase):
    def test_family_order_covers_each_candidate_once(self) -> None:
        family_files = {
            path.name
            for path in STACK.PATCH_DIR.glob("*.patch")
            if path.name not in STACK.CORE_ORDER
        }
        self.assertEqual(set(STACK.FAMILY_ORDER), family_files)
        self.assertEqual(len(STACK.FAMILY_ORDER), len(set(STACK.FAMILY_ORDER)))

    def test_inert_focus_provider_precedes_modal_focus_consumer(self) -> None:
        order = STACK.CORE_ORDER + STACK.FAMILY_ORDER
        self.assertLess(
            order.index("t17-inert-focus-scope-v1-kit08-kit09.patch"),
            order.index("t13-modal-focus-recovery-v1-kit09.patch"),
        )

    def test_sheet_patch_preserves_default_close_name_and_optional_override(self) -> None:
        patch = (
            STACK.PATCH_DIR / "gpui-component-sheet-close-accessibility-v1-kit09.patch"
        ).read_text(encoding="utf-8")
        self.assertIn('.accessibility_label("Close")', patch)
        self.assertIn("self.close_accessibility_label.clone()", patch)


if __name__ == "__main__":
    unittest.main()
