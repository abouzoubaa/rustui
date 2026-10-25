from __future__ import annotations

import hashlib
import runpy
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

REPO = Path(__file__).resolve().parents[2]
HELPER = REPO / "scripts/apply_gpui_tree_accessibility_series.py"
FILES = (
    "Cargo.toml",
    "Cargo.lock",
    "vendor/gpui-pre-0.3.9/Cargo.toml",
    "vendor/gpui-pre-0.3.9/.cargo-checksum.json",
    "vendor/gpui-pre-0.3.9/LICENSE-APACHE",
    "vendor/gpui-pre-0.3.9/src/elements/div.rs",
    "vendor/gpui-pre-0.3.9/src/platform/test/window.rs",
    "vendor/gpui-base-0.7.3/Cargo.toml",
    "vendor/gpui-base-0.7.3/.cargo-checksum.json",
    "vendor/gpui-base-0.7.3/LICENSE-APACHE",
    "vendor/gpui-base-0.7.3/src/tree.rs",
    "vendor/gpui-component-0.7.3/Cargo.toml",
    "vendor/gpui-component-0.7.3/.cargo-checksum.json",
    "vendor/gpui-component-0.7.3/LICENSE-APACHE",
    "vendor/gpui-component-0.7.3/src/tree.rs",
    "patches/approved/gpui-tree-aria-disabled-v1.patch",
    "patches/approved/gpui-tree-aria-level-v1.patch",
    "patches/approved/gpui-component-tree-accessible-name-v1.patch",
    "patches/approved/gpui-pre-test-a11y-activation-v1.patch",
    "patches/approved/gpui-tree-accessibility-series-v1.json",
    "patches/proposed/gpui-tree-aria-level-v1.patch",
    "patches/proposed/gpui-component-tree-accessible-name-v1.patch",
    "patches/proposed/gpui-pre-test-a11y-activation-v1.patch",
)
PRE = {
    "vendor/gpui-base-0.7.3/src/tree.rs": "9ef2efa0a44af810eef3088343b0838179d5e952c6ca8734e5306d5c74736924",
    "vendor/gpui-component-0.7.3/src/tree.rs": "121c7b715b51d520e531682a14f9ba2791cd3a39554d68ab7cab289bfe7814f9",
    "vendor/gpui-pre-0.3.9/src/platform/test/window.rs": "3b359c9e7943ccb065096c718756b32400dac3394bfd9ab6f361dda8163e97bd",
}
POST = {
    "vendor/gpui-base-0.7.3/src/tree.rs": "0a1321f50154fb145b325517f9be93f262e959d21a213d61ecfdfadec38e4747",
    "vendor/gpui-component-0.7.3/src/tree.rs": "12e2e2eca25f85878c4ab681e8fae4e58342c6ac9b3cc60a831e4c8894d53df4",
    "vendor/gpui-pre-0.3.9/src/platform/test/window.rs": "8b267a9dfb6aadbfb00430ce491a85c1446789726ad84e54366247afa953d4c4",
}


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


class TreeAccessibilitySeriesTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="tree-accessibility-series-test-")
        self.root = Path(self.temporary.name)
        series = runpy.run_path(str(HELPER))
        support = set(FILES) | {str(path) for name in ("DISABLED_FILES", "PROVENANCE", "IDENTITY", "PATCH_HASHES") for path in series[name]}
        for relative in support:
            target = self.root / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(REPO / relative, target)
        helper = self.root / "scripts/apply_gpui_tree_accessibility_series.py"
        helper.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(HELPER, helper)
        series = runpy.run_path(str(helper))
        current = {path: (self.root / path).read_bytes() for path in series["FILES"]}
        hashes = {path: series["digest"](data) for path, data in current.items()}
        if hashes == series["POST"]:
            for path, data in series["transform"](self.root, current, reverse=True).items():
                (self.root / path).write_bytes(data)
        elif hashes != series["PRE"]:
            raise AssertionError("fixture requires exact PRE or POST images")

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def run_helper(self, mode: str, success: bool) -> subprocess.CompletedProcess[str]:
        result = subprocess.run(
            [
                "python3",
                str(self.root / "scripts/apply_gpui_tree_accessibility_series.py"),
                mode,
                "--root",
                str(self.root),
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode == 0, success, result.stdout + result.stderr)
        return result

    def test_check_round_trips_exact_series_without_writing(self) -> None:
        before = {relative: sha(self.root / relative) for relative in PRE}
        result = self.run_helper("--check", True)
        self.assertIn("checkout unchanged", result.stdout)
        self.assertEqual(before, {relative: sha(self.root / relative) for relative in PRE})

    def test_apply_and_reverse_composes_all_three_proposals(self) -> None:
        self.run_helper("--apply", True)
        self.assertEqual(POST, {relative: sha(self.root / relative) for relative in POST})
        self.run_helper("--reverse", True)
        self.assertEqual(PRE, {relative: sha(self.root / relative) for relative in PRE})

    def test_rejects_unrecognized_source_without_mutation(self) -> None:
        target = self.root / "vendor/gpui-base-0.7.3/src/tree.rs"
        target.write_bytes(target.read_bytes() + b"\n// changed\n")
        before = sha(target)
        self.run_helper("--check", False)
        self.assertEqual(before, sha(target))


if __name__ == "__main__":
    unittest.main()
