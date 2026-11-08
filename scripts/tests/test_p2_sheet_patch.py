import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts/apply_gpui_component_sheet_close_accessibility_v2_patch.py"
MANIFEST = ROOT / "patches/approved/gpui-component-sheet-close-accessibility-v2.json"


class SheetPatchTest(unittest.TestCase):
    def test_exact_image_round_trip_and_tamper_rejection(self):
        meta = json.loads(MANIFEST.read_text())
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = root / meta["source"]
            source.parent.mkdir(parents=True)
            shutil.copyfile(ROOT / meta["source"], source)
            command = [sys.executable, str(SCRIPT), "--root", str(root), "--artifacts", str(ROOT)]
            for mode in ("--check", "--verify", "--reverse", "--check", "--apply", "--verify"):
                result = subprocess.run(command + [mode], capture_output=True, text=True)
                self.assertEqual(result.returncode, 0, result.stderr)
            source.write_bytes(source.read_bytes() + b"\n// tampered\n")
            result = subprocess.run(command + ["--check"], capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)


if __name__ == "__main__":
    unittest.main()
