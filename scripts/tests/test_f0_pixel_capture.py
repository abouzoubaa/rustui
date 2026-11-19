"""Native pixel classifier tests; no screen capture or GUI session needed."""
from pathlib import Path
import platform
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


@unittest.skipUnless(platform.system() == "Darwin", "native macOS capture harness")
class PixelCaptureTests(unittest.TestCase):
    def test_blank_painted_and_stale_row_classification(self):
        with tempfile.TemporaryDirectory() as temporary:
            binary = Path(temporary) / "pixel-capture"
            build = subprocess.run(["swiftc", "-O", "-parse-as-library", str(ROOT / "scripts/f0_capture_frames.swift"), "-o", str(binary)], capture_output=True, text=True)
            self.assertEqual(build.returncode, 0, build.stdout + build.stderr)
            result = subprocess.run([str(binary), "self-test"], capture_output=True, text=True)
            sys.stderr.write(result.stdout + result.stderr)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn("blank, painted, stale = PASS", result.stdout)
            self.assertIn("check=editor fixture ID type expected=Int actual=Int", result.stderr)
            self.assertIn("check=editor source position expected=true actual=true", result.stderr)
            self.assertEqual(result.stderr.count("check="), 12)
