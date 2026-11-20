"""Exact creation/deletion and layered stack replay, without touching real sources."""
import contextlib
import hashlib
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))
from apply_f0_fast_candidate_patch import run
from check_fast_patch_stack import verify
from check_f0_fast_core_stack import run as core_stack


def new_file_manifest(directory, relative="vendor/example/src/fast/damage/new.rs"):
    content = b"pub const VALUE: u32 = 42;\n"
    patch = directory / "new.patch"
    patch.write_text(f"diff --git a/{relative} b/{relative}\n"
                     "new file mode 100644\n--- /dev/null\n"
                     f"+++ b/{relative}\n@@ -0,0 +1 @@\n+" + content.decode())
    manifest = directory / "new.json"
    manifest.write_text(json.dumps({
        "name": "new-file-test", "patch": patch.name,
        "patch_sha256": hashlib.sha256(patch.read_bytes()).hexdigest(),
        "files": {relative: {"pre_sha256": None,
                             "post_sha256": hashlib.sha256(content).hexdigest()}},
    }))
    return manifest, relative, content


class NewFilePatchTests(unittest.TestCase):
    def test_check_apply_reverse_and_layered_stack_restore_absence(self):
        with tempfile.TemporaryDirectory() as temp, contextlib.redirect_stdout(io.StringIO()):
            directory = Path(temp)
            root = directory / "root"
            root.mkdir()
            manifest, relative, content = new_file_manifest(directory)
            target = root / relative
            run(manifest, "--check", root)
            self.assertFalse(target.exists())
            run(manifest, "--apply", root)
            self.assertEqual(target.read_bytes(), content)
            run(manifest, "--check", root)
            base = directory / "base.json"
            base.write_text(json.dumps({"patches": []}))
            stack = directory / "stack.json"
            stack.write_text(json.dumps({"base_stack": base.name, "patches": [manifest.name]}))
            verify(root, stack)
            core_stack("--check", root, stack)
            self.assertEqual(target.read_bytes(), content)
            run(manifest, "--reverse", root)
            self.assertFalse(target.exists())
            run(manifest, "--check", root)

    def test_tampered_new_file_patch_is_rejected_without_creating_a_file(self):
        with tempfile.TemporaryDirectory() as temp:
            directory = Path(temp)
            root = directory / "root"
            root.mkdir()
            manifest, relative, _ = new_file_manifest(directory)
            patch = directory / "new.patch"
            patch.write_bytes(patch.read_bytes().replace(b"42", b"43"))
            with self.assertRaisesRegex(ValueError, "patch hash mismatch"):
                run(manifest, "--apply", root)
            data = json.loads(manifest.read_text())
            data["patch_sha256"] = hashlib.sha256(patch.read_bytes()).hexdigest()
            manifest.write_text(json.dumps(data))
            with self.assertRaisesRegex(ValueError, "patch result hash mismatch"):
                run(manifest, "--apply", root)
            self.assertFalse((root / relative).exists())

    def test_existing_empty_file_and_symlink_are_not_absence(self):
        with tempfile.TemporaryDirectory() as temp:
            directory = Path(temp)
            manifest, relative, _ = new_file_manifest(directory)
            root = directory / "root"
            target = root / relative
            target.parent.mkdir(parents=True)
            target.write_bytes(b"")
            with self.assertRaisesRegex(ValueError, "unexpected hash"):
                run(manifest, "--apply", root)
            target.unlink()
            target.symlink_to(directory / "missing")
            with self.assertRaisesRegex(ValueError, "non-regular target"):
                run(manifest, "--apply", root)


if __name__ == "__main__":
    unittest.main()
