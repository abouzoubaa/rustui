import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts/apply_gpui_primary_mousedown_click_v2_patch.py"
MANIFEST = ROOT / "patches/approved/gpui-primary-mousedown-click-v2.json"


class PrimaryMousePatchTests(unittest.TestCase):
    def test_exact_image_round_trip_and_tamper_rejection(self):
        meta = json.loads(MANIFEST.read_text())
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = root / meta["source"]
            source.parent.mkdir(parents=True)
            shutil.copyfile(ROOT / meta["source"], source)
            command = [sys.executable, str(SCRIPT), "--root", str(root), "--artifacts", str(ROOT)]
            for mode in ("--check", "--verify", "--reverse", "--check", "--apply", "--verify"):
                result = subprocess.run(command + [mode], text=True, capture_output=True)
                self.assertEqual(result.returncode, 0, result.stderr)
            source.write_bytes(source.read_bytes() + b"\n// tampered\n")
            self.assertNotEqual(subprocess.run(command + ["--check"], capture_output=True).returncode, 0)

    def test_earlier_a11y_layer_verifies_without_mutating_the_new_postimage(self):
        source = ROOT / "vendor/gpui-pre-0.3.9/src/elements/div.rs"
        before = source.read_bytes()
        result = subprocess.run([sys.executable, str(ROOT / "scripts/apply_gpui_validation_grid_a11y_patch.py"), "--verify"], text=True, capture_output=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(source.read_bytes(), before)


if __name__ == "__main__":
    unittest.main()
