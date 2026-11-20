import hashlib
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts/apply_f0_component_manifest_normalize.py"
SOURCE = "vendor/gpui-component-0.9.1/Cargo.toml"


class ComponentManifestNormalizeTests(unittest.TestCase):
    def test_patch_checks_applies_and_reverses_exact_vendor_manifest(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            target = root / SOURCE
            target.parent.mkdir(parents=True)
            target.write_bytes((ROOT / SOURCE).read_bytes())
            run = lambda mode: subprocess.run(
                ["python3", str(SCRIPT), mode, "--root", str(root)],
                check=True, capture_output=True, text=True
            )
            run("--check")
            run("--reverse")
            gpui_dependency = target.read_text().split("[dependencies.gpui]\n", 1)[1].split("\n[", 1)[0]
            self.assertIn("optional = true", gpui_dependency)
            run("--apply")
            gpui_dependency = target.read_text().split("[dependencies.gpui]\n", 1)[1].split("\n[", 1)[0]
            self.assertNotIn("optional = true", gpui_dependency)
            run("--check")


if __name__ == "__main__":
    unittest.main()
