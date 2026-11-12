from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))
SPEC = importlib.util.spec_from_file_location(
    "check_kit09_core_stack", ROOT / "scripts/check_kit09_core_stack.py"
)
assert SPEC is not None and SPEC.loader is not None
STACK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(STACK)


class Kit09CoreStackTests(unittest.TestCase):
    def test_candidate_order_covers_each_patch_once(self) -> None:
        patch_files = {path.name for path in STACK.DEFAULT_PATCH_DIR.glob("*.patch")} - STACK.LATER_FAMILY_CANDIDATES
        self.assertEqual(set(STACK.CANDIDATE_ORDER), patch_files)
        self.assertEqual(len(STACK.CANDIDATE_ORDER), len(set(STACK.CANDIDATE_ORDER)))

    def test_provider_patches_precede_consumers(self) -> None:
        positions = {name: index for index, name in enumerate(STACK.CANDIDATE_ORDER)}
        self.assertLess(
            positions["g10-parent-node-id-v1.patch"],
            positions["g10-labelled-by-tree-v1-kit08.patch"],
        )
        self.assertLess(
            positions["t17-inert-focus-scope-v1-kit08-kit09.patch"],
            positions["gpui-tree-aria-disabled-v1-kit08.patch"],
        )


if __name__ == "__main__":
    unittest.main()
