import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "package_release_notices.py"
FONT_NAMES = ("FiraMono-LICENSE", "Inter-LICENSE", "NotoSans-LICENSE")
REPORT_TEXT = (
    "Dependency notice collection\n"
    "This report is not a complete license or compatibility determination.\n"
    "Observed packages: 2; unresolved notice gaps: 1.\n"
    "See report.json for package IDs, sources, hashes and gaps.\n"
)


class PackageReleaseNoticesTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="rustui-release-notices-test-")
        self.root = Path(self.temp.name)
        self.collection = self.root / "collection"
        self.collection.mkdir()
        (self.collection / "notices").mkdir()
        self.notice_bytes = b"verbatim notice fixture\n"
        (self.collection / "notices" / "LICENSE").write_bytes(self.notice_bytes)
        entry = {"path": "notices/LICENSE", "sha256": hashlib.sha256(self.notice_bytes).hexdigest()}
        report = {
            "schema_version": 1, "status": "gaps",
            "complete_license_or_compatibility_claim": False,
            "inputs": {"metadata_sha256": "1" * 64, "build_json_sha256": "2" * 64,
                       "supplement_manifest_sha256": "3" * 64},
            "packages": [{
                "name": "fixture", "version": "1.0.0", "source": None,
                "package_id": f"path+file://{self.root}/repo/Cargo.toml#fixture@1.0.0",
                "manifest_path": str(self.root / "repo/Cargo.toml"),
                "manifest_sha256": "4" * 64, "source_root": str(self.root / "repo"),
                "notices": [{**entry, "source": str(self.root / "repo/LICENSE")}],
                "notice_review_notes": ["fixture review note"],
            }, {
                "name": "gap", "version": "1", "source": None,
                "package_id": f"path+file://{self.root}/unknown/Cargo.toml#gap@1",
                "manifest_path": str(self.root / "unknown/Cargo.toml"),
                "notices": [],
            }],
            "unresolved_gaps": [{"package_id": f"path+file://{self.root}/unknown/Cargo.toml#gap@1", "name": "gap",
                                 "version": "1", "reason": "no captured notice files"}],
        }
        (self.collection / "report.json").write_text(json.dumps(report), encoding="utf-8")
        (self.collection / "REPORT.txt").write_text(REPORT_TEXT, encoding="utf-8")
        self.fonts = self.root / "fonts"
        self.fonts.mkdir()
        for name in FONT_NAMES:
            (self.fonts / name).write_text(f"verbatim {name}\n", encoding="utf-8")
        self.destination = self.root / "bundle" / "licenses" / "release"

    def tearDown(self):
        self.temp.cleanup()

    def run_package(self, collection=None, destination=None, repo_root=None, font_source=None,
                    require_no_gaps=False, validate_only=False):
        command = [sys.executable, str(SCRIPT), "--collection",
                   str(collection if collection is not None else self.collection),
                   "--font-source", str(font_source if font_source is not None else self.fonts),
                   "--repo-root", str(repo_root if repo_root is not None else self.root),
                   "--destination", str(destination if destination is not None else self.destination)]
        if require_no_gaps:
            command.append("--require-no-gaps")
        if validate_only:
            command.append("--validate-only")
        return subprocess.run(command, text=True, capture_output=True)

    def test_rejects_inconsistent_gap_reports_without_replacing_bundle(self):
        initial = self.run_package()
        self.assertEqual(initial.returncode, 0, initial.stderr)
        packaged = self.destination / "dependency-notices/report.json"
        before = packaged.read_bytes()
        report_path = self.collection / "report.json"
        original = json.loads(report_path.read_text())

        def missing_field(report):
            del report["unresolved_gaps"]

        def hidden_gap(report):
            report["unresolved_gaps"] = []

        def duplicate_gap(report):
            report["unresolved_gaps"].append(dict(report["unresolved_gaps"][0]))

        def wrong_package(report):
            report["unresolved_gaps"][0]["package_id"] = report["packages"][0]["package_id"]

        def wrong_name(report):
            report["unresolved_gaps"][0]["name"] = "other"

        def false_status(report):
            report["status"] = "notice_files_captured"

        def duplicate_package(report):
            report["packages"].append(dict(report["packages"][0]))

        def missing_reason(report):
            report["unresolved_gaps"][0]["reason"] = " "

        for mutate in (missing_field, hidden_gap, duplicate_gap, wrong_package, wrong_name,
                       false_status, duplicate_package, missing_reason):
            with self.subTest(case=mutate.__name__):
                report = json.loads(json.dumps(original))
                mutate(report)
                report_path.write_text(json.dumps(report), encoding="utf-8")
                result = self.run_package()
                self.assertNotEqual(result.returncode, 0, result.stdout)
                self.assertEqual(packaged.read_bytes(), before)

    def test_rejects_misleading_report_text_without_replacing_bundle(self):
        initial = self.run_package()
        self.assertEqual(initial.returncode, 0, initial.stderr)
        packaged = self.destination / "dependency-notices/REPORT.txt"
        before = packaged.read_bytes()
        (self.collection / "REPORT.txt").write_text(
            REPORT_TEXT.replace("unresolved notice gaps: 1", "unresolved notice gaps: 0"),
            encoding="utf-8",
        )
        result = self.run_package()
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertEqual(packaged.read_bytes(), before)

    def test_accepts_complete_notice_inventory_without_gaps(self):
        second_notice = self.collection / "notices/SECOND-LICENSE"
        second_notice.write_bytes(b"second notice\n")
        report_path = self.collection / "report.json"
        report = json.loads(report_path.read_text())
        report["packages"][1]["notices"] = [{
            "path": "notices/SECOND-LICENSE",
            "source": str(second_notice),
            "sha256": hashlib.sha256(second_notice.read_bytes()).hexdigest(),
        }]
        report["unresolved_gaps"] = []
        report["status"] = "notice_files_captured"
        report_path.write_text(json.dumps(report), encoding="utf-8")
        (self.collection / "REPORT.txt").write_text(
            REPORT_TEXT.replace("unresolved notice gaps: 1", "unresolved notice gaps: 0"),
            encoding="utf-8",
        )
        preflight = self.run_package(require_no_gaps=True, validate_only=True)
        self.assertEqual(preflight.returncode, 0, preflight.stderr)
        self.assertFalse(self.destination.exists())

        result = self.run_package(require_no_gaps=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        packaged = json.loads((self.destination / "dependency-notices/report.json").read_text())
        self.assertEqual(packaged["unresolved_gaps"], [])

    def test_require_no_gaps_rejects_report_without_replacing_bundle(self):
        initial = self.run_package()
        self.assertEqual(initial.returncode, 0, initial.stderr)
        report_path = self.destination / "dependency-notices/report.json"
        original_report = report_path.read_bytes()

        result = self.run_package(require_no_gaps=True)

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("unresolved package gaps", result.stderr)
        self.assertEqual(report_path.read_bytes(), original_report)

    def test_validate_only_rejects_gaps_without_creating_destination(self):
        destination = self.root / "not-created"
        result = self.run_package(
            destination=destination,
            require_no_gaps=True,
            validate_only=True,
        )

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("unresolved package gaps", result.stderr)
        self.assertFalse(destination.exists())

    def test_packages_and_repackages_verified_data(self):
        for _ in range(2):
            result = self.run_package()
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual((self.destination / "dependency-notices/notices/LICENSE").read_bytes(), self.notice_bytes)
            packaged_report = json.loads((self.destination / "dependency-notices/report.json").read_text())
            self.assertNotIn("metadata_sha256", packaged_report["inputs"])
            self.assertNotIn("build_json_sha256", packaged_report["inputs"])
            self.assertEqual(packaged_report["inputs"]["supplement_manifest_sha256"], "3" * 64)
            source_report = json.loads((self.collection / "report.json").read_text())
            self.assertEqual(source_report["inputs"]["metadata_sha256"], "1" * 64)
            self.assertEqual(source_report["inputs"]["build_json_sha256"], "2" * 64)
            self.assertEqual(packaged_report["packages"][0]["manifest_sha256"], "4" * 64)
            self.assertEqual(packaged_report["packages"][0]["notice_review_notes"], ["fixture review note"])
            self.assertEqual(packaged_report["unresolved_gaps"][0]["name"], "gap")
            self.assertEqual(packaged_report["unresolved_gaps"][0]["package_id"],
                             "path+repository:unknown/Cargo.toml#gap@1")
            self.assertNotIn(str(self.root), json.dumps(packaged_report))
            self.assertEqual(packaged_report["packages"][0]["package_id"],
                             "path+repository:repo/Cargo.toml#fixture@1.0.0")
            for name in FONT_NAMES:
                self.assertEqual((self.destination / "fonts" / name).read_text(), f"verbatim {name}\n")

    def test_equivalent_collections_are_stable_across_roots_and_raw_input_hashes(self):
        first_result = self.run_package()
        self.assertEqual(first_result.returncode, 0, first_result.stderr)

        other_root = self.root / "other-checkout"
        other_collection = other_root / "collection"
        other_notices = other_collection / "notices"
        other_notices.mkdir(parents=True)
        (other_notices / "LICENSE").write_bytes(self.notice_bytes)
        other_report = json.loads((self.collection / "report.json").read_text())
        other_report["inputs"]["metadata_sha256"] = "a" * 64
        other_report["inputs"]["build_json_sha256"] = "b" * 64
        package = other_report["packages"][0]
        package["package_id"] = f"path+file://{other_root}/repo/Cargo.toml#fixture@1.0.0"
        package["manifest_path"] = str(other_root / "repo/Cargo.toml")
        package["source_root"] = str(other_root / "repo")
        package["notices"][0]["source"] = str(other_root / "repo/LICENSE")
        other_report["unresolved_gaps"][0]["package_id"] = (
            f"path+file://{other_root}/unknown/Cargo.toml#gap@1"
        )
        other_report["packages"][1]["package_id"] = other_report["unresolved_gaps"][0]["package_id"]
        other_report["packages"][1]["manifest_path"] = str(other_root / "unknown/Cargo.toml")
        (other_collection / "report.json").write_text(json.dumps(other_report), encoding="utf-8")
        (other_collection / "REPORT.txt").write_bytes((self.collection / "REPORT.txt").read_bytes())

        other_fonts = other_root / "fonts"
        other_fonts.mkdir()
        for name in FONT_NAMES:
            (other_fonts / name).write_text(f"verbatim {name}\n", encoding="utf-8")
        other_destination = other_root / "bundle" / "licenses" / "release"
        result = self.run_package(other_collection, other_destination, other_root, other_fonts)
        self.assertEqual(result.returncode, 0, result.stderr)

        first_dependency = self.destination / "dependency-notices"
        other_dependency = other_destination / "dependency-notices"
        self.assertEqual((first_dependency / "report.json").read_bytes(),
                         (other_dependency / "report.json").read_bytes())
        self.assertEqual((first_dependency / "REPORT.txt").read_text(), REPORT_TEXT)
        self.assertEqual((first_dependency / "REPORT.txt").read_bytes(),
                         (other_dependency / "REPORT.txt").read_bytes())
        self.assertEqual((first_dependency / "notices/LICENSE").read_bytes(),
                         (other_dependency / "notices/LICENSE").read_bytes())
        for packaged in (first_dependency / "report.json", first_dependency / "REPORT.txt",
                         other_dependency / "report.json", other_dependency / "REPORT.txt"):
            contents = packaged.read_text(encoding="utf-8")
            self.assertNotIn(str(self.root), contents)
            self.assertNotIn(str(other_root), contents)

        # Raw machine-local hashes remain available only in the external collections.
        self.assertEqual(json.loads((self.collection / "report.json").read_text())
                         ["inputs"]["metadata_sha256"], "1" * 64)
        self.assertEqual(json.loads((other_collection / "report.json").read_text())
                         ["inputs"]["metadata_sha256"], "a" * 64)

    def test_rejects_tampered_notice_and_preserves_previous_package(self):
        self.assertEqual(self.run_package().returncode, 0)
        original = (self.destination / "dependency-notices/notices/LICENSE").read_bytes()
        (self.collection / "notices/LICENSE").write_bytes(b"tampered\n")
        self.assertNotEqual(self.run_package().returncode, 0)
        self.assertEqual((self.destination / "dependency-notices/notices/LICENSE").read_bytes(), original)

    def test_rejects_symlink_notice_input(self):
        target = self.root / "outside-notice"
        target.write_bytes(self.notice_bytes)
        (self.collection / "notices" / "LICENSE").unlink()
        (self.collection / "notices" / "LICENSE").symlink_to(target)
        self.assertNotEqual(self.run_package().returncode, 0)

    def test_rejects_destination_overlap_through_symlinked_ancestor(self):
        alias = self.root / "alias"
        alias.symlink_to(self.root, target_is_directory=True)
        for source in (self.collection, self.fonts):
            with self.subTest(source=source.name):
                destination = alias / source.name / "nested" / "release"
                result = self.run_package(destination=destination)
                self.assertNotEqual(result.returncode, 0, result.stdout)
                self.assertFalse((source / "nested").exists())
        safe = self.run_package(destination=alias / "safe" / "release")
        self.assertEqual(safe.returncode, 0, safe.stderr)

    def test_rejects_traversal_and_unknown_destination_content(self):
        report_path = self.collection / "report.json"
        report = json.loads(report_path.read_text())
        report["packages"][0]["notices"][0]["path"] = "notices/../escape"
        report_path.write_text(json.dumps(report))
        self.assertNotEqual(self.run_package().returncode, 0)
        report["packages"][0]["notices"][0]["path"] = "notices/LICENSE"
        report_path.write_text(json.dumps(report))
        self.assertEqual(self.run_package().returncode, 0)
        (self.destination / "unrelated.txt").write_text("leave me\n")
        self.assertNotEqual(self.run_package().returncode, 0)


if __name__ == "__main__":
    unittest.main()
