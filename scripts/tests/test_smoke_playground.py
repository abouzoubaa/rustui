"""Argument and temporary-path checks for the Playground smoke script."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "smoke-playground.sh"


class PlaygroundSmokeArguments(unittest.TestCase):
    def test_dry_run_uses_and_removes_an_isolated_temporary_bundle(self):
        with tempfile.TemporaryDirectory(prefix="smoke test ") as directory:
            env = {**os.environ, "TMPDIR": directory, "RUSTUI_PLAYGROUND_BUNDLE_PATH": "/not/used.app"}
            result = subprocess.run([str(SCRIPT), "--dry-run"], env=env, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            bundle = Path(result.stdout.strip().removeprefix("temporary bundle: "))
            self.assertEqual(bundle.name, "RustUI Playground.app")
            self.assertEqual(bundle.parent.parent, Path(directory).resolve())
            self.assertFalse(bundle.parent.exists())
            self.assertEqual(list(Path(directory).iterdir()), [])

    def test_unknown_and_multiple_arguments_fail_before_launch(self):
        for args in (["--replace"], ["--dry-run", "extra"]):
            with self.subTest(args=args):
                result = subprocess.run([str(SCRIPT), *args], capture_output=True, text=True)
                self.assertEqual(result.returncode, 2)
                self.assertIn("error:", result.stderr)


if __name__ == "__main__":
    unittest.main()
