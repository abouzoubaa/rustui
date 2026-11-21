"""Keep the approved Kit gate strict when explicitly layered candidates exist."""
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))
from apply_gpui_kit_09_stack import check_postimages
from apply_f0_fast_candidate_patch import run as apply_candidate


def digest(value):
    return hashlib.sha256(value).hexdigest()


class KitCandidateCheckTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.source = "vendor/gpui-component-0.9.1/src/a.rs"
        self.created = "vendor/gpui-component-0.9.1/src/new.rs"
        self.stack = {"files": {self.source: {"post_sha256": digest(b"old\n")}}}
        self.write(self.source, b"old\n")

    def write(self, path, data):
        path = self.root / path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)

    def candidate(self):
        text = (f"diff --git a/{self.source} b/{self.source}\n"
                f"--- a/{self.source}\n+++ b/{self.source}\n"
                "@@ -1 +1 @@\n-old\n+new\n"
                f"diff --git a/{self.created} b/{self.created}\nnew file mode 100644\n"
                f"--- /dev/null\n+++ b/{self.created}\n@@ -0,0 +1 @@\n+created\n")
        self.write("patches/proposed/example.patch", text.encode())
        self.manifest = self.root / "patches/proposed/example.json"
        self.manifest.write_text(json.dumps({"patch": "example.patch", "patch_sha256": digest(text.encode()),
            "files": {self.source: {"pre_sha256": digest(b"old\n"), "post_sha256": digest(b"new\n")},
                      self.created: {"pre_sha256": None, "post_sha256": digest(b"created\n")}}}))
        self.write("patches/candidate-fast-damage/damage-stack.json",
                   json.dumps({"patches": ["../proposed/example.json"]}).encode())
        self.write(self.source, b"new\n")
        self.write(self.created, b"created\n")

    def test_original_approved_state_still_passes(self):
        check_postimages(self.stack, self.root)

    def test_unknown_change_without_candidate_is_rejected(self):
        self.write(self.source, b"unknown\n")
        with self.assertRaisesRegex(RuntimeError, "postimage mismatch"):
            check_postimages(self.stack, self.root)

    def test_candidate_proves_absence_on_reverse_and_exact_reapplication(self):
        self.candidate()
        actions = []
        def observe(path, action, fixture):
            apply_candidate(path, action, fixture)
            actions.append(action)
            if action == "--reverse":
                self.assertFalse((fixture / self.created).exists())
                self.assertEqual((fixture / self.source).read_bytes(), b"old\n")
        with patch("apply_f0_fast_candidate_patch.run", side_effect=observe):
            check_postimages(self.stack, self.root)
        self.assertEqual(actions, ["--reverse", "--apply"])
        self.assertEqual((self.root / self.created).read_bytes(), b"created\n")
        self.assertEqual((self.root / self.source).read_bytes(), b"new\n")

    def test_candidate_must_bridge_from_approved_hash(self):
        self.candidate()
        data = json.loads(self.manifest.read_text())
        data["files"][self.source]["pre_sha256"] = digest(b"unapproved\n")
        self.manifest.write_text(json.dumps(data))
        with self.assertRaisesRegex(RuntimeError, "does not chain from approved"):
            check_postimages(self.stack, self.root)

    def test_tampered_patch_is_rejected(self):
        self.candidate()
        self.write("patches/proposed/example.patch", b"tampered\n")
        with self.assertRaisesRegex(ValueError, "patch hash mismatch"):
            check_postimages(self.stack, self.root)

    def test_tampered_new_file_is_rejected(self):
        self.candidate()
        self.write(self.created, b"tampered\n")
        with self.assertRaisesRegex(RuntimeError, "postimage mismatch"):
            check_postimages(self.stack, self.root)

    def checksum_candidate(self):
        # Historical approved Kit tables omitted Cargo checksum metadata and
        # did not normalize its file map. Preserve that exact preimage.
        self.candidate()
        name = "vendor/gpui-component-0.9.1/.cargo-checksum.json"
        old, new = b'{"files":{"src/a.rs":"legacy"}}\n', b'{"files":{"src/a.rs":"candidate"}}\n'
        patch_path = self.manifest.with_suffix(".patch")
        text = patch_path.read_text() + (f"diff --git a/{name} b/{name}\n"
            f"--- a/{name}\n+++ b/{name}\n@@ -1 +1 @@\n-{old.decode()}+{new.decode()}")
        patch_path.write_text(text)
        data = json.loads(self.manifest.read_text())
        data["patch_sha256"] = digest(text.encode())
        data["files"][name] = {"pre_sha256": digest(old), "post_sha256": digest(new)}
        self.manifest.write_text(json.dumps(data))
        self.write(name, new)
        return name

    def test_explicit_checksum_preimage_reverses_without_changing_approved_sources(self):
        name = self.checksum_candidate()
        check_postimages(self.stack, self.root)
        self.assertIn(b'candidate', (self.root / name).read_bytes())

    def test_tampered_checksum_postimage_is_rejected(self):
        name = self.checksum_candidate()
        self.write(name, b"unknown\n")
        with self.assertRaisesRegex(RuntimeError, "postimage mismatch"):
            check_postimages(self.stack, self.root)

    def test_existing_source_without_approved_anchor_is_rejected(self):
        self.candidate()
        data = json.loads(self.manifest.read_text())
        data["files"]["vendor/gpui-component-0.9.1/src/unanchored.rs"] = {
            "pre_sha256": digest(b"old\n"), "post_sha256": digest(b"new\n")}
        self.manifest.write_text(json.dumps(data))
        with self.assertRaisesRegex(RuntimeError, "lacks an approved base image"):
            check_postimages(self.stack, self.root)

    def test_unsafe_target_is_rejected_before_fixture_copy(self):
        self.candidate()
        original = self.manifest.read_text()
        for target in ("../outside", "/outside"):
            with self.subTest(target=target):
                data = json.loads(original)
                data["files"][target] = {"pre_sha256": None, "post_sha256": digest(b"new\n")}
                self.manifest.write_text(json.dumps(data))
                with self.assertRaisesRegex(ValueError, "unsafe candidate target"):
                    check_postimages(self.stack, self.root)

    def test_symlink_target_is_rejected_before_fixture_copy(self):
        self.candidate()
        self.write("elsewhere", b"created\n")
        (self.root / self.created).unlink()
        (self.root / self.created).symlink_to(self.root / "elsewhere")
        with self.assertRaisesRegex(RuntimeError, "non-regular candidate target"):
            check_postimages(self.stack, self.root)


if __name__ == "__main__":
    unittest.main()
