"""Check reproducibility and source identity of the Cosmic bidi patch."""
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts/apply_cosmic_bidi_initial_span_patch.py"
FILES = (
    "vendor/cosmic-text-0.19.0/Cargo.toml",
    "vendor/cosmic-text-0.19.0/.cargo-checksum.json",
    "vendor/cosmic-text-0.19.0/src/shape.rs",
    "patches/approved/cosmic-text-initial-bidi-span-v1.patch",
)


class CosmicBidiPatchTests(unittest.TestCase):
    def test_replay_and_reject_tampered_source(self):
        with tempfile.TemporaryDirectory(prefix="rustui-cosmic-bidi-patch-") as directory:
            root = Path(directory)
            for relative in FILES:
                target = root / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(ROOT / relative, target)

            def run(action):
                return subprocess.run(
                    [sys.executable, str(SCRIPT), "--root", str(root), action],
                    capture_output=True, text=True, check=False,
                )

            for action in ("--check", "--reverse", "--apply", "--check", "--reverse"):
                result = run(action)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            source = root / FILES[2]
            source.write_text(source.read_text(encoding="utf-8") + "\n", encoding="utf-8")
            result = run("--apply")
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("unexpected source SHA-256", result.stderr)


if __name__ == "__main__":
    unittest.main()
