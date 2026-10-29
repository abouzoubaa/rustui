import importlib.util
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('select_popup_patch', ROOT / 'scripts/apply_gpui_select_popup_rtl_patch.py')
patcher = importlib.util.module_from_spec(spec)
spec.loader.exec_module(patcher)


class SelectPopupPatchTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        for path in [*patcher.IMAGES, *map(Path, patcher.PRESERVED)]:
            target = self.root / path
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / path, target)
        (self.root / 'Cargo.toml').write_text(
            '[patch.crates-io]\ngpui-component = { path = "vendor/gpui-component-0.7.3" }\n'
        )
        patcher.run(self.root, ROOT, '--reverse')

    def test_replay_and_tampering(self):
        baseline = {p: (self.root / p).read_bytes() for p in patcher.IMAGES}
        patcher.run(self.root, ROOT, '--apply')
        patcher.run(self.root, ROOT, '--verify')
        patcher.run(self.root, ROOT, '--reverse')
        self.assertEqual(baseline, {p: (self.root / p).read_bytes() for p in patcher.IMAGES})
        path = next(iter(patcher.IMAGES))
        (self.root / path).write_bytes(baseline[path] + b'\n')
        with self.assertRaises(ValueError):
            patcher.run(self.root, ROOT, '--apply')
        (self.root / path).write_bytes(baseline[path])
        altered = self.root / patcher.PATCH
        altered.parent.mkdir(parents=True, exist_ok=True)
        altered.write_bytes((ROOT / patcher.PATCH).read_bytes() + b'\n')
        with self.assertRaises(ValueError):
            patcher.run(self.root, self.root, '--apply')

    def test_failed_replacement_restores_previous_files(self):
        baseline = {p: (self.root / p).read_bytes() for p in patcher.IMAGES}
        real_replace = patcher.os.replace
        calls = 0

        def fail_second(source, target):
            nonlocal calls
            calls += 1
            if calls == 2:
                raise OSError('simulated replacement failure')
            return real_replace(source, target)

        with patch.object(patcher.os, 'replace', side_effect=fail_second):
            with self.assertRaises(OSError):
                patcher.run(self.root, ROOT, '--apply')
        self.assertEqual(baseline, {p: (self.root / p).read_bytes() for p in patcher.IMAGES})


if __name__ == '__main__':
    unittest.main()
