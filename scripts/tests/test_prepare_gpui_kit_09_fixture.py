import hashlib
import json
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "scripts"))

from prepare_gpui_kit_09_fixture import (  # noqa: E402
    FixtureError,
    HashMismatch,
    MissingArchive,
    MissingSourceFile,
    extract_verified_crate,
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
            archive_path = Path(directory) / "gpui-base-0.9.1.crate"
            source_path = Path(directory) / "gpui-base-0.9.1" / "src" / "present.rs"
            source_path.parent.mkdir(parents=True)
            source_path.write_text("pub fn present() {}\n", encoding="utf-8")
            with tarfile.open(archive_path, "w:gz") as archive:
                archive.add(source_path, arcname="gpui-base-0.9.1/src/present.rs")

            with self.assertRaises(MissingSourceFile):
                read_crate_file(archive_path, "gpui-base", "0.9.1", "src/missing.rs")

    def test_extracts_full_verified_crate_and_writes_directory_checksum(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            archive_path = root / "gpui-base-0.9.1.crate"
            source_file = root / "gpui-base-0.9.1" / "src" / "lib.rs"
            source_file.parent.mkdir(parents=True)
            source_file.write_text("pub fn ready() {}\n", encoding="utf-8")
            with tarfile.open(archive_path, "w:gz") as archive:
                archive.add(source_file, arcname="gpui-base-0.9.1/src/lib.rs")
            source = {
                "name": "gpui-base",
                "version": "0.9.1",
                "archive_sha256": hashlib.sha256(archive_path.read_bytes()).hexdigest(),
            }
            destination = root / "extracted"

            checksum_hash = extract_verified_crate(archive_path, source, destination)

            checksum_path = destination / ".cargo-checksum.json"
            checksum = json.loads(checksum_path.read_text(encoding="utf-8"))
            self.assertEqual((destination / "src/lib.rs").read_text(), "pub fn ready() {}\n")
            self.assertEqual(checksum["package"], source["archive_sha256"])
            self.assertEqual(checksum["files"]["src/lib.rs"], hashlib.sha256(source_file.read_bytes()).hexdigest())
            self.assertEqual(checksum_hash, hashlib.sha256(checksum_path.read_bytes()).hexdigest())

    def test_full_extraction_rejects_archive_path_traversal(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            archive_path = root / "gpui-base-0.9.1.crate"
            payload = root / "payload"
            payload.write_text("escape\n", encoding="utf-8")
            with tarfile.open(archive_path, "w:gz") as archive:
                archive.add(payload, arcname="gpui-base-0.9.1/../escape")
            source = {
                "name": "gpui-base",
                "version": "0.9.1",
                "archive_sha256": hashlib.sha256(archive_path.read_bytes()).hexdigest(),
            }
            with self.assertRaisesRegex(FixtureError, "unsafe or unexpected archive entry"):
                extract_verified_crate(archive_path, source, root / "extracted")


if __name__ == "__main__":
    unittest.main()
