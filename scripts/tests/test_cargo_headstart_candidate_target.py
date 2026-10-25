#!/usr/bin/env python3
"""Verify cargo-headstart keeps candidate builds out of the accepted cache."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


WRAPPER = Path(__file__).resolve().parents[1] / "cargo-headstart"


class CandidateTargetTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.workspace = self.root / "workspace"
        self.wrapper_dir = self.workspace / "scripts"
        self.wrapper_dir.mkdir(parents=True)
        shutil.copy2(WRAPPER, self.wrapper_dir / "cargo-headstart")
        self.tools = self.root / "headstart"
        host = "x86_64-unknown-linux-gnu"
        (self.tools / "cargo/target/release").mkdir(parents=True)
        stage1 = self.tools / f"rustc/build/{host}/stage1/bin"
        stage1.mkdir(parents=True)
        cargo = self.tools / "cargo/target/release/cargo"
        cargo.write_text('#!/bin/sh\nprintf "%s\\n" "$CARGO_TARGET_DIR"\n')
        cargo.chmod(0o755)
        for name in ("rustc", "rustdoc"):
            binary = stage1 / name
            binary.write_text("#!/bin/sh\nexit 0\n")
            binary.chmod(0o755)
        rustc = self.root / "rustc"
        rustc.write_text('#!/bin/sh\nprintf "host: x86_64-unknown-linux-gnu\\n"\n')
        rustc.chmod(0o755)
        self.path = f"{self.root}:{os.environ['PATH']}"

    def tearDown(self):
        self.temp.cleanup()

    def invoke(self, cwd, target=None):
        env = {**os.environ, "PATH": self.path, "RUSTUI_HEADSTART_ROOT": str(self.tools)}
        if target is not None:
            env["CARGO_TARGET_DIR"] = str(target)
        else:
            env.pop("CARGO_TARGET_DIR", None)
        return subprocess.run([str(self.wrapper_dir / "cargo-headstart"), "check"],
                              cwd=cwd, env=env, capture_output=True, text=True)

    def test_normal_workspace_default_is_unchanged(self):
        result = self.invoke(self.workspace)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), str(self.workspace / "target/headstart"))

    def test_candidate_workspace_defaults_to_candidate_target(self):
        source = self.workspace / "target/headstart/candidates/demo-20261020/candidate/source"
        source.mkdir(parents=True)
        result = self.invoke(source)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), str((source.parents[1] / "target").resolve()))

    def test_candidate_accepts_manifest_safe_identifier_characters(self):
        candidate = "Release_2026.10"
        source = self.workspace / "target/headstart/candidates" / candidate / "source"
        source.mkdir(parents=True)
        result = self.invoke(source)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), str((source.parent / "target").resolve()))

    def test_candidate_rejects_identifier_with_double_dot(self):
        source = self.workspace / "target/headstart/candidates/a..b/source"
        source.mkdir(parents=True)
        result = self.invoke(source)
        self.assertEqual(result.returncode, 2)
        self.assertIn("Invalid candidate workspace ID", result.stderr)

    def test_candidate_rejects_accepted_workspace_target_override(self):
        source = self.workspace / "target/headstart/candidates/demo-20261020/source"
        source.mkdir(parents=True)
        accepted_target = self.workspace / "target/headstart"
        result = self.invoke(source, accepted_target)
        self.assertEqual(result.returncode, 2)
        self.assertIn("Candidate builds require CARGO_TARGET_DIR", result.stderr)

    def test_candidate_rejects_symlink_to_accepted_workspace_target(self):
        source = self.workspace / "target/headstart/candidates/demo-20261020/source"
        candidate_root = source.parent
        source.mkdir(parents=True)
        accepted_target = self.workspace / "target/headstart"
        (candidate_root / "cache-alias").symlink_to(accepted_target, target_is_directory=True)
        result = self.invoke(source, candidate_root / "cache-alias")
        self.assertEqual(result.returncode, 2)
        self.assertIn("Candidate builds require CARGO_TARGET_DIR", result.stderr)

    def test_candidate_rejects_dangling_target_symlink(self):
        source = self.workspace / "target/headstart/candidates/demo-20261020/source"
        candidate_root = source.parent
        source.mkdir(parents=True)
        dangling_target = candidate_root / "dangling-target"
        dangling_target.symlink_to(self.root / "outside-target", target_is_directory=True)
        result = self.invoke(source, dangling_target)
        self.assertEqual(result.returncode, 2)
        self.assertIn("cannot be a dangling", result.stderr)

    def test_candidate_allows_explicit_candidate_local_target(self):
        source = self.workspace / "target/headstart/candidates/demo-20261020/source"
        source.mkdir(parents=True)
        local_target = source.parents[0] / "baseline/target"
        local_target.mkdir(parents=True)
        result = self.invoke(source, local_target)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), str(local_target))


if __name__ == "__main__":
    unittest.main()
