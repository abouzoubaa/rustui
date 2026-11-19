import importlib.util
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location("invalidation",ROOT/"scripts/invalidate_patched_vendor_cache.py")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class PatchCacheTests(unittest.TestCase):
    def test_changed_patch_rebuilds_registry_and_transitive_consumers_only(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root/"Cargo.toml").write_text('[workspace.dependencies]\ngpui = {package="gpui-fast",version="=0.1.14"}\n')
            (root/"Cargo.lock").write_text('''version = 4
[[package]]
name = "gpui-fast"
version = "0.1.14"
[[package]]
name = "gpui-base"
version = "0.9.1"
dependencies = ["gpui-fast 0.1.14"]
[[package]]
name = "rustui"
version = "0.4.0"
dependencies = ["gpui-base"]
''')
            patches = root/"patches/candidate-kit-fast"
            patches.mkdir(parents=True)
            patch = patches/"core.patch"
            patch.write_text("first")
            target = root/"build"
            stamps = []
            for profile in ("debug","release","aarch64-apple-darwin/debug"):
                for package in ("gpui-fast","gpui-base","rustui","serde"):
                    unit = target/profile/".fingerprint"/(package+"-123456")
                    unit.mkdir(parents=True)
                    stamp = unit/("lib-"+package.replace("-","_"))
                    stamp.write_text("old fingerprint")
                    (unit/(stamp.name+".json")).write_text("metadata")
                    stamps.append((package,stamp))
                (target/profile/"keep.rlib").write_text("compiled artifact")
            self.assertEqual(module.invalidate(root,target),9)
            for package,stamp in stamps:
                self.assertEqual(stamp.exists(),package=="serde")
                self.assertTrue(stamp.with_name(stamp.name+".json").exists())
            self.assertEqual(module.invalidate(root,target),0)
            for package,stamp in stamps:
                if package!="serde":stamp.write_text("new fingerprint")
            patch.write_text("second")
            self.assertEqual(module.invalidate(root,target),9)
            self.assertTrue((target/"release/keep.rlib").is_file())

    def test_gpui_pre_baseline_cache_is_untouched(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root/"Cargo.toml").write_text('[workspace.dependencies]\ngpui={package="gpui-pre",version="0.3.12"}\n')
            self.assertEqual(module.invalidate(root,root/"target"),0)
            self.assertFalse((root/"target").exists())
