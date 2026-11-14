import hashlib
import json
import subprocess
import sys as python_executable
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "scripts"))

from approved_single_source_patch import run  # noqa: E402


PATCHES = (
    ("patches/candidate-kit-fast/gpui-base-owned-gpui-pre-optional-v1.json", "crates/rustui-gpui-base/Cargo.toml"),
    ("patches/candidate-kit-fast/gpui-component-owned-gpui-pre-optional-v1.json", "crates/rustui-gpui-component/Cargo.toml"),
    ("patches/candidate-kit-fast/gpui-kit-assets-owned-gpui-pre-optional-v1.json", "crates/rustui-gpui-kit-assets/Cargo.toml"),
)


class FastManifestPatchTests(unittest.TestCase):
    def test_each_manifest_patch_applies_and_reverses_exactly(self):
        for manifest_relative, source_relative in PATCHES:
            with self.subTest(source=source_relative), tempfile.TemporaryDirectory() as temporary:
                fixture = Path(temporary)
                source = fixture / source_relative
                source.parent.mkdir(parents=True, exist_ok=True)
                manifest_path = REPO_ROOT / manifest_relative
                manifest_hash = hashlib.sha256(manifest_path.read_bytes()).hexdigest()
                source.write_bytes((REPO_ROOT / source_relative).read_bytes())
                if source_relative == "crates/rustui-gpui-base/Cargo.toml":
                    # These follow-up manifest patches share this file; unwind
                    # them first so this isolated check sees its exact postimage.
                    for later in (
                        "gpui-base-fast-feature-wiring-v1.json",
                        "gpui-base-fast-test-support-path-v1.json",
                        "gpui-base-fast-test-support-v1.json",
                        "gpui-base-fast-platform-test-dep-v1.json",
                    ):
                        later_path = REPO_ROOT / "patches/candidate-kit-fast" / later
                        if later_path.exists():
                            subprocess.run(
                                [
                                    python_executable.executable,
                                    str(REPO_ROOT / "scripts/apply_f0_fast_candidate_patch.py"),
                                    str(later_path),
                                    "--root",
                                    str(fixture),
                                    "--reverse",
                                ],
                                check=True,
                                capture_output=True,
                                text=True,
                            )
                run(fixture, REPO_ROOT, Path(manifest_relative), manifest_hash, "--reverse")
                original = source.read_bytes()
                run(fixture, REPO_ROOT, Path(manifest_relative), manifest_hash, "--check")
                run(fixture, REPO_ROOT, Path(manifest_relative), manifest_hash, "--apply")
                self.assertIn(b"optional = true", source.read_bytes())
                run(fixture, REPO_ROOT, Path(manifest_relative), manifest_hash, "--reverse")
                self.assertEqual(source.read_bytes(), original)


if __name__ == "__main__":
    unittest.main()
