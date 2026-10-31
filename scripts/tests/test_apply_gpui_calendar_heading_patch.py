import importlib.util
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch as mock_patch

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    'calendar_heading_patch', ROOT / 'scripts/apply_gpui_calendar_heading_patch.py'
)
heading = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(heading)


class CalendarHeadingPatchTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        files = [*heading.IMAGES, *map(Path, heading.PRESERVED)]
        files += [Path(f'vendor/{name}-0.7.3/Cargo.toml') for name in ('gpui-base', 'gpui-component')]
        for relative in files:
            target = self.root / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / relative, target)
        (self.root / 'Cargo.toml').write_text(
            '[patch.crates-io]\ngpui-base = { path = "vendor/gpui-base-0.7.3" }\n'
            'gpui-component = { path = "vendor/gpui-component-0.7.3" }\n'
        )
        heading.run(self.root, ROOT, '--reverse')

    def test_replay_and_reject_tampering(self):
        original = {p: (self.root / p).read_bytes() for p in heading.IMAGES}
        heading.run(self.root, ROOT, '--apply')
        heading.run(self.root, ROOT, '--verify')
        heading.run(self.root, ROOT, '--reverse')
        self.assertEqual(original, {p: (self.root / p).read_bytes() for p in heading.IMAGES})
        path = next(iter(heading.IMAGES))
        (self.root / path).write_bytes(original[path] + b'tampered')
        with self.assertRaisesRegex(ValueError, 'source images mismatch'):
            heading.run(self.root, ROOT, '--apply')

    def test_rejects_patch_and_checksum_tampering(self):
        artifact = self.root / heading.PATCH
        artifact.parent.mkdir(parents=True, exist_ok=True)
        artifact.write_bytes((ROOT / heading.PATCH).read_bytes() + b'tampered')
        with self.assertRaisesRegex(ValueError, 'patch hash mismatch'):
            heading.run(self.root, self.root, '--apply')
        checksum = self.root / next(iter(heading.PRESERVED))
        checksum.write_bytes(checksum.read_bytes() + b'tampered')
        with self.assertRaisesRegex(ValueError, 'upstream metadata changed'):
            heading.run(self.root, ROOT, '--apply')



if __name__ == '__main__':
    unittest.main()
