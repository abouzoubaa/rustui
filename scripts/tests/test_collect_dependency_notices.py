import importlib.util
import json
import subprocess
import sys
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

SCRIPT = Path(__file__).resolve().parents[1] / "collect_dependency_notices.py"
spec = importlib.util.spec_from_file_location("notice_collector", SCRIPT)
collector = importlib.util.module_from_spec(spec)
spec.loader.exec_module(collector)


def dump(path, value):
    path.write_text(json.dumps(value), encoding="utf-8")


class CollectorTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.base = Path(temporary.name)
        self.repo = self.base / "repo"
        self.dep = self.repo / "vendor" / "dep"
        self.dep.mkdir(parents=True)
        (self.repo / "Cargo.toml").write_text("[workspace]\n")
        (self.repo / "LICENSE").write_text("RustUI root license\n")
        (self.dep / "Cargo.toml").write_text("[package]\n")
        (self.dep / "COPYING").write_text("verbatim copying text\n")
        (self.dep / "licenses").mkdir()
        (self.dep / "licenses" / "NOTICE.extra").write_text("verbatim notice\n")
        self.root_id, self.dep_id = "root-id", "dependency-id"
        self.packages = [
            {"id": self.root_id, "name": "rustui", "version": "1.0.0", "license": "MIT",
             "source": None, "manifest_path": str(self.repo / "Cargo.toml")},
            {"id": self.dep_id, "name": "dep", "version": "1.0.0", "license": "MIT",
             "source": "registry+x", "manifest_path": str(self.dep / "Cargo.toml")},
            {"id": "unused-id", "name": "unused", "version": "9.9.9", "license": "MIT",
             "source": "registry+x", "manifest_path": str(self.dep / "Cargo.toml")},
        ]
        self.metadata = self.base / "metadata.json"
        self.save_metadata()
        self.build = self.base / "build.jsonl"
        self.build.write_text("\n".join(json.dumps(msg) for msg in [
            {"reason": "compiler-artifact", "package_id": self.root_id},
            {"reason": "compiler-artifact", "package_id": self.dep_id},
            {"reason": "build-finished", "success": True},
        ]) + "\n")
        self.out = self.base / "out"
        self.args = SimpleNamespace(repo_root=self.repo, source_root=[], metadata=self.metadata,
            build_json=self.build, supplement_manifest=None, supplement_root=None, output=self.out)

    def save_metadata(self):
        dump(self.metadata, {"packages": self.packages, "workspace_members": [self.root_id]})

    def remove_notices(self):
        (self.dep / "COPYING").unlink()
        (self.dep / "licenses" / "NOTICE.extra").unlink()
        (self.dep / "licenses").rmdir()

    def add_supplement(self):
        root = self.base / "supplement"
        root.mkdir()
        notice = root / "LICENSE-MIT"
        notice.write_text("supplement verbatim\n")
        revision = "a" * 40
        package = self.packages[1]
        record = {
            "package_id": self.dep_id, "name": package["name"], "version": package["version"],
            "source": package["source"], "manifest_sha256": collector.sha256(Path(package["manifest_path"])),
            "upstream": {"repository": "example/dep", "revision": revision},
            "files": [{"path": "LICENSE-MIT", "sha256": collector.sha256(notice),
                "url": f"https://raw.githubusercontent.com/example/dep/{revision}/LICENSE-MIT"}],
        }
        manifest = self.base / "supplements.json"
        dump(manifest, {"schema_version": 1, "packages": [record]})
        self.args.supplement_manifest, self.args.supplement_root = manifest, root
        return record

    def test_collects_only_observed_packages_and_verbatim_notice_bytes(self):
        report = collector.collect(self.args)
        self.assertEqual(report["status"], "notice_files_captured")
        self.assertFalse(report["complete_license_or_compatibility_claim"])
        self.assertEqual({p["name"] for p in report["packages"]}, {"rustui", "dep"})
        texts = set()
        for package in report["packages"]:
            for notice in package["notices"]:
                copied = self.out / notice["path"]
                self.assertEqual(collector.sha256(copied), notice["sha256"])
                texts.add(copied.read_text())
        self.assertEqual(texts, {"verbatim copying text\n", "verbatim notice\n", "RustUI root license\n"})

    def test_failed_final_build_rejected_without_touching_output(self):
        with self.build.open("a") as stream:
            stream.write(json.dumps({"reason": "build-finished", "success": False}) + "\n")
        self.out.mkdir()
        keep = self.out / "keep"
        keep.write_text("old")
        with self.assertRaises(ValueError):
            collector.collect(self.args)
        self.assertEqual(keep.read_text(), "old")

    def test_existing_unrelated_output_is_never_replaced(self):
        self.out.mkdir()
        (self.out / "keep").write_text("old")
        with self.assertRaisesRegex(ValueError, "new directory"):
            collector.collect(self.args)
        self.assertEqual((self.out / "keep").read_text(), "old")

    def test_missing_notice_reports_gap_without_a_completeness_claim(self):
        self.remove_notices()
        report = collector.collect(self.args)
        self.assertEqual(len(report["unresolved_gaps"]), 1)
        self.assertEqual(report["unresolved_gaps"][0]["package_id"], self.dep_id)
        self.assertFalse(report["complete_license_or_compatibility_claim"])

    def test_path_patched_dependency_does_not_inherit_workspace_license(self):
        self.remove_notices()
        self.packages[1]["source"] = None
        self.save_metadata()
        report = collector.collect(self.args)
        dep = next(p for p in report["packages"] if p["package_id"] == self.dep_id)
        self.assertFalse(dep["workspace_member"])
        self.assertEqual(dep["notices"], [])

    def test_explicit_license_file_is_honored_and_source_helpers_are_excluded(self):
        (self.dep / "license.rs").write_text("fn helper() {}")
        explicit = self.dep / "custom.txt"
        explicit.write_text("explicit license text")
        self.packages[1]["license_file"] = "custom.txt"
        self.save_metadata()
        report = collector.collect(self.args)
        dep = next(p for p in report["packages"] if p["package_id"] == self.dep_id)
        names = {Path(n["source"]).name for n in dep["notices"]}
        self.assertIn("custom.txt", names)
        self.assertNotIn("license.rs", names)

    def test_in_root_parent_symlink_is_rejected_before_resolution(self):
        alias = self.repo / "alias"
        alias.symlink_to(self.dep, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "symlink"):
            collector.safe_path(alias / "Cargo.toml", [self.repo])

    def test_symlink_notice_is_rejected(self):
        (self.dep / "LICENSE-link").symlink_to(self.dep / "COPYING")
        with self.assertRaisesRegex(ValueError, "symlink"):
            collector.collect(self.args)

    def test_output_cannot_overlap_source(self):
        self.args.output = self.dep / "generated"
        with self.assertRaisesRegex(ValueError, "overlaps"):
            collector.collect(self.args)
        self.assertFalse(self.args.output.exists())

    def test_output_cannot_overlap_unbuilt_package_source(self):
        unused = self.base / "unused-package"
        unused.mkdir()
        (unused / "Cargo.toml").write_text("[package]\n")
        self.packages[2]["manifest_path"] = str(unused / "Cargo.toml")
        self.save_metadata()
        self.args.output = unused / "generated"
        with self.assertRaisesRegex(ValueError, "overlaps"):
            collector.collect(self.args)
        self.assertFalse(self.args.output.exists())

    def test_exact_identity_supplement_is_collected(self):
        self.remove_notices()
        self.add_supplement()
        report = collector.collect(self.args)
        dep = next(p for p in report["packages"] if p["package_id"] == self.dep_id)
        self.assertEqual(len(dep["notices"]), 1)
        self.assertEqual((self.out / dep["notices"][0]["path"]).read_text(), "supplement verbatim\n")
        self.assertIn("supplement", dep["notices"][0])

    def test_supplement_traversal_and_hash_mismatch_are_rejected(self):
        record = self.add_supplement()
        manifest = self.args.supplement_manifest
        record["files"][0]["path"] = "../outside"
        dump(manifest, {"schema_version": 1, "packages": [record]})
        with self.assertRaisesRegex(ValueError, "supplement path"):
            collector.collect(self.args)
        record["files"][0]["path"] = "LICENSE-MIT"
        record["files"][0]["sha256"] = "0" * 64
        dump(manifest, {"schema_version": 1, "packages": [record]})
        with self.assertRaisesRegex(ValueError, "hash mismatch"):
            collector.collect(self.args)

    def test_supplement_package_identity_must_match_metadata(self):
        record = self.add_supplement()
        record["manifest_sha256"] = "0" * 64
        dump(self.args.supplement_manifest, {"schema_version": 1, "packages": [record]})
        with self.assertRaisesRegex(ValueError, "identity mismatch"):
            collector.collect(self.args)

    def test_supplement_rejects_unpinned_source_url(self):
        record = self.add_supplement()
        record["files"][0]["url"] = "https://example.test/latest/LICENSE"
        dump(self.args.supplement_manifest, {"schema_version": 1, "packages": [record]})
        with self.assertRaisesRegex(ValueError, "revision-pinned"):
            collector.collect(self.args)

    def test_malformed_supplement_source_is_rejected_cleanly(self):
        record = self.add_supplement()
        record["source"] = []
        dump(self.args.supplement_manifest, {"schema_version": 1, "packages": [record]})
        with self.assertRaisesRegex(ValueError, "exact Cargo source"):
            collector.collect(self.args)

    def test_changed_metadata_is_not_bound_to_old_collection(self):
        original_copy = collector.shutil.copyfile
        def mutate(source, destination):
            result = original_copy(source, destination)
            with self.metadata.open("a") as stream:
                stream.write("\n")
            return result
        with patch.object(collector.shutil, "copyfile", side_effect=mutate):
            with self.assertRaisesRegex(ValueError, "inputs changed"):
                collector.collect(self.args)
        self.assertFalse(self.out.exists())

    def test_copy_detects_changed_source(self):
        original_copy = collector.shutil.copyfile
        def mutate(source, destination):
            result = original_copy(source, destination)
            Path(source).write_text("changed during copy")
            return result
        with patch.object(collector.shutil, "copyfile", side_effect=mutate):
            with self.assertRaisesRegex(ValueError, "changed while copying"):
                collector.collect(self.args)
        self.assertFalse(self.out.exists())

    def test_unknown_artifact_package_is_rejected(self):
        with self.build.open("a") as stream:
            stream.write(json.dumps({"reason": "compiler-artifact", "package_id": "unknown"}) + "\n")
        with self.assertRaisesRegex(ValueError, "absent from metadata"):
            collector.collect(self.args)

    def test_public_cli_collects_build_json(self):
        result = subprocess.run([
            sys.executable, str(SCRIPT), "--repo-root", str(self.repo),
            "--metadata", str(self.metadata), "--build-json", str(self.build),
            "--output", str(self.out),
        ], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(json.loads((self.out / "report.json").read_text())["packages"]), 2)


if __name__ == "__main__":
    unittest.main()
