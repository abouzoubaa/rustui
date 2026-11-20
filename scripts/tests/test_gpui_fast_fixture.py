import hashlib
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "scripts"))

from prepare_gpui_fast_fixture import prepare  # noqa: E402
from prepare_gpui_kit_09_fixture import FixtureError, HashMismatch  # noqa: E402


class GpuiFastFixtureTests(unittest.TestCase):
    def test_hash_mismatch_is_rejected_before_extraction(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            archive_dir = root / "archives"
            archive_dir.mkdir()
            archive = archive_dir / "gpui-fast-0.1.14.crate"
            archive.write_bytes(b"wrong")
            manifest = root / "manifest.json"
            manifest.write_text(
                '{"schema_version":1,"sources":[{"name":"gpui-fast",'
                '"version":"0.1.14","archive_url":"unused",'
                f'"archive_sha256":"{hashlib.sha256(b"right").hexdigest()}"}}]}}',
                encoding="utf-8",
            )
            with self.assertRaises(HashMismatch):
                prepare(manifest, root)
            self.assertFalse((root / "crates/gpui-fast-0.1.14").exists())

    def test_verified_archive_is_extracted_with_cargo_checksum(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            archive = root / "archives/gpui-fast-0.1.14.crate"
            archive.parent.mkdir()
            payload = root / "payload.txt"
            payload.write_text("gpui fast\n", encoding="utf-8")
            with tarfile.open(archive, "w:gz") as bundle:
                bundle.add(payload, arcname="gpui-fast-0.1.14/README.md")
            digest = hashlib.sha256(archive.read_bytes()).hexdigest()
            manifest = root / "manifest.json"
            manifest.write_text(
                '{"schema_version":1,"sources":[{"name":"gpui-fast",'
                '"version":"0.1.14","archive_url":"unused",'
                f'"archive_sha256":"{digest}"}}]}}',
                encoding="utf-8",
            )
            downloaded, extracted = prepare(manifest, root)
            self.assertEqual((downloaded, extracted), (0, 1))
            installed = root / "crates/gpui-fast-0.1.14"
            self.assertEqual((installed / "README.md").read_text(), "gpui fast\n")
            self.assertTrue((installed / ".cargo-checksum.json").is_file())


if __name__ == "__main__":
    unittest.main()
