import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("row_a11y_proposal", ROOT / "scripts/check_gpui_row_accessibility_proposal.py")
PROPOSAL = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PROPOSAL)


class RowAccessibilityProposalTests(unittest.TestCase):
    def fixture(self, root):
        manifest = json.loads((ROOT / PROPOSAL.MANIFEST).read_text())
        paths = [PROPOSAL.MANIFEST]
        for record in manifest["sources"]:
            package = Path("vendor") / record["package_directory"]
            paths.extend([Path(record["source"]), Path(record["patch"]), package / "Cargo.toml", package / ".cargo-checksum.json"])
            if record["vcs_metadata_sha256"] is not None:
                paths.append(package / ".cargo_vcs_info.json")
        for path in paths:
            (root / path).parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / path, root / path)
        return manifest

    def test_exact_disposable_roundtrip_preserves_selected_sources(self):
        self.assertEqual(PROPOSAL.verify(ROOT), {"status": "pass", "sources": 2, "apply_reverse": "exact", "selected_sources": "unchanged"})

    def test_changed_patch_rejects_without_touching_sources(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = self.fixture(root)
            patch = root / manifest["sources"][0]["patch"]
            patch.write_bytes(patch.read_bytes() + b"\n")
            with self.assertRaisesRegex(ValueError, "hash mismatch"):
                PROPOSAL.verify(root)
            for record in manifest["sources"]:
                self.assertEqual(PROPOSAL.digest(root / record["source"]), record["pre_sha256"])

    def test_partial_source_image_rejects_without_repair(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = self.fixture(root)
            source = root / manifest["sources"][1]["source"]
            source.write_bytes(source.read_bytes() + b"\n")
            before = source.read_bytes()
            with self.assertRaisesRegex(ValueError, "hash mismatch"):
                PROPOSAL.verify(root)
            self.assertEqual(source.read_bytes(), before)


if __name__ == "__main__":
    unittest.main()
