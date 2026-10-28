"""Check reproducibility and source identity of the DatePicker patch."""
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts/apply_gpui_component_date_picker_patch.py"
FILES = (
    "vendor/gpui-component-0.7.3/Cargo.toml",
    "vendor/gpui-component-0.7.3/.cargo-checksum.json",
    "vendor/gpui-component-0.7.3/src/time/date_picker.rs",
    "patches/approved/gpui-component-date-picker-localization-v2.patch",
)


class DatePickerPatchTests(unittest.TestCase):
    def test_replay_and_reject_tampered_source(self):
        with tempfile.TemporaryDirectory(prefix="rustui-date-picker-patch-") as directory:
            root = Path(directory)
            for relative in FILES:
                target = root / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(ROOT / relative, target)

            def run(action):
                return subprocess.run(
                    [sys.executable, str(SCRIPT), "--root", str(root), action],
                    capture_output=True, text=True, check=False,
                )

            for action in ("--check", "--reverse", "--apply", "--check", "--reverse"):
                result = run(action)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            source = root / FILES[2]
            source.write_text(source.read_text(encoding="utf-8") + "\n", encoding="utf-8")
            result = run("--apply")
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("unexpected source SHA-256", result.stderr)


if __name__ == "__main__":
    unittest.main()
