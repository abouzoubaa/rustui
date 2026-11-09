import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))
CASES = ("apply_g10_parent_node_id_v1_patch.py", "apply_g10_labelled_by_tree_v1_patch.py")

def manifest_for(script):
    spec = importlib.util.spec_from_file_location("g10_patch", ROOT / "scripts" / script)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.MANIFEST

class GridAccessibilityPatchTests(unittest.TestCase):
    def test_exact_roundtrip_and_source_patch_manifest_tamper_rejection(self):
        for script in CASES:
            with self.subTest(script=script), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                manifest = manifest_for(script)
                meta = json.loads((ROOT / manifest).read_text())
                source = root / meta["source"]
                source.parent.mkdir(parents=True)
                shutil.copyfile(ROOT / meta["source"], source)
                checksum = root / "vendor/gpui-pre-0.3.9/.cargo-checksum.json"
                checksum.write_bytes(b"unchanged checksum metadata")
                command = [sys.executable, str(ROOT / "scripts" / script), "--root", str(root), "--artifacts", str(ROOT)]
                for mode in ("--check", "--verify", "--reverse", "--check", "--apply", "--verify"):
                    result = subprocess.run(command + [mode], capture_output=True, text=True)
                    self.assertEqual(result.returncode, 0, result.stderr)
                before = source.read_bytes()
                source.write_bytes(before + b"\n// drift\n")
                result = subprocess.run(command + ["--check"], capture_output=True, text=True)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("source differs", result.stderr)
                source.write_bytes(before)
                artifacts = root / "artifacts"
                manifest_path = artifacts / manifest
                manifest_path.parent.mkdir(parents=True)
                shutil.copyfile(ROOT / manifest, manifest_path)
                patch_path = artifacts / meta["patch"]
                shutil.copyfile(ROOT / meta["patch"], patch_path)
                command[-1] = str(artifacts)
                patch_path.write_bytes(patch_path.read_bytes() + b"drift")
                result = subprocess.run(command + ["--check"], capture_output=True, text=True)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("patch hash mismatch", result.stderr)
                shutil.copyfile(ROOT / meta["patch"], patch_path)
                manifest_path.write_bytes(manifest_path.read_bytes() + b" ")
                result = subprocess.run(command + ["--check"], capture_output=True, text=True)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("manifest hash mismatch", result.stderr)
                self.assertEqual(source.read_bytes(), before)
                self.assertEqual(checksum.read_bytes(), b"unchanged checksum metadata")

    def test_debug_stack_verification_preserves_source_and_rejects_drift(self):
        manifest = manifest_for("apply_g10_labelled_by_tree_v1_patch.py")
        meta = json.loads((ROOT / manifest).read_text())
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = root / meta["source"]
            source.parent.mkdir(parents=True)
            shutil.copyfile(ROOT / meta["source"], source)
            before = source.read_bytes()
            command = [sys.executable, str(ROOT / "scripts/verify_g10_grid_debug_patch_stack.py"), "--root", str(root), "--artifacts", str(ROOT)]
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
