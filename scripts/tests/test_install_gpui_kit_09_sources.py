from __future__ import annotations

import hashlib
import importlib.util
import json
import tarfile
import tempfile
import unittest
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))
SPEC = importlib.util.spec_from_file_location(
    "install_gpui_kit_09_sources", ROOT / "scripts/install_gpui_kit_09_sources.py"
)
INSTALL = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(INSTALL)


class InstallKit09SourcesTests(unittest.TestCase):
    def add_crate(self, fixture: Path, name: str, version: str, group: str) -> dict[str, str]:
        source_root = fixture / "crates" / f"{name}-{version}"
        (source_root / "src").mkdir(parents=True)
        (source_root / "Cargo.toml").write_text(
            f'[package]\nname="{name}"\nversion="{version}"\n', encoding="utf-8"
        )
        (source_root / "src/lib.rs").write_text(f"pub const VERSION: &str = \"{version}\";\n")
        files = {
            path.relative_to(source_root).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in sorted(source_root.rglob("*"))
            if path.is_file()
        }
        checksum = {"files": files, "package": ""}
        (source_root / ".cargo-checksum.json").write_text(
            json.dumps(checksum, sort_keys=True, separators=(",", ":")) + "\n", encoding="utf-8"
        )
        archive = fixture / "archives" / f"{name}-{version}.crate"
        archive.parent.mkdir(parents=True, exist_ok=True)
        with tarfile.open(archive, "w:gz") as output:
            for path in sorted(source_root.rglob("*")):
                if path.is_file() and path.name != ".cargo-checksum.json":
                    output.add(path, arcname=f"{name}-{version}/{path.relative_to(source_root).as_posix()}")
        digest = hashlib.sha256(archive.read_bytes()).hexdigest()
        checksum["package"] = digest
        (source_root / ".cargo-checksum.json").write_text(
            json.dumps(checksum, sort_keys=True, separators=(",", ":")) + "\n", encoding="utf-8"
        )
        return {
            "name": name,
            "version": version,
            "group": group,
            "archive_url": "unused",
            "archive_sha256": digest,
        }

    def test_install_replaces_only_snapshot_crates_and_refreshes_mirror_sources(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "repo"
            fixture = Path(temporary) / "fixture"
            (root / "vendor").mkdir(parents=True)
            (root / "crates/rustui-gpui/src").mkdir(parents=True)
            (root / "crates/rustui-gpui/Cargo.toml").write_text("owned manifest\n")
            (root / "crates/rustui-gpui/src/lib.rs").write_text("old owned source\n")
            (root / "crates/rustui-gpui-base/src").mkdir(parents=True)
            (root / "crates/rustui-gpui-base/Cargo.toml").write_text("owned base manifest\n")
            (root / "vendor/gpui-pre-0.3.10").mkdir()
            (root / "vendor/gpui-pre-0.3.10/old.rs").write_text("old upstream source\n")
            (root / "vendor/serde-1.0.0").mkdir()
            (root / "vendor/serde-1.0.0/lib.rs").write_text("keep\n")
            (root / "archive/pre-reset-2026-11").mkdir(parents=True)

            sources = [
                self.add_crate(fixture, "gpui-pre", "0.3.12", "gpui-pre"),
                self.add_crate(fixture, "gpui-base", "0.9.1", "gpui-kit"),
            ]
            sources.extend(
                self.add_crate(fixture, package, "2.4.0", "resolved-lock-graph")
                for package in sorted(INSTALL.ADDED_PACKAGES - {"objc2-uniform-type-identifiers"})
            )
            sources.append(
                self.add_crate(fixture, "objc2-uniform-type-identifiers", "0.3.2", "new-lock-dependency")
            )
            manifest_path = Path(temporary) / "sources.json"
            manifest_path.write_text(json.dumps({"schema_version": 1, "sources": sources}), encoding="utf-8")

            verified, mirrors = INSTALL.install(
                fixture, manifest_path, root, root / "archive/pre-reset-2026-11"
            )

            self.assertEqual(verified, 10)
            self.assertEqual(mirrors, 2)
            self.assertFalse((root / "vendor/gpui-pre-0.3.10").exists())
            self.assertTrue((root / "vendor/gpui-pre-0.3.12/.cargo-checksum.json").is_file())
            self.assertEqual((root / "vendor/serde-1.0.0/lib.rs").read_text(), "keep\n")
            self.assertEqual((root / "crates/rustui-gpui/Cargo.toml").read_text(), "owned manifest\n")
            self.assertIn("0.3.12", (root / "crates/rustui-gpui/src/lib.rs").read_text())
            self.assertTrue((root / "archive/pre-reset-2026-11/vendor/gpui-pre-0.3.10/old.rs").is_file())

            installed_source = root / "vendor/gpui-pre-0.3.12/src/lib.rs"
            installed_source.write_text("approved patch hunk\n", encoding="utf-8")
            INSTALL.install(fixture, manifest_path, root, root / "archive/pre-reset-2026-11")
            self.assertEqual(installed_source.read_text(), "approved patch hunk\n")
            self.assertEqual(
                (root / "crates/rustui-gpui/src/lib.rs").read_text(), "approved patch hunk\n"
            )


if __name__ == "__main__":
    unittest.main()
