import contextlib
import io
import json
from pathlib import Path
import shutil
import sys
import tempfile
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))
from refresh_owned_fast_sources import SOURCES, WIRING, operate


class OwnedFastSourceTests(unittest.TestCase):
    def fixture(self, root):
        for copy in json.loads(SOURCES.read_text())["copies"]:
            for key in ("vendor", "owned"):
                destination = root / copy[key]
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copytree(ROOT / copy[key], destination)
        for relative in json.loads(WIRING.read_text())["files"]:
            target = root / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / relative, target)

    def test_creation_wiring_reverse_and_recreation_restore_all_sources(self):
        with tempfile.TemporaryDirectory() as temporary, contextlib.redirect_stdout(io.StringIO()):
            root = Path(temporary)
            self.fixture(root)
            operate("--check", root)
            operate("--reverse", root)
            originals = {name: (root / name).read_bytes()
                         for name in json.loads(WIRING.read_text())["files"]
                         if (root / name).is_file()}
            for copy in json.loads(SOURCES.read_text())["copies"]:
                self.assertFalse((root / copy["owned"]).exists())
            operate("--apply", root)
            operate("--check", root)
            for name in json.loads(WIRING.read_text())["files"]:
                self.assertEqual((root / name).read_bytes(), (ROOT / name).read_bytes())
            operate("--reverse", root)
            self.assertEqual(originals, {name: (root / name).read_bytes() for name in originals})

    def test_modified_vendor_snapshot_is_rejected_before_creating_owned_copy(self):
        with tempfile.TemporaryDirectory() as temporary, contextlib.redirect_stdout(io.StringIO()):
            root = Path(temporary)
            self.fixture(root)
            operate("--reverse", root)
            first = json.loads(SOURCES.read_text())["copies"][0]
            source = root / first["vendor"] / "Cargo.toml"
            source.write_bytes(source.read_bytes() + b"\n# unexpected\n")
            with self.assertRaisesRegex(ValueError, "pinned Fast vendor snapshot differs"):
                operate("--apply", root)
            for copy in json.loads(SOURCES.read_text())["copies"]:
                self.assertFalse((root / copy["owned"]).exists())

    def test_later_wiring_failure_restores_prior_manifests(self):
        import refresh_owned_fast_sources as module
        with tempfile.TemporaryDirectory() as temporary, contextlib.redirect_stdout(io.StringIO()):
            root = Path(temporary)
            self.fixture(root)
            operate("--reverse", root)
            originals = {name: (root / name).read_bytes()
                         for name in json.loads(WIRING.read_text())["files"]
                         if (root / name).is_file()}
            real_apply = module.apply_patch
            later = module.manifest_wiring()[-1]

            def fail_later(path, action, fixture):
                if path == later and action == "--apply":
                    raise ValueError("injected later wiring failure")
                return real_apply(path, action, fixture)

            with mock.patch.object(module, "apply_patch", side_effect=fail_later):
                with self.assertRaisesRegex(ValueError, "injected later wiring failure"):
                    operate("--apply", root)
            self.assertEqual(originals, {name: (root / name).read_bytes() for name in originals})
            for copy in json.loads(SOURCES.read_text())["copies"]:
                self.assertFalse((root / copy["owned"]).exists())

    def test_missing_owned_source_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary, contextlib.redirect_stdout(io.StringIO()):
            root = Path(temporary)
            self.fixture(root)
            first = json.loads(SOURCES.read_text())["copies"][0]
            (root / first["owned"] / "src/gpui.rs").unlink()
            with self.assertRaisesRegex(ValueError, "owned Fast snapshot differs"):
                operate("--check", root)
