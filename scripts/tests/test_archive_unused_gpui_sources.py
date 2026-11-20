import contextlib
import io
import json
from pathlib import Path
import shutil
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))
from archive_unused_gpui_sources import MANIFEST, check, operate


class LegacyArchiveTests(unittest.TestCase):
    def fixture(self, root):
        for move in json.loads(MANIFEST.read_text())["moves"]:
            destination = root / move["destination"]
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / move["destination"], destination)

    def test_all_archived_files_restore_and_rearchive_byte_for_byte(self):
        with tempfile.TemporaryDirectory() as temporary, contextlib.redirect_stdout(io.StringIO()):
            root = Path(temporary)
            self.fixture(root)
            operate("--check", root)
            operate("--reverse", root)
            operate("--apply", root)
            for move in json.loads(MANIFEST.read_text())["moves"]:
                self.assertFalse((root / move["source"]).exists())
                self.assertEqual((root / move["destination"]).read_bytes(), (ROOT / move["destination"]).read_bytes())

    def test_changed_archive_image_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.fixture(root)
            move = json.loads(MANIFEST.read_text())["moves"][0]
            (root / move["destination"]).write_bytes(b"unexpected change")
            with self.assertRaisesRegex(ValueError, "archive image differs"):
                check(root, archived=True)

    def test_remaining_core_and_deferred_manifests_are_not_archive_targets(self):
        data = json.loads(MANIFEST.read_text())
        self.assertEqual(len(data["directories"]), 8)
        self.assertEqual(len(data["moves"]), 184)
        self.assertNotIn("crates/rustui-gpui", data["directories"])
        for move in data["moves"]:
            self.assertEqual(move["pre_sha256"], move["post_sha256"])
            self.assertTrue(move["destination"].startswith("archive/pre-fast/"))
        for name in ("rustui-gpui-linux", "rustui-gpui-wgpu", "rustui-gpui-windows", "rustui-gpui-web"):
            self.assertNotIn("crates/" + name, data["directories"])
