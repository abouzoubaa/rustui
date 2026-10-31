import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "manifest_playground_web.py"
SPEC = importlib.util.spec_from_file_location("web_manifest", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class WebManifestTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name) / "repo"
        self.root.mkdir()
        for relative in MODULE.SOURCE_ROOTS:
            (self.root / relative).mkdir(parents=True)
        for relative in MODULE.SOURCE_INPUTS:
            path = self.root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("fixture")
        self.output = self.root.parent / "manifest.json"

    def run_manifest(self, *arguments):
        return subprocess.run(
            [sys.executable, str(SCRIPT), "--root", str(self.root),
             "--output", str(self.output), *map(str, arguments)],
            capture_output=True, text=True, check=False,
        )

    def test_mutation_rejected_before_existing_manifest_is_replaced(self):
        self.assertEqual(self.run_manifest().returncode, 0)
        original = self.output.read_bytes()
        (self.root / "crates/changed.rs").write_text("changed")
        result = self.run_manifest("--source-baseline", self.output)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("sources changed", result.stderr)
        self.assertEqual(self.output.read_bytes(), original)

    def test_missing_input_or_source_directory_is_rejected(self):
        (self.root / "Cargo.lock").unlink()
        self.assertNotEqual(self.run_manifest().returncode, 0)
        self.assertFalse(self.output.exists())
        (self.root / "Cargo.lock").touch()
        (self.root / "themes").rmdir()
        self.assertNotEqual(self.run_manifest().returncode, 0)
        self.assertFalse(self.output.exists())

    def test_symlinked_input_and_directory_are_rejected(self):
        external = self.root.parent / "external"
        external.mkdir()
        (self.root / "crates/escape").symlink_to(external, target_is_directory=True)
        self.assertNotEqual(self.run_manifest().returncode, 0)
        (self.root / "crates/escape").unlink()
        (external / "data").write_text("outside")
        (self.root / "crates/escape.rs").symlink_to(external / "data")
        self.assertNotEqual(self.run_manifest().returncode, 0)

    def test_artifact_hashes_and_cache_exclusion(self):
        bundle = self.root.parent / "bundle"
        bundle.mkdir()
        (bundle / "index.html").write_text("html")
        self.assertEqual(self.run_manifest().returncode, 0)
        source_digest = json.loads(self.output.read_text())["source"]["sha256"]
        (self.root / "crates/graft").mkdir()
        (self.root / "crates/graft/noise").write_text("cache")
        self.assertEqual(self.run_manifest("--bundle-dir", bundle).returncode, 0)
        result = json.loads(self.output.read_text())
        self.assertEqual(result["source"]["sha256"], source_digest)
        artifact, = result["bundle"]["files"]
        self.assertEqual(artifact["path"], "index.html")
        self.assertEqual(artifact["sha256"], hashlib.sha256(b"html").hexdigest())


if __name__ == "__main__":
    unittest.main()
