import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts/apply_gpui_base_slider_first_move.py"
SOURCE = Path("vendor/gpui-base-0.7.3/src/slider.rs")
PATCH = ROOT / "patches/approved/gpui-base-slider-first-move-v1.patch"


class SliderFirstMovePatchTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="rustui-slider-patch-test-")
        self.root = Path(self.temporary.name)
        source = self.root / SOURCE
        source.parent.mkdir(parents=True)
        shutil.copyfile(ROOT / SOURCE, source)
        shutil.copyfile(ROOT / "vendor/gpui-base-0.7.3/Cargo.toml", source.parent.parent / "Cargo.toml")
        (self.root / "Cargo.toml").write_text(
            '[patch.crates-io]\ngpui-base = { path = "vendor/gpui-base-0.7.3" }\n'
        )
        subprocess.run(["git", "apply", "--reverse", str(PATCH)], cwd=self.root, check=True)

    def tearDown(self):
        self.temporary.cleanup()

    def run_patch(self, mode):
        return subprocess.run(
            [sys.executable, str(SCRIPT), "--root", str(self.root), "--artifacts", str(ROOT), mode],
            capture_output=True,
            text=True,
        )

    def test_apply_verify_and_reverse_exact_images(self):
        source = self.root / SOURCE
        before = source.read_bytes()
        for mode in ("--check", "--apply", "--verify", "--reverse"):
            result = self.run_patch(mode)
            self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(source.read_bytes(), before)

    def test_rejects_unknown_source_and_wrong_package_selection(self):
        source = self.root / SOURCE
        source.write_bytes(source.read_bytes() + b"\n// unexpected change\n")
        self.assertNotEqual(self.run_patch("--apply").returncode, 0)
        (self.root / "Cargo.toml").write_text("[patch.crates-io]\n")
        self.assertNotEqual(self.run_patch("--check").returncode, 0)

    def test_composed_stack_detects_tampered_earlier_patch_without_mutation(self):
        self.assertEqual(self.run_patch("--apply").returncode, 0)
        source = self.root / SOURCE
        before = source.read_bytes()
        artifacts = self.root / "artifacts"
        patches = artifacts / "patches/approved"
        patches.mkdir(parents=True)
        for name in ("gpui-base-slider-first-move-v1.patch", "gpui-base-slider-zero-track-v1.patch"):
            shutil.copyfile(ROOT / "patches/approved" / name, patches / name)
        command = [sys.executable, str(ROOT / "scripts/verify_gpui_base_slider_patch_stack.py"),
                   "--root", str(self.root), "--artifacts", str(artifacts)]
        self.assertEqual(subprocess.run(command, capture_output=True).returncode, 0)
        with (patches / "gpui-base-slider-zero-track-v1.patch").open("ab") as patch:
            patch.write(b"\n# unexpected change\n")
        self.assertNotEqual(subprocess.run(command, capture_output=True).returncode, 0)
        self.assertEqual(source.read_bytes(), before)


if __name__ == "__main__":
    unittest.main()
