import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts/apply_gpui_tree_disabled_keyboard_v2_patch.py"
MANIFEST = ROOT / "patches/approved/gpui-tree-disabled-keyboard-v2.json"


class TreeKeyboardPatchTests(unittest.TestCase):
    def test_exact_image_round_trip_and_tamper_rejection(self):
        meta = json.loads(MANIFEST.read_text())
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = root / meta["source"]
            source.parent.mkdir(parents=True)
            shutil.copyfile(ROOT / meta["source"], source)
            subprocess.run([sys.executable, str(ROOT / "scripts/apply_r15_tree_disabled_focus_v1_patch.py"), "--root", str(root), "--artifacts", str(ROOT), "--reverse"], check=True, capture_output=True)
            command = [sys.executable, str(SCRIPT), "--root", str(root), "--artifacts", str(ROOT)]
            for mode in ("--check", "--verify", "--reverse", "--check", "--apply", "--verify"):
                result = subprocess.run(command + [mode], text=True, capture_output=True)
                self.assertEqual(result.returncode, 0, result.stderr)
            source.write_bytes(source.read_bytes() + b"\n// tampered\n")
            self.assertNotEqual(subprocess.run(command + ["--check"], capture_output=True).returncode, 0)

    def test_existing_tree_series_checks_accept_stacked_postimage_without_mutation(self):
        source = ROOT / "vendor/gpui-base-0.7.3/src/tree.rs"
        before = source.read_bytes()
        for script, mode in (("apply_gpui_tree_accessibility_series.py", "--verify"), ("apply_gpui_tree_aria_disabled_patch.py", "--check")):
            result = subprocess.run([sys.executable, str(ROOT / "scripts" / script), mode], text=True, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(source.read_bytes(), before)


if __name__ == "__main__":
    unittest.main()
