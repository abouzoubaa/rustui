import importlib.util
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch as mock_patch

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    'calendar_keyboard_patch', ROOT / 'scripts/apply_gpui_calendar_keyboard_patch.py'
)
keyboard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(keyboard)


class CalendarKeyboardPatchTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        files = [*keyboard.IMAGES, *map(Path, keyboard.PRESERVED)]
        files += [Path(f'vendor/{name}-0.7.3/Cargo.toml') for name in ('gpui-base', 'gpui-component')]
        for relative in files:
            target = self.root / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / relative, target)
        (self.root / 'Cargo.toml').write_text(
            '[patch.crates-io]\ngpui-base = { path = "vendor/gpui-base-0.7.3" }\n'
            'gpui-component = { path = "vendor/gpui-component-0.7.3" }\n'
        )
        keyboard.run(self.root, ROOT, '--reverse')

    def test_replay_and_reject_tampering(self):
        original = {p: (self.root / p).read_bytes() for p in keyboard.IMAGES}
        keyboard.run(self.root, ROOT, '--apply')
        keyboard.run(self.root, ROOT, '--verify')
        keyboard.run(self.root, ROOT, '--reverse')
        self.assertEqual(original, {p: (self.root / p).read_bytes() for p in keyboard.IMAGES})
        path = next(iter(keyboard.IMAGES))
        (self.root / path).write_bytes(original[path] + b'tampered')
        with self.assertRaisesRegex(ValueError, 'source images mismatch'):
            keyboard.run(self.root, ROOT, '--apply')

    def test_rejects_patch_and_checksum_tampering(self):
        artifact = self.root / keyboard.PATCH
        artifact.parent.mkdir(parents=True, exist_ok=True)
        artifact.write_bytes((ROOT / keyboard.PATCH).read_bytes() + b'tampered')
        with self.assertRaisesRegex(ValueError, 'patch hash mismatch'):
            keyboard.run(self.root, self.root, '--apply')
        checksum = self.root / next(iter(keyboard.PRESERVED))
        checksum.write_bytes(checksum.read_bytes() + b'tampered')
        with self.assertRaisesRegex(ValueError, 'upstream metadata changed'):
            keyboard.run(self.root, ROOT, '--apply')

    def test_second_replace_failure_restores_first_file(self):
        original = {p: (self.root / p).read_bytes() for p in keyboard.IMAGES}
        real_replace = keyboard.os.replace
        calls = 0

        def fail_second(source, target):
            nonlocal calls
            calls += 1
            if calls == 2:
                raise OSError('simulated replacement failure')
            return real_replace(source, target)

        with mock_patch.object(keyboard.os, 'replace', side_effect=fail_second):
            with self.assertRaisesRegex(OSError, 'simulated replacement failure'):
                keyboard.run(self.root, ROOT, '--apply')
        self.assertEqual(original, {p: (self.root / p).read_bytes() for p in keyboard.IMAGES})


if __name__ == '__main__':
    unittest.main()
