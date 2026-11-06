"""Check reproducibility and source identity of the WebNoopener patch."""
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts/apply_gpui_web_noopener_patch.py"
FILES = (
    "vendor/gpui-pre-web-0.3.9/Cargo.toml",
    "vendor/gpui-pre-web-0.3.9/.cargo-checksum.json",
    "vendor/gpui-pre-web-0.3.9/src/platform.rs",
    "patches/approved/gpui-pre-web-noopener-v1.patch",
)


class WebNoopenerPatchTests(unittest.TestCase):
    def test_replay_and_reject_tampered_source(self):
        with tempfile.TemporaryDirectory(prefix="rustui-web-noopener-patch-") as directory:
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
