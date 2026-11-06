import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "provision-ci-wasm-bindgen.sh"


class ProvisionCiWasmBindgenTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / "scripts").mkdir()
        shutil.copy2(SCRIPT, self.root / "scripts")
        self.bin = self.root / "fake-bin"
        self.bin.mkdir()
        self.fake("uname", 'case "$1" in -s) echo Darwin;; -m) echo arm64;; esac')
        self.fake("curl", 'while [ "$#" -gt 0 ]; do if [ "$1" = -o ]; then shift; printf corrupt > "$1"; exit 0; fi; shift; done; exit 1')
        self.tool = self.root / "target/headstart/tools/bin/wasm-bindgen"

    def fake(self, name, body):
        path = self.bin / name
        path.write_text("#!/bin/sh\n" + body + "\n")
        path.chmod(0o755)

    def run_provision(self):
        env = dict(os.environ, PATH=f"{self.bin}{os.pathsep}{os.environ['PATH']}")
        return subprocess.run([self.root / "scripts" / SCRIPT.name], env=env,
                              capture_output=True, text=True)

    def test_corrupt_archive_is_rejected_before_installation(self):
        result = self.run_provision()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("SHA-256 mismatch", result.stderr)
        self.assertFalse(self.tool.exists())

    def test_incompatible_existing_tool_is_preserved(self):
        self.tool.parent.mkdir(parents=True)
        self.tool.write_text("#!/bin/sh\necho wasm-bindgen 0.1.0\n")
        self.tool.chmod(0o755)
        original = self.tool.read_bytes()
        result = self.run_provision()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("preserving", result.stderr)
        self.assertEqual(self.tool.read_bytes(), original)

    def test_compatible_existing_tool_is_reused(self):
        self.tool.parent.mkdir(parents=True)
        self.tool.write_text("#!/bin/sh\necho wasm-bindgen 0.2.129\n")
        self.tool.chmod(0o755)
        original = self.tool.read_bytes()
        self.assertEqual(self.run_provision().returncode, 0)
        self.assertEqual(self.tool.read_bytes(), original)


if __name__ == "__main__":
    unittest.main()
