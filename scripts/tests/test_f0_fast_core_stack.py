import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "scripts"))

from apply_f0_fast_candidate_patch import run as apply_patch  # noqa: E402
from check_f0_fast_core_stack import run as apply_stack  # noqa: E402


STACK_PATH = REPO_ROOT / "patches/candidate-kit-fast/core-stack.json"
MENU_STACK_PATH = REPO_ROOT / "patches/candidate-kit-fast/menu-select-stack.json"
TREE_STACK_PATH = REPO_ROOT / "patches/candidate-kit-fast/tree-list-grid-stack.json"


class FastCoreStackTests(unittest.TestCase):
    def assert_family_candidates_are_exact_kit09_copies(self, stack_path: Path, family: str):
        stack = json.loads(stack_path.read_text())
        self.assertEqual(stack["family"], family)
        approved = REPO_ROOT / "patches/approved/kit09"
        candidate = REPO_ROOT / "patches/candidate-kit-fast"
        for manifest_name in stack["patches"]:
            manifest_path = candidate / manifest_name
            manifest = json.loads(manifest_path.read_text())
            source_manifest_path = approved / manifest_name
            source_manifest = json.loads(source_manifest_path.read_text())
            source_patch = approved / source_manifest["candidate"]
            candidate_patch = candidate / manifest["patch"]
            self.assertEqual(candidate_patch.read_bytes(), source_patch.read_bytes())
            self.assertEqual(manifest["patch_sha256"], source_manifest["patch_sha256"])
            self.assertEqual(manifest["files"], source_manifest["files"])

    def test_menu_select_candidates_are_exact_kit09_copies(self):
        self.assert_family_candidates_are_exact_kit09_copies(MENU_STACK_PATH, "menu-select")

    def test_tree_list_grid_candidates_are_exact_kit09_copies(self):
        self.assert_family_candidates_are_exact_kit09_copies(TREE_STACK_PATH, "tree-list-grid")

    def test_core_stack_reverses_and_reapplies_exactly(self):
        stack = json.loads(STACK_PATH.read_text())
        manifests = [REPO_ROOT / "patches/candidate-kit-fast" / name for name in stack["patches"]]
        union = {}
        for manifest_path in manifests:
            manifest = json.loads(manifest_path.read_text())
            patch = manifest_path.parent / manifest["patch"]
            self.assertEqual(hashlib.sha256(patch.read_bytes()).hexdigest(), manifest["patch_sha256"])
            for relative, hashes in manifest["files"].items():
                previous = union.get(relative)
                if previous is not None:
                    self.assertEqual(previous["post_sha256"], hashes["pre_sha256"])
                union[relative] = hashes
        with tempfile.TemporaryDirectory() as temporary:
            fixture = Path(temporary)
            for relative in union:
                target = fixture / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes((REPO_ROOT / relative).read_bytes())
            for manifest_path in reversed(manifests):
                manifest = json.loads(manifest_path.read_text())
                apply_patch(manifest_path, "--reverse", fixture)
                for relative, hashes in manifest["files"].items():
                    self.assertEqual(hashlib.sha256((fixture / relative).read_bytes()).hexdigest(), hashes["pre_sha256"])
            for manifest_path in manifests:
                manifest = json.loads(manifest_path.read_text())
                apply_patch(manifest_path, "--apply", fixture)
                for relative, hashes in manifest["files"].items():
                    self.assertEqual(hashlib.sha256((fixture / relative).read_bytes()).hexdigest(), hashes["post_sha256"])
            apply_stack("--check", fixture)
            for relative, hashes in union.items():
                self.assertEqual(hashlib.sha256((fixture / relative).read_bytes()).hexdigest(), hashes["post_sha256"])


if __name__ == "__main__":
    unittest.main()
