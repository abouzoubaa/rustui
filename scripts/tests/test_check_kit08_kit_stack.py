import hashlib
import json
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

from check_kit08_core_stack import apply_stack  # noqa: E402
from check_kit08_kit_stack import A2_4D_ORDER, A2_4E_ORDER, A2_4F_ORDER  # noqa: E402


class Kit08FamilyStackTests(unittest.TestCase):
    def test_menu_provider_precedes_dropdown_refresh_consumer(self):
        self.assertEqual(
            A2_4D_ORDER,
            (
                "gpui-component-popup-menu-disabled-confirm-v1-kit08.patch",
                "r15b-menu-item-disabled-v1.patch",
                "r15b-popup-menu-recovery-v1.patch",
                "r15b-dropdown-menu-refresh-v1.patch",
            ),
        )

    def test_select_rtl_provider_precedes_alignment_and_auto_width(self):
        self.assertEqual(
            A2_4E_ORDER,
            (
                "gpui-component-select-popup-rtl-v1-kit08.patch",
                "gpui-component-select-rtl-alignment-v1.patch",
                "gpui-component-select-rtl-alignment-v2-kit08.patch",
                "gpui-component-select-rtl-alignment-v3-kit08.patch",
                "gpui-component-select-rtl-auto-width-v4-kit08.patch",
                "gpui-component-select-rtl-auto-width-v6-kit08.patch",
            ),
        )

    def test_selectable_list_semantics_follows_select_stack(self):
        self.assertEqual(A2_4F_ORDER, ("gpui-selectable-list-semantics-v2.patch",))

    def test_core_then_family_applies_and_reverse_restores_fixture(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "source.rs"
            source.write_text("pub fn seed() {}\n", encoding="utf-8")
            archive_path = root / "gpui-pre-0.3.10.crate"
            with tarfile.open(archive_path, "w:gz") as archive:
                archive.add(source, arcname="gpui-pre-0.3.10/src/sample.rs")
            manifest = {
                "schema_version": 1,
                "upstream": {"repository": "test", "tag": "v0.8.1", "commit": "test"},
                "sources": [{
                    "name": "gpui-pre", "version": "0.3.10",
                    "archive_url": archive_path.as_uri(),
                    "archive_sha256": hashlib.sha256(archive_path.read_bytes()).hexdigest(),
                }],
            }
            fixture = root / "fixture"
            target = fixture / "sources/gpui-pre-0.3.10/src/sample.rs"
            target.parent.mkdir(parents=True)
            target.write_text("pub fn seed() {}\n", encoding="utf-8")
            patches = root / "patches"
            patches.mkdir()
            (patches / "core.patch").write_text(
                "--- a/vendor/gpui-pre-0.3.10/src/sample.rs\n"
                "+++ b/vendor/gpui-pre-0.3.10/src/sample.rs\n"
                "@@ -1 +1,2 @@\n pub fn seed() {}\n+pub fn core() {}\n",
                encoding="utf-8",
            )
            (patches / "family.patch").write_text(
                "--- a/vendor/gpui-pre-0.3.10/src/sample.rs\n"
                "+++ b/vendor/gpui-pre-0.3.10/src/sample.rs\n"
                "@@ -1,2 +1,3 @@\n pub fn seed() {}\n pub fn core() {}\n+pub fn family() {}\n",
                encoding="utf-8",
            )
            observed = {}

            def after_apply(stage, targets, _manifest, _fixture):
                observed["targets"] = targets
                observed["content"] = (stage / "vendor/gpui-pre-0.3.10/src/sample.rs").read_text(
                    encoding="utf-8"
                )

            result = apply_stack(
                manifest, patches, fixture, "manifest-hash",
                patch_order=("core.patch", "family.patch"),
                after_apply=after_apply,
            )
            self.assertTrue(result["reverse_verified"])
            self.assertEqual(result["order"], ["core.patch", "family.patch"])
            self.assertEqual(observed["content"], "pub fn seed() {}\npub fn core() {}\npub fn family() {}\n")
            self.assertEqual(
                observed["targets"]["vendor/gpui-pre-0.3.10/src/sample.rs"],
                ("gpui-pre", "src/sample.rs"),
            )
            self.assertEqual(
                result["files"]["vendor/gpui-pre-0.3.10/src/sample.rs"]["pre_sha256"],
                hashlib.sha256(b"pub fn seed() {}\n").hexdigest(),
            )


if __name__ == "__main__":
    unittest.main()
