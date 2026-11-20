import contextlib
import difflib
import hashlib
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

    def test_new_files_and_candidate_manifest_features_regenerate_and_reverse(self):
        from apply_f0_fast_candidate_patch import run
        from refresh_owned_fast_sources import DAMAGE_STACK
        from test_fast_new_file_patches import new_file_manifest
        from check_fast_patch_stack import active_stack, load_manifests, verify
        with tempfile.TemporaryDirectory() as temporary, contextlib.redirect_stdout(io.StringIO()):
            directory = Path(temporary)
            root = directory / "root"
            self.fixture(root)
            for name in load_manifests(active_stack())[1]:
                target = root / name
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes((ROOT / name).read_bytes())
            copy = next(item for item in json.loads(SOURCES.read_text())["copies"]
                        if item["vendor"] == "vendor/gpui-fast-apple-0.1.14")
            original_owned = (root / copy["owned"] / "Cargo.toml").read_text()
            operate("--reverse", root)
            relative = copy["vendor"] + "/src/fast/damage/test_creation.rs"
            manifest, relative, content = new_file_manifest(directory, relative)
            cargo = copy["vendor"] + "/Cargo.toml"
            before = (root / cargo).read_text()
            after = before.replace('    "CALayer",', '    "CALayer",\n    "CAFrameRateRange",')
            self.assertNotEqual(before, after)
            patch = directory / "new.patch"
            patch.write_text(patch.read_text() + f"diff --git a/{cargo} b/{cargo}\n" +
                             "".join(difflib.unified_diff(before.splitlines(True), after.splitlines(True),
                                                         fromfile=f"a/{cargo}", tofile=f"b/{cargo}")))
            data = json.loads(manifest.read_text())
            data["patch_sha256"] = hashlib.sha256(patch.read_bytes()).hexdigest()
            data["files"][cargo] = {"pre_sha256": hashlib.sha256(before.encode()).hexdigest(),
                                     "post_sha256": hashlib.sha256(after.encode()).hexdigest()}
            manifest.write_text(json.dumps(data))
            stack = directory / "damage-stack.json"
            stack.write_text(json.dumps({"base_stack": str(DAMAGE_STACK), "patches": [manifest.name]}))
            run(manifest, "--apply", root)
            operate("--apply", root, stack)
            operate("--check", root, stack)
            owned = root / copy["owned"] / "src/fast/damage/test_creation.rs"
            self.assertEqual(owned.read_bytes(), content)
            self.assertEqual((root / copy["owned"] / "Cargo.toml").read_text(),
                             original_owned.replace('    "CALayer",', '    "CALayer",\n    "CAFrameRateRange",'))
            verify(root, stack)
            # Historical core proofs must still start from the approved images
            # when the active candidate also changes an owned manifest.
            import check_fast_patch_stack
            import test_f0_fast_core_stack
            peeled = directory / "approved"
            with mock.patch.object(check_fast_patch_stack, "DAMAGE_STACK", stack), \
                 mock.patch.object(test_f0_fast_core_stack, "REPO_ROOT", root):
                test_f0_fast_core_stack.approved_fixture(peeled)
            check_fast_patch_stack.check(peeled, check_fast_patch_stack.STACK)
            operate("--reverse", root, stack)
            self.assertFalse(owned.exists())
            run(manifest, "--reverse", root)
            self.assertFalse((root / relative).exists())
            operate("--apply", root)
            operate("--check", root)
            self.assertFalse(owned.exists())
            self.assertEqual((root / copy["owned"] / "Cargo.toml").read_text(), original_owned)
