import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
CASES = (
    ("apply_gpui_base_focus_trap_accessibility_v1_patch.py", "gpui-base-focus-trap-accessibility-v1.json"),
    ("apply_gpui_component_popup_menu_disabled_confirm_v1_patch.py", "gpui-component-popup-menu-disabled-confirm-v1.json"),
)


class ApprovedPatchTests(unittest.TestCase):
    def test_exact_source_round_trip_and_tamper_rejection(self):
        for script, manifest in CASES:
            with self.subTest(script=script), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                meta = json.loads((ROOT / "patches/approved" / manifest).read_text())
                source = root / meta["source"]
                source.parent.mkdir(parents=True)
                shutil.copyfile(ROOT / meta["source"], source)
                if meta["source"] == "vendor/gpui-component-0.7.3/src/menu/popup_menu.rs":
                    subprocess.run([sys.executable, str(ROOT / "scripts/apply_r15b_popup_menu_recovery_v1_patch.py"), "--root", str(root), "--artifacts", str(ROOT), "--reverse"], check=True, capture_output=True)
                command = [sys.executable, str(ROOT / "scripts" / script), "--root", str(root), "--artifacts", str(ROOT)]
                for mode in ("--check", "--verify", "--reverse", "--check", "--apply", "--verify"):
                    result = subprocess.run(command + [mode], text=True, capture_output=True)
                    self.assertEqual(result.returncode, 0, result.stderr)
                source.write_bytes(source.read_bytes() + b"\n// tampered\n")
                result = subprocess.run(command + ["--check"], text=True, capture_output=True)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("source differs", result.stderr)


if __name__ == "__main__":
    unittest.main()
