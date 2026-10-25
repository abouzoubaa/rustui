import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "package_release_notices.py"
FONT_NAMES = ("FiraMono-LICENSE", "Inter-LICENSE", "NotoSans-LICENSE")


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
            }],
            "unresolved_gaps": [{"package_id": f"path+file://{self.root}/unknown/Cargo.toml#gap@1", "name": "gap",
                                 "version": "1", "reason": "no captured notice files"}],
        }
        (self.collection / "report.json").write_text(json.dumps(report), encoding="utf-8")
        (self.collection / "REPORT.txt").write_text("fixture report\n", encoding="utf-8")
        self.fonts = self.root / "fonts"
        self.fonts.mkdir()
        for name in FONT_NAMES:
            (self.fonts / name).write_text(f"verbatim {name}\n", encoding="utf-8")
        self.destination = self.root / "bundle" / "licenses" / "release"

    def tearDown(self):
        self.temp.cleanup()

    def run_package(self, collection=None, destination=None):
        return subprocess.run([sys.executable, str(SCRIPT), "--collection", str(collection or self.collection),
                               "--font-source", str(self.fonts), "--repo-root", str(self.root),
                               "--destination", str(destination or self.destination)],
                              text=True, capture_output=True)

    def test_packages_and_repackages_verified_data(self):
        for _ in range(2):
            result = self.run_package()
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual((self.destination / "dependency-notices/notices/LICENSE").read_bytes(), self.notice_bytes)
            packaged_report = json.loads((self.destination / "dependency-notices/report.json").read_text())
            self.assertEqual(packaged_report["inputs"]["metadata_sha256"], "1" * 64)
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
