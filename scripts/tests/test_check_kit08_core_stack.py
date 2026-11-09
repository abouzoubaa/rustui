import hashlib
import json
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

from check_kit08_core_stack import run  # noqa: E402
from prepare_gpui_kit_08_fixture import MissingSourceFile  # noqa: E402


class Kit08CoreStackTests(unittest.TestCase):
    def make_fixture(self, root: Path, include_target: bool = True) -> tuple[Path, Path]:
        package = "gpui-pre"
        version = "0.3.10"
        archive_path = root / "gpui-pre-0.3.10.crate"
        target = root / "source.rs"
        target.write_text("pub fn seed() {}\n", encoding="utf-8")
        with tarfile.open(archive_path, "w:gz") as archive:
            if include_target:
                archive.add(target, arcname="gpui-pre-0.3.10/src/sample.rs")
        manifest = {
            "schema_version": 1,
            "upstream": {"repository": "test", "tag": "v0.8.1", "commit": "test"},
            "sources": [
                {
                    "name": package,
                    "version": version,
                    "archive_url": archive_path.as_uri(),
                    "archive_sha256": hashlib.sha256(archive_path.read_bytes()).hexdigest(),
                }
            ],
        }
        manifest_path = root / "manifest.json"
        manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
        return manifest_path, archive_path

    def write_patch(self, directory: Path) -> None:
        patch = directory / "test.patch"
        patch.write_text(
            "--- a/vendor/gpui-pre-0.3.10/src/sample.rs\n"
            "+++ b/vendor/gpui-pre-0.3.10/src/sample.rs\n"
            "@@ -1 +1,2 @@\n"
            " pub fn seed() {}\n"
            "+pub fn added() {}\n",
            encoding="utf-8",
        )

    def test_candidate_stack_applies_and_reverse_restores_fixture(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest_path, _ = self.make_fixture(root)
            patch_dir = root / "patches"
            patch_dir.mkdir()
            self.write_patch(patch_dir)

            output = root / "core-manifest.json"
            result = run(manifest_path, patch_dir, output)

            self.assertTrue(result["reverse_verified"])
            self.assertEqual(
                result["files"]["vendor/gpui-pre-0.3.10/src/sample.rs"]["pre_sha256"],
                hashlib.sha256(b"pub fn seed() {}\n").hexdigest(),
            )
            self.assertEqual(json.loads(output.read_text())["order"], ["test.patch"])

    def test_missing_patch_target_file_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest_path, _ = self.make_fixture(root, include_target=False)
            patch_dir = root / "patches"
            patch_dir.mkdir()
            self.write_patch(patch_dir)

            with self.assertRaises(MissingSourceFile):
                run(manifest_path, patch_dir, root / "core-manifest.json")


if __name__ == "__main__":
    unittest.main()
