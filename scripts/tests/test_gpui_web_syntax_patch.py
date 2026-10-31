"""Exercise the exact Web syntax patch stack on a disposable package copy."""
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "scripts/apply_gpui_component_web_syntax_patch.py"
RUNTIME = ROOT / "scripts/apply_gpui_component_web_syntax_runtime_patch.py"
FILES = (
    "vendor/gpui-component-0.7.3/Cargo.toml",
    "vendor/gpui-component-0.7.3/.cargo-checksum.json",
    "vendor/gpui-component-0.7.3/src/highlighter/registry.rs",
    "vendor/gpui-component-0.7.3/src/highlighter/highlighter.rs",
    "vendor/gpui-component-0.7.3/src/diff/state.rs",
    "vendor/gpui-component-0.7.3/src/input/syntax_context.rs",
    "patches/approved/gpui-component-web-syntax-v1.patch",
    "patches/approved/gpui-component-web-syntax-runtime-v3.patch",
)


class WebSyntaxPatchTests(unittest.TestCase):
    def test_replay_and_reject_tampering(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for relative in FILES:
                target = root / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(ROOT / relative, target)
            def run(script, action):
                return subprocess.run([sys.executable, str(script), "--root", str(root), action],
                                      capture_output=True, text=True, check=False)
            for script, action in ((RUNTIME, "--check"), (RUNTIME, "--reverse"),
                                   (MANIFEST, "--reverse"), (MANIFEST, "--apply"),
                                   (RUNTIME, "--apply"), (MANIFEST, "--check"),
                                   (RUNTIME, "--check")):
                result = run(script, action)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            source = root / FILES[2]
            source.write_text(source.read_text() + "\n")
            self.assertNotEqual(run(RUNTIME, "--check").returncode, 0)


if __name__ == "__main__":
    unittest.main()
