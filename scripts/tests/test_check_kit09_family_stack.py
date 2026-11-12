from __future__ import annotations

import importlib.util
import sys
import tempfile
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

    def test_slider_resize_family_follows_the_tree_grid_family(self) -> None:
        order = STACK.CORE_ORDER + STACK.FAMILY_ORDER
        self.assertLess(
            order.index("gpui-component-table-delegate-accessibility-v1-kit09.patch"),
            order.index("gpui-base-slider-endpoint-v1-kit09.patch"),
        )
        self.assertEqual(
            STACK.A3_4D_ORDER,
            (
                "gpui-base-slider-endpoint-v1-kit09.patch",
                "gpui-base-slider-first-move-v1.patch",
                "gpui-base-slider-set-value-v2-kit09.patch",
                "gpui-base-slider-zero-track-v1.patch",
                "gpui-base-resize-first-move-v1-kit08.patch",
                "gpui-base-resize-first-move-v3-kit08.patch",
            ),
        )

    def test_inert_focus_provider_precedes_modal_focus_consumer(self) -> None:
        order = STACK.CORE_ORDER + STACK.FAMILY_ORDER
        self.assertLess(
            order.index("t17-inert-focus-scope-v1-kit08-kit09.patch"),
            order.index("t13-modal-focus-recovery-v1-kit09.patch"),
        )

    def test_menu_select_provider_order_is_preserved(self) -> None:
        order = STACK.CORE_ORDER + STACK.FAMILY_ORDER
        self.assertLess(order.index("gpui-component-popup-menu-disabled-confirm-v1-kit08.patch"), order.index("r15b-menu-item-disabled-v1-kit09.patch"))
        self.assertLess(order.index("r15b-menu-item-disabled-v1-kit09.patch"), order.index("r15b-popup-menu-recovery-v1-kit09.patch"))

    def test_tree_aria_provider_precedes_tree_component_consumers(self) -> None:
        order = STACK.CORE_ORDER + STACK.FAMILY_ORDER
        provider = order.index("gpui-tree-aria-disabled-v1-kit08.patch")
        self.assertLess(provider, order.index("gpui-tree-aria-disabled-row-v1-kit08.patch"))
        self.assertLess(provider, order.index("gpui-tree-disabled-keyboard-v2.patch"))

    def test_grid_debug_metadata_follows_disabled_tree_metadata(self) -> None:
        order = STACK.CORE_ORDER
        self.assertLess(
            order.index("r15-test-tree-disabled-v1-kit08.patch"),
            order.index("a11y-grid-readonly-sort-v1-kit09.patch"),
        )

    def test_sheet_patch_preserves_default_close_name_and_optional_override(self) -> None:
        patch = (
            STACK.PATCH_DIR / "gpui-component-sheet-close-accessibility-v1-kit09.patch"
        ).read_text(encoding="utf-8")
        self.assertIn('.accessibility_label("Close")', patch)
        self.assertIn("self.close_accessibility_label.clone()", patch)

    def test_historical_targets_for_one_source_share_stack_changes(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            stage = Path(temporary)
            old = "vendor/gpui-pre-0.3.9/src/elements/div.rs"
            new = "vendor/gpui-pre-0.3.10/src/elements/div.rs"
            (stage / old).parent.mkdir(parents=True)
            (stage / new).parent.mkdir(parents=True)
            (stage / old).write_text("base\n", encoding="utf-8")
            (stage / new).write_text("base\n", encoding="utf-8")
            sources = {
                old: ("gpui-pre", "src/elements/div.rs"),
                new: ("gpui-pre", "src/elements/div.rs"),
            }

            (stage / old).write_text("base\nprimary change\n", encoding="utf-8")
            STACK.check_kit09_core_stack.sync_source_aliases(stage, sources, [old])
            self.assertEqual((stage / new).read_text(encoding="utf-8"), "base\nprimary change\n")

            (stage / new).write_text("base\nprimary change\naria disabled\n", encoding="utf-8")
            STACK.check_kit09_core_stack.sync_source_aliases(stage, sources, [new])
            expected = "base\nprimary change\naria disabled\n"
            self.assertEqual((stage / old).read_text(encoding="utf-8"), expected)
            self.assertEqual((stage / new).read_text(encoding="utf-8"), expected)


if __name__ == "__main__":
    unittest.main()
