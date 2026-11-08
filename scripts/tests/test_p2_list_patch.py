import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts/apply_gpui_selectable_list_semantics_v2_patch.py"
MANIFEST = ROOT / "patches/approved/gpui-selectable-list-semantics-v2.json"

class ListSemanticsPatchTests(unittest.TestCase):
    def test_round_trip_tamper_and_stacked_popup_check(self):
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
            source.write_bytes(source.read_bytes() + b"tamper")
            self.assertNotEqual(subprocess.run(command + ["--check"], capture_output=True).returncode, 0)
        source = ROOT / meta["source"]
        before = source.read_bytes()
        result = subprocess.run([sys.executable, str(ROOT / "scripts/apply_gpui_select_popup_rtl_patch.py"), "--verify"], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(source.read_bytes(), before)

if __name__ == "__main__":
    unittest.main()
