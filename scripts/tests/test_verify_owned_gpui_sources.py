import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

from verify_owned_gpui_sources import compare_trees


class VerifyOwnedGpuSourcesTests(unittest.TestCase):
    def test_accepts_identical_copy_with_manifest_change(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            owned = root / "owned"
            vendor = root / "vendor"
            (owned / "src").mkdir(parents=True)
            (vendor / "src").mkdir(parents=True)
            (owned / "Cargo.toml").write_text('[package]\nname="rustui-gpui"\n')
            (vendor / "Cargo.toml").write_text('[package]\nname="gpui-pre"\n')
            (owned / "src/lib.rs").write_bytes(b"pub fn pinned() {}\n")
            (vendor / "src/lib.rs").write_bytes(b"pub fn pinned() {}\n")

            mismatches, files_checked = compare_trees(owned, vendor)

            self.assertEqual(mismatches, [])
            self.assertEqual(files_checked, 1)

    def test_rejects_changed_source_file(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            owned = root / "owned"
            vendor = root / "vendor"
            owned.mkdir()
            vendor.mkdir()
            (owned / "lib.rs").write_bytes(b"changed\n")
            (vendor / "lib.rs").write_bytes(b"pinned\n")

            mismatches, _ = compare_trees(owned, vendor)

            self.assertEqual(mismatches, ["lib.rs"])

    def test_rejects_missing_source_file(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            owned = root / "owned"
            vendor = root / "vendor"
            owned.mkdir()
            vendor.mkdir()
            (vendor / "lib.rs").write_bytes(b"pinned\n")

            mismatches, _ = compare_trees(owned, vendor)

            self.assertEqual(mismatches, ["lib.rs"])


if __name__ == "__main__":
    unittest.main()
