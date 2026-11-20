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


STACK_PATH = REPO_ROOT / "patches/approved/fast/core-stack.json"
MENU_STACK_PATH = REPO_ROOT / "patches/approved/fast/menu-select-stack.json"
TREE_STACK_PATH = REPO_ROOT / "patches/approved/fast/tree-list-grid-stack.json"
SLIDER_STACK_PATH = REPO_ROOT / "patches/approved/fast/slider-resize-stack.json"
CALENDAR_STACK_PATH = REPO_ROOT / "patches/approved/fast/calendar-date-stack.json"
INPUT_TEXT_STACK_PATH = REPO_ROOT / "patches/approved/fast/input-text-stack.json"
TOOLBAR_STACK_PATH = REPO_ROOT / "patches/approved/fast/toolbar-rtl-stack.json"
SYNTAX_DIFF_STACK_PATH = REPO_ROOT / "patches/approved/fast/syntax-diff-stack.json"
NATIVE_POPUP_STACK_PATH = REPO_ROOT / "patches/approved/fast/native-popup-stack.json"


def approved_fixture(fixture):
    """Peel active damage candidates before replaying historical base patches."""
    from check_fast_patch_stack import STACK, active_stack, load_manifests
    entries, final = load_manifests(active_stack())
    approved_count = len(load_manifests(STACK)[0])
    for relative in final:
        target = fixture / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes((REPO_ROOT / relative).read_bytes())
    from refresh_owned_fast_sources import SOURCES, manifest_wiring, source_images, wired_manifests
    copies = json.loads(SOURCES.read_text())["copies"]
    changed_manifests = any(source_images(copy, active_stack())["Cargo.toml"] != copy["files"]["Cargo.toml"]
                            for copy in copies)
    if changed_manifests:
        with wired_manifests(REPO_ROOT, active_stack()) as paths:
            for path in reversed(paths):
                apply_patch(path, "--reverse", fixture)
    for path, _, _ in reversed(entries[approved_count:]):
        apply_patch(path, "--reverse", fixture)
    if changed_manifests:
        for copy in copies:
            owned = fixture / copy["owned"] / "Cargo.toml"
            if not owned.exists():
                continue
            vendor = fixture / copy["vendor"] / "Cargo.toml"
            source = vendor if vendor.exists() else REPO_ROOT / copy["vendor"] / "Cargo.toml"
            data = source.read_bytes()
            if hashlib.sha256(data).hexdigest() != copy["files"]["Cargo.toml"]:
                raise ValueError("historical fixture did not restore the approved vendor manifest")
            owned.write_bytes(data)
        for path in manifest_wiring():
            apply_patch(path, "--apply", fixture)


class FastCoreStackTests(unittest.TestCase):
    def assert_family_candidates_are_exact_kit09_copies(self, stack_path: Path, family: str):
        stack = json.loads(stack_path.read_text())
        self.assertEqual(stack["family"], family)
        approved = REPO_ROOT / "patches/approved/kit09"
        candidate = REPO_ROOT / "patches/approved/fast"
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

    def test_slider_resize_candidates_are_exact_kit09_copies(self):
        self.assert_family_candidates_are_exact_kit09_copies(SLIDER_STACK_PATH, "slider-resize")

    def test_calendar_date_candidates_are_exact_kit09_copies(self):
        self.assert_family_candidates_are_exact_kit09_copies(CALENDAR_STACK_PATH, "calendar-date")

    def test_input_text_candidates_are_exact_kit09_copies(self):
        self.assert_family_candidates_are_exact_kit09_copies(INPUT_TEXT_STACK_PATH, "input-text")

    def test_toolbar_rtl_candidates_are_exact_kit09_copies(self):
        self.assert_family_candidates_are_exact_kit09_copies(TOOLBAR_STACK_PATH, "toolbar-rtl")

    def test_syntax_diff_candidates_are_exact_kit09_copies(self):
        self.assert_family_candidates_are_exact_kit09_copies(SYNTAX_DIFF_STACK_PATH, "syntax-diff")

    def test_native_popup_stack_references_hashed_core_candidates(self):
        stack = json.loads(NATIVE_POPUP_STACK_PATH.read_text())
        core = json.loads(STACK_PATH.read_text())
        self.assertEqual(stack["family"], "native-popup")
        self.assertEqual(
            stack["patches"],
            [
                "gpui-macos-anchored-popup-v1-fast.json",
                "gpui-macos-popup-child-teardown-v1-fast.json",
                "gpui-macos-popup-defer-frame-v1-fast.json",
            ],
        )
        for manifest_name in stack["patches"]:
            self.assertIn(manifest_name, core["patches"])
            manifest = json.loads((REPO_ROOT / "patches/approved/fast" / manifest_name).read_text())
            patch = REPO_ROOT / "patches/approved/fast" / manifest["patch"]
            self.assertEqual(hashlib.sha256(patch.read_bytes()).hexdigest(), manifest["patch_sha256"])

    def test_bounded_offset_experiment_is_inactive_but_replays_exactly(self):
        core = json.loads(STACK_PATH.read_text())
        name = "gpui-fast-bounded-wheel-offset-v1-fast.json"
        self.assertNotIn(name, core["patches"])
        manifest = REPO_ROOT / "patches/retired/fast-experiments" / name
        data = json.loads(manifest.read_text())
        with tempfile.TemporaryDirectory() as temporary:
            fixture = Path(temporary)
            approved_fixture(fixture)
            originals = {relative: (fixture / relative).read_bytes() for relative in data["files"]}
            apply_patch(manifest,"--apply",fixture)
            apply_patch(manifest,"--check",fixture)
            apply_patch(manifest,"--reverse",fixture)
            for relative in data["files"]:
                self.assertEqual((fixture/relative).read_bytes(), originals[relative])

    def test_promoted_inventory_contains_only_the_active_stack(self):
        from check_fast_patch_stack import PATCH_DIR, load_manifests
        stack = json.loads((PATCH_DIR / "fast-stack.json").read_text())
        names = stack["patches"] + stack["alternative_postimage_proofs"]
        self.assertEqual(len(names), 66)
        self.assertEqual(len(set(names)), 66)
        self.assertEqual(
            {Path(json.loads((PATCH_DIR / name).read_text())["patch"]).name for name in names},
            {path.name for path in PATCH_DIR.glob("*.patch")},
        )
        self.assertNotIn("gpui-fast-bounded-wheel-offset-v1-fast.json", names)
        entries, final = load_manifests()
        self.assertEqual(len(entries), 65)
        self.assertEqual(len(final), 59)

    def test_full_approved_stack_reverses_and_reapplies_exactly(self):
        from check_fast_patch_stack import verify
        verify()

    def test_core_stack_reverses_and_reapplies_exactly(self):
        stack = json.loads(STACK_PATH.read_text())
        manifests = [REPO_ROOT / "patches/approved/fast" / name for name in stack["patches"]]
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
            approved_fixture(fixture)
            originals = {relative: (fixture / relative).read_bytes() for relative in union}
            from refresh_owned_fast_sources import manifest_wiring
            for path in reversed(manifest_wiring()):
                apply_patch(path, "--reverse", fixture)
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
            for path in manifest_wiring():
                apply_patch(path, "--apply", fixture)
            from check_fast_patch_stack import STACK, check
            check(fixture, STACK)
            for relative in union:
                self.assertEqual((fixture / relative).read_bytes(), originals[relative])


if __name__ == "__main__":
    unittest.main()
