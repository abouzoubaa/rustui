import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
CASES = (
    ("apply_gpui_base_slider_set_value_v2_patch.py", "gpui-base-slider-set-value-v2.json"),
    ("apply_gpui_component_slider_accessibility_label_v2_patch.py", "gpui-component-slider-accessibility-label-v2.json"),
)


class SliderPatchTests(unittest.TestCase):
    def test_exact_roundtrip_and_source_patch_manifest_tamper_rejection(self):
        for script, manifest in CASES:
            with self.subTest(script=script), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                meta = json.loads((ROOT / "patches/approved" / manifest).read_text())
                source = root / meta["source"]
                source.parent.mkdir(parents=True)
                shutil.copyfile(ROOT / meta["source"], source)
                command = [sys.executable, str(ROOT / "scripts" / script), "--root", str(root), "--artifacts", str(ROOT)]
                for mode in ("--check", "--verify", "--reverse", "--check", "--apply", "--verify"):
                    result = subprocess.run(command + [mode], capture_output=True, text=True)
                    self.assertEqual(result.returncode, 0, result.stderr)
                before = source.read_bytes()
                source.write_bytes(before + b"\n// tamper\n")
                result = subprocess.run(command + ["--check"], capture_output=True, text=True)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("source differs", result.stderr)
                source.write_bytes(before)
                artifacts = root / "artifacts"
                patches = artifacts / "patches/approved"
                patches.mkdir(parents=True)
                manifest_path = patches / manifest
                shutil.copyfile(ROOT / "patches/approved" / manifest, manifest_path)
                patch_path = artifacts / meta["patch"]
                shutil.copyfile(ROOT / meta["patch"], patch_path)
                command[-1] = str(artifacts)
                patch_path.write_bytes(patch_path.read_bytes() + b"tamper")
                result = subprocess.run(command + ["--check"], capture_output=True, text=True)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("patch hash mismatch", result.stderr)
                shutil.copyfile(ROOT / meta["patch"], patch_path)
                manifest_path.write_bytes(manifest_path.read_bytes() + b" ")
                result = subprocess.run(command + ["--check"], capture_output=True, text=True)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("manifest hash mismatch", result.stderr)
                self.assertEqual(source.read_bytes(), before)


if __name__ == "__main__":
    unittest.main()
