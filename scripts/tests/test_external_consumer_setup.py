import subprocess
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "examples/external-consumer/create-local.py"


class ExternalConsumerSetupTests(unittest.TestCase):
    def test_new_workspace_uses_checkout_and_never_overwrites_destination(self):
        with tempfile.TemporaryDirectory() as directory:
            destination = Path(directory) / "consumer with spaces"
            command = [sys.executable, str(SCRIPT), str(destination)]
            subprocess.run(command, check=True, capture_output=True, text=True)
            manifest = tomllib.loads((destination / "Cargo.toml").read_text())
            self.assertEqual(
                Path(manifest["dependencies"]["rustui"]["path"]), ROOT / "crates/rustui"
            )
            self.assertNotIn("patch", manifest)
            self.assertEqual(set(manifest["dependencies"]), {"rustui"})
            config = tomllib.loads((destination / ".cargo/config.toml").read_text())
            self.assertEqual(
                Path(config["source"]["vendored-sources"]["directory"]), ROOT / "vendor"
            )
            self.assertEqual(
                (destination / "macos27.json").read_bytes(),
                (ROOT / "themes/macos27.json").read_bytes(),
            )
            self.assertEqual(
                (destination / "Cargo.lock").read_bytes(),
                (SCRIPT.parent / "Cargo.lock").read_bytes(),
            )
            self.assertIn("../macos27.json", (destination / "src/main.rs").read_text())

            sentinel = destination / "sentinel"
            sentinel.write_text("keep")
            result = subprocess.run(command, capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(sentinel.read_text(), "keep")


if __name__ == "__main__":
    unittest.main()
