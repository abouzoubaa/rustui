import importlib.util
from pathlib import Path
import shutil
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("block_patch", ROOT / "scripts/apply_block_stack_symbol_patch.py")
patch = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(patch)


class BlockPatchTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        for relative in [patch.SOURCE, *map(Path, patch.PRESERVED)]:
            target = self.root / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / relative, target)
        (self.root / "Cargo.toml").write_text('[patch.crates-io]\nblock = { path = "vendor/block-0.1.6" }\n')
        if patch.digest((self.root / patch.SOURCE).read_bytes()) == patch.POST:
            patch.run(self.root, ROOT, "--reverse")

    def test_apply_verify_reverse_and_reject_altered_source(self):
        source = self.root / patch.SOURCE
        original = source.read_bytes()
        patch.run(self.root, ROOT, "--apply")
        self.assertEqual(patch.digest(source.read_bytes()), patch.POST)
        patch.run(self.root, ROOT, "--verify")
        patch.run(self.root, ROOT, "--reverse")
        self.assertEqual(source.read_bytes(), original)
        source.write_bytes(original + b"altered")
        with self.assertRaisesRegex(ValueError, "known preimage or postimage"):
            patch.run(self.root, ROOT, "--apply")

    def test_rejects_altered_metadata_and_wrong_selection(self):
        checksum = self.root / "vendor/block-0.1.6/.cargo-checksum.json"
        checksum.write_bytes(checksum.read_bytes() + b" ")
        with self.assertRaisesRegex(ValueError, "metadata changed"):
            patch.run(self.root, ROOT, "--apply")
        shutil.copyfile(ROOT / "vendor/block-0.1.6/.cargo-checksum.json", checksum)
        (self.root / "Cargo.toml").write_text('[patch.crates-io]\nblock = { path = "other" }\n')
        with self.assertRaisesRegex(ValueError, "selection mismatch"):
            patch.run(self.root, ROOT, "--apply")

    def test_rejects_altered_patch(self):
        artifacts = self.root / "artifacts"
        target = artifacts / patch.PATCH
        target.parent.mkdir(parents=True)
        target.write_bytes((ROOT / patch.PATCH).read_bytes() + b"altered")
        with self.assertRaisesRegex(ValueError, "patch hash mismatch"):
            patch.run(self.root, artifacts, "--apply")


if __name__ == "__main__":
    unittest.main()
