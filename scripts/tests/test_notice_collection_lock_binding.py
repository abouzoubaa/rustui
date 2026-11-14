import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
spec = __import__("importlib.util").util.spec_from_file_location(
    "notice_freshness", SCRIPTS / "check_dependency_notices_freshness.py"
)
checker = __import__("importlib.util").util.module_from_spec(spec)
spec.loader.exec_module(checker)

COLLECTOR = SCRIPTS / "collect_dependency_notices.py"
PACKAGER = SCRIPTS / "package_release_notices.py"
FONT_NAMES = ("FiraMono-LICENSE", "Inter-LICENSE", "NotoSans-LICENSE")


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


class NoticeCollectionLockBindingTests(unittest.TestCase):
    def test_old_collection_cannot_be_repackaged_or_pass_freshness_after_lock_change(self):
        with tempfile.TemporaryDirectory(prefix="rustui-notice-lock-binding-") as tmp:
            work = Path(tmp)
            repo = work / "repo"
            package_root = repo / "fixture"
            package_root.mkdir(parents=True)
            (repo / "LICENSE").write_text("repository notice\n", encoding="utf-8")
            manifest = package_root / "Cargo.toml"
            manifest.write_text(
                '[package]\nname = "fixture"\nversion = "1.0.0"\nedition = "2024"\n',
                encoding="utf-8",
            )
            (package_root / "LICENSE").write_text("fixture notice\n", encoding="utf-8")
            cargo_lock = repo / "Cargo.lock"
            cargo_lock.write_text(
                'version = 4\n\n[[package]]\nname = "fixture"\nversion = "1.0.0"\n',
                encoding="utf-8",
            )
            supplement_dir = repo / "docs/release/notice-supplements"
            supplement_dir.mkdir(parents=True)
            supplement_manifest = supplement_dir / "manifest.json"
            supplement_manifest.write_text(
                json.dumps({"schema_version": 1, "packages": []}), encoding="utf-8"
            )
            fonts = repo / "crates/rustui/assets/fonts"
            fonts.mkdir(parents=True)
            for name in FONT_NAMES:
                (fonts / name).write_text(f"fixture {name}\n", encoding="utf-8")

            package_id = f"path+file://{manifest}#fixture@1.0.0"
            metadata = work / "metadata.json"
            metadata.write_text(json.dumps({
                "packages": [{
                    "id": package_id,
                    "name": "fixture",
                    "version": "1.0.0",
                    "source": None,
                    "manifest_path": str(manifest),
                    "license": "MIT",
                }],
                "workspace_members": [package_id],
            }), encoding="utf-8")
            build = work / "build.jsonl"
            build.write_text(
                json.dumps({"reason": "compiler-artifact", "package_id": package_id}) + "\n"
                + json.dumps({"reason": "build-finished", "success": True}) + "\n",
                encoding="utf-8",
            )
            collection = work / "old-collection"
            collect = subprocess.run([
                sys.executable, str(COLLECTOR),
                "--metadata", str(metadata),
                "--build-json", str(build),
                "--repo-root", str(repo),
                "--supplement-manifest", str(supplement_manifest),
                "--supplement-root", str(supplement_dir),
                "--output", str(collection),
                "--fail-on-unresolved-gaps",
            ], text=True, capture_output=True)
            self.assertEqual(collect.returncode, 0, collect.stderr)
            old_lock_hash = sha256(cargo_lock)
            old_report = json.loads((collection / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(old_report["inputs"]["cargo_lock_sha256"], old_lock_hash)
            self.assertEqual(
                old_report["inputs"]["supplement_manifest_path"],
                "docs/release/notice-supplements/manifest.json",
            )
            self.assertEqual(
                old_report["inputs"]["supplement_manifest_sha256"], sha256(supplement_manifest)
            )

            with cargo_lock.open("a", encoding="utf-8") as lock:
                lock.write(
                    '\n[[package]]\nname = "new-dependency"\nversion = "2.0.0"\n'
                )

            snapshot = repo / "docs/release/current-notices"
            package = subprocess.run([
                sys.executable, str(PACKAGER),
                "--collection", str(collection),
                "--font-source", str(fonts),
                "--repo-root", str(repo),
                "--destination", str(snapshot),
                "--require-no-gaps",
            ], text=True, capture_output=True)
            package_rejected = package.returncode != 0

            # Model the stale snapshot produced by the old packager: pair the old
            # collection with a binding freshly computed from today's lockfile.
            if not snapshot.exists():
                snapshot.mkdir(parents=True)
                shutil.copytree(collection, snapshot / "dependency-notices")
                shutil.copytree(fonts, snapshot / "fonts")
            (snapshot / "lockfile.json").write_text(json.dumps({
                "schema_version": 1,
                "cargo_lock_sha256": sha256(cargo_lock),
                "supplement_manifest_sha256": sha256(supplement_manifest),
            }), encoding="utf-8")
            try:
                checker.check(repo)
            except ValueError as error:
                freshness_rejected = True
                freshness_error = str(error)
            else:
                freshness_rejected = False
                freshness_error = ""

            self.assertNotEqual(old_lock_hash, sha256(cargo_lock))
            self.assertTrue(
                package_rejected and freshness_rejected,
                "old collection must be rejected both at packaging and freshness: "
                f"package_rejected={package_rejected}, stdout={package.stdout!r}, "
                f"stderr={package.stderr!r}; freshness_rejected={freshness_rejected}, "
                f"collection_lock={old_report.get('inputs', {}).get('cargo_lock_sha256')!r}, "
                f"current_lock={sha256(cargo_lock)}, checker_error={freshness_error!r}",
            )


if __name__ == "__main__":
    unittest.main()

