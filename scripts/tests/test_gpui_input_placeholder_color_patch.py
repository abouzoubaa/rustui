"""Verify the exact-image input placeholder patch is reproducible and guarded."""
import importlib.util
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts/apply_gpui_input_placeholder_color_patch.py"
spec = importlib.util.spec_from_file_location("placeholder_patch", SCRIPT)
patcher = importlib.util.module_from_spec(spec)
spec.loader.exec_module(patcher)


class PlaceholderPatchTest(unittest.TestCase):
    def test_reverse_apply_round_trip_and_reject_changed_source(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for name in [patcher.PATCH, *patcher.IMAGES]:
                destination = root / name
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(ROOT / name, destination)
            def run(action):
                return subprocess.run(
                    ["python3", str(SCRIPT), "--root", str(root), action],
                    capture_output=True, text=True,
                )
            for action in ("--check", "--reverse", "--apply", "--check"):
                result = run(action)
                self.assertEqual(result.returncode, 0, result.stderr)
            source = root / "vendor/gpui-component-0.7.3/src/input/input.rs"
            source.write_text(source.read_text() + "\n// unexpected change\n")
            self.assertNotEqual(run("--check").returncode, 0)


if __name__ == "__main__":
    unittest.main()
