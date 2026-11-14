import sys
import tempfile
import unittest
import hashlib
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

    def test_accepts_independently_regenerated_macos_test_lock(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            owned = root / "owned"
            vendor = root / "vendor"
            owned.mkdir()
            vendor.mkdir()
            (owned / "Cargo.toml").write_text("[package]\nname='rustui-gpui-macos'\n")
            (vendor / "Cargo.toml").write_text("[package]\nname='gpui-pre-macos'\n")
            (owned / "Cargo.lock").write_text("version = 4\n")
            (vendor / "Cargo.lock").write_text("version = 3\n")
            (owned / "lib.rs").write_bytes(b"same\n")
            (vendor / "lib.rs").write_bytes(b"same\n")

            mismatches, files_checked = compare_trees(
                owned, vendor, {"Cargo.lock"}
            )

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

    def test_accepts_only_candidate_manifest_bound_owned_overlay(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            owned = root / "owned"
            vendor = root / "vendor"
            owned.mkdir()
            vendor.mkdir()
            (owned / "tab_stops.rs").write_bytes(b"fast backend source\n")
            (vendor / "tab_stops.rs").write_bytes(b"upstream source\n")
            expected = hashlib.sha256(b"fast backend source\n").hexdigest()

            mismatches, _ = compare_trees(
                owned, vendor, expected_owned_overlays={"tab_stops.rs": expected}
            )
            self.assertEqual(mismatches, [])

            (owned / "tab_stops.rs").write_bytes(b"untracked source change\n")
            mismatches, _ = compare_trees(
                owned, vendor, expected_owned_overlays={"tab_stops.rs": expected}
            )
            self.assertEqual(mismatches, ["tab_stops.rs"])


if __name__ == "__main__":
    unittest.main()
