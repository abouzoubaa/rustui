import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / 'scripts/apply_gpui_toolbar_rtl.py'
META = json.loads((ROOT / 'patches/approved/gpui-toolbar-rtl-v1.json').read_text())


class ToolbarPatchTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        paths = ['Cargo.toml', *META['pre'], *META['preserved'],
                 'vendor/gpui-base-0.7.3/Cargo.toml', 'vendor/gpui-component-0.7.3/Cargo.toml']
        for path in paths:
            dest = self.root / path
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / path, dest)

    def run_patch(self, mode):
        return subprocess.run([sys.executable, str(SCRIPT), '--root', str(self.root),
                               '--artifacts', str(ROOT), mode], capture_output=True, text=True)

    def test_exact_apply_verify_and_reverse(self):
        if self.run_patch('--verify').returncode == 0:
            self.assertEqual(self.run_patch('--reverse').returncode, 0)
        before = {path: (self.root / path).read_bytes() for path in META['pre']}
        for mode in ['--check', '--apply', '--verify', '--reverse']:
            result = self.run_patch(mode)
            self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(before, {path: (self.root / path).read_bytes() for path in before})

    def test_unknown_source_is_rejected_without_partial_write(self):
        source = self.root / next(iter(META['pre']))
        source.write_bytes(source.read_bytes() + b'\n// changed\n')
        before = {path: (self.root / path).read_bytes() for path in META['pre']}
        self.assertNotEqual(self.run_patch('--apply').returncode, 0)
        self.assertEqual(before, {path: (self.root / path).read_bytes() for path in before})


if __name__ == '__main__':
    unittest.main()
