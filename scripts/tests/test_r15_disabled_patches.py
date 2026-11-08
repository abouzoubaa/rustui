import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
CASES = (
    ("apply_r15_test_tree_disabled_v1_patch.py", "r15-test-tree-disabled-v1.json"),
    ("apply_r15_button_disabled_focus_v1_patch.py", "r15-button-disabled-focus-v1.json"),
    ("apply_r15_breadcrumb_disabled_v1_patch.py", "r15-breadcrumb-disabled-v1.json"),
    ("apply_r15_sidebar_disabled_v1_patch.py", "r15-sidebar-disabled-v1.json"),
    ("apply_r15_tree_disabled_focus_v1_patch.py", "r15-tree-disabled-focus-v1.json"),
)


class DisabledControlPatchTests(unittest.TestCase):
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

    def test_tree_stack_verification_preserves_source_and_rejects_drift(self):
        meta = json.loads((ROOT / "patches/approved/r15-tree-disabled-focus-v1.json").read_text())
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = root / meta["source"]
            source.parent.mkdir(parents=True)
            shutil.copyfile(ROOT / meta["source"], source)
            before = source.read_bytes()
            command = [sys.executable, str(ROOT / "scripts/verify_r15_tree_patch_stack.py"), "--root", str(root), "--artifacts", str(ROOT)]
            result = subprocess.run(command, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(source.read_bytes(), before)
            source.write_bytes(before + b"// drift")
            result = subprocess.run(command, capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("source differs", result.stderr)
            self.assertEqual(source.read_bytes(), before + b"// drift")

if __name__ == "__main__":
    unittest.main()
