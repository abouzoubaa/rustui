import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts/apply_gpui_component_tracked_dialog_focus_v2_patch.py"
MANIFEST = ROOT / "patches/approved/gpui-component-tracked-dialog-focus-rebase-v2.json"

class TrackedDialogPatchTests(unittest.TestCase):
    def test_roundtrip_tamper_and_stacked_modal_verifier(self):
        meta = json.loads(MANIFEST.read_text())
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for name in meta["files"]:
                target = root / name
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(ROOT / name, target)
            command = [sys.executable, str(SCRIPT), "--root", str(root), "--artifacts", str(ROOT)]
            for mode in ("--check", "--verify", "--reverse", "--check", "--apply", "--verify"):
                result = subprocess.run(command + [mode], capture_output=True, text=True)
                self.assertEqual(result.returncode, 0, result.stderr)
            target = root / next(iter(meta["files"]))
            target.write_bytes(target.read_bytes() + b"tamper")
            self.assertNotEqual(subprocess.run(command + ["--check"], capture_output=True).returncode, 0)
        before = {name: (ROOT / name).read_bytes() for name in meta["files"]}
        result = subprocess.run([sys.executable, str(ROOT / "scripts/apply_gpui_modal_focus_recovery_patch.py"), "--verify"], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(before, {name: (ROOT / name).read_bytes() for name in meta["files"]})

if __name__ == "__main__":
    unittest.main()
