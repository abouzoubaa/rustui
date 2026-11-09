import hashlib
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "scripts"))

from prepare_gpui_kit_08_fixture import (  # noqa: E402
    HashMismatch,
    MissingArchive,
    MissingSourceFile,
    read_crate_file,
    verify_archive_file,
)


class SourceFixtureTests(unittest.TestCase):
    def test_hash_mismatch_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory) / "crate.crate"
            archive.write_bytes(b"unexpected crate bytes")

            with self.assertRaises(HashMismatch):
                verify_archive_file(archive, hashlib.sha256(b"pinned bytes").hexdigest())

    def test_missing_archive_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory) / "missing.crate"

            with self.assertRaises(MissingArchive):
                verify_archive_file(archive, "0" * 64)

    def test_missing_source_file_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            archive_path = Path(directory) / "gpui-base-0.8.1.crate"
            source_path = Path(directory) / "gpui-base-0.8.1" / "src" / "present.rs"
            source_path.parent.mkdir(parents=True)
            source_path.write_text("pub fn present() {}\n", encoding="utf-8")
            with tarfile.open(archive_path, "w:gz") as archive:
                archive.add(source_path, arcname="gpui-base-0.8.1/src/present.rs")

            with self.assertRaises(MissingSourceFile):
                read_crate_file(archive_path, "gpui-base", "0.8.1", "src/missing.rs")


if __name__ == "__main__":
    unittest.main()
