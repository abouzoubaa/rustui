from __future__ import annotations

import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))
from fixture_target_dir import FixtureTargetError, fixture_target_root  # noqa: E402


class FixtureTargetDirTests(unittest.TestCase):
    def test_kit08_and_kit09_targets_are_separate_from_workspace_target(self) -> None:
        workspace = Path.home() / ".cache" / "rustui-target"
        kit08 = fixture_target_root(8, workspace)
        kit09 = fixture_target_root(9, workspace)
        self.assertNotEqual(kit08, workspace.resolve())
        self.assertNotEqual(kit09, workspace.resolve())
        self.assertNotEqual(kit08, kit09)
        self.assertNotIn(workspace.resolve(), kit08.parents)
        self.assertNotIn(workspace.resolve(), kit09.parents)

    def test_rejects_target_overlap(self) -> None:
        target = Path.home() / ".cache" / "rustui-target-fixture-kit9"
        with self.assertRaises(FixtureTargetError):
            fixture_target_root(9, target)

    def test_building_stack_scripts_use_isolated_target_helper(self) -> None:
        scripts = (
            ROOT / "scripts/check_kit08_kit_stack.py",
            ROOT / "scripts/check_kit09_core_stack.py",
            ROOT / "scripts/check_kit09_family_stack.py",
        )
        for script in scripts:
            with self.subTest(script=script.name):
                source = script.read_text(encoding="utf-8")
                self.assertIn("fixture_target_root", source)

    def test_rejects_unsupported_kit_version(self) -> None:
        with self.assertRaises(FixtureTargetError):
            fixture_target_root(10, Path("/tmp/workspace-target"))


if __name__ == "__main__":
    unittest.main()
