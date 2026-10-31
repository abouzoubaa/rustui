from scripts.tests.calendar_patch_fixture import reverse_keyboard_layer
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / 'scripts/apply_gpui_base_calendar_date_limits.py'
SOURCE = Path('vendor/gpui-base-0.7.3/src/calendar.rs')


class CalendarPatchTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        for path in [Path('Cargo.toml'), SOURCE, Path('vendor/gpui-base-0.7.3/Cargo.toml'),
                     Path('vendor/gpui-base-0.7.3/.cargo-checksum.json'),
                     Path('vendor/gpui-base-0.7.3/LICENSE-APACHE')]:
            dest = self.root / path
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / path, dest)

        reverse_keyboard_layer(self.root, SOURCE)

    def run_patch(self, mode):
        return subprocess.run([sys.executable, str(SCRIPT), '--root', str(self.root),
                               '--artifacts', str(ROOT), mode], capture_output=True, text=True)

    def test_apply_verify_and_reverse(self):
        initial = (self.root / SOURCE).read_bytes()
        # A developer checkout may contain either image. Normalize to preimage.
        if self.run_patch('--verify').returncode == 0:
            self.assertEqual(self.run_patch('--reverse').returncode, 0)
        preimage = (self.root / SOURCE).read_bytes()
        for mode in ['--check', '--apply', '--verify', '--reverse']:
            result = self.run_patch(mode)
            self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((self.root / SOURCE).read_bytes(), preimage)
        self.assertTrue(initial)

    def test_unknown_source_is_rejected_without_changes(self):
        source = self.root / SOURCE
        source.write_bytes(source.read_bytes() + b'\n// unexpected\n')
        changed = source.read_bytes()
        self.assertNotEqual(self.run_patch('--apply').returncode, 0)
        self.assertEqual(source.read_bytes(), changed)


if __name__ == '__main__':
    unittest.main()
