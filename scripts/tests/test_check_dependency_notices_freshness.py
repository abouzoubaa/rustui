import importlib.util
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
spec = importlib.util.spec_from_file_location("notice_freshness", SCRIPTS / "check_dependency_notices_freshness.py")
checker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checker)


class NoticeFreshnessTests(unittest.TestCase):
    def test_lockfile_change_invalidates_packaged_notices(self):
        source = SCRIPTS.parent
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for path in ("docs/release/current-notices", "docs/release/notice-supplements", "crates/rustui/assets/fonts"):
                shutil.copytree(source / path, root / path)
            shutil.copyfile(source / "Cargo.lock", root / "Cargo.lock")
            self.assertGreater(checker.check(root), 0)
            with (root / "Cargo.lock").open("a") as lock:
                lock.write("\n# changed\n")
            with self.assertRaisesRegex(ValueError, "out of date with Cargo.lock"):
                checker.check(root)


if __name__ == "__main__":
    unittest.main()
