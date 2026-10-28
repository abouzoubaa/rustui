"""Focused tests for the read-only paired candidate snapshot validator."""

from __future__ import annotations

import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import resource
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from typing import Any
from unittest import mock


SCRIPT = Path(__file__).resolve().parents[1] / "validate_candidate_snapshot.py"
SPEC = importlib.util.spec_from_file_location("validate_candidate_snapshot", SCRIPT)
VALIDATOR = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(VALIDATOR)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tree_hash(files: dict[str, str]) -> str:
    return hashlib.sha256(json.dumps(files, sort_keys=True).encode()).hexdigest()


class ValidateCandidateSnapshotTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.snapshots = {}
        for variant, source in (("baseline", "before"), ("candidate", "after")):
            source_root = self.root / variant / "source"
            (source_root / "apps").mkdir(parents=True)
            (source_root / "apps" / "input.rs").write_text("same\n", encoding="utf-8")
            package = source_root / "path-sources" / "gpui-pre-0.3.9" / "src"
            package.mkdir(parents=True)
            (package / "scene.rs").write_text(source + "\n", encoding="utf-8")
            files = {str(path.relative_to(source_root)): digest(path)
                     for path in sorted(source_root.rglob("*")) if path.is_file()}
            path_files = {path.removeprefix("path-sources/gpui-pre-0.3.9/"): value
                          for path, value in files.items() if path.startswith("path-sources/")}
            self.snapshots[variant] = {
                "source": str(source_root), "files": files,
                "path_tree_sha256": tree_hash(path_files),
            }
        self.manifest = {
            "schema_version": 1,
            "candidate_id": "candidate-1",
            "snapshots": self.snapshots,
        }
        self.manifest_path = self.root / "manifest.json"
        self.write_manifest()

    def write_manifest(self) -> None:
        self.manifest_path.write_text(json.dumps(self.manifest), encoding="utf-8")

    def invoke(self) -> tuple[subprocess.CompletedProcess[str], dict]:
        result = subprocess.run(
            [sys.executable, str(SCRIPT), str(self.manifest_path), "--root", str(self.root)],
            capture_output=True, text=True, check=False,
        )
        return result, json.loads(result.stdout)

    def test_inventory_entry_budget_accepts_limit_and_rejects_limit_plus_one(self) -> None:
        source = self.root / "entry-budget"
        source.mkdir()
        (source / "a").write_bytes(b"a")
        (source / "b").write_bytes(b"b")
        with mock.patch.object(VALIDATOR, "MAX_SNAPSHOT_ENTRIES", 2):
            self.assertEqual(len(VALIDATOR._inventory(source.resolve())), 2)
            (source / "c").write_bytes(b"c")
            with self.assertRaisesRegex(VALIDATOR.SnapshotError, "exceeds 2 entries"):
                VALIDATOR._inventory(source.resolve())

    def test_inventory_handles_depth_beyond_python_recursion_limit(self) -> None:
        depth = 1100
        deep_root = self.root / "deep-tree"
        deep_root.mkdir()
        original_soft, hard_limit = resource.getrlimit(resource.RLIMIT_NOFILE)
        required_limit = depth + 16
        if hard_limit != resource.RLIM_INFINITY and hard_limit < required_limit:
            self.skipTest("hard file descriptor limit is too low to construct the deep tree")
        working_limit = max(original_soft, required_limit)
        resource.setrlimit(resource.RLIMIT_NOFILE, (working_limit, hard_limit))
        descriptor_count = len(os.listdir("/dev/fd"))
        low_limit = max(32, descriptor_count + 8)
        if low_limit >= depth:
            self.skipTest("cannot set a descriptor limit far below the tree depth")
        if hard_limit != resource.RLIM_INFINITY and hard_limit < low_limit:
            self.skipTest("hard file descriptor limit is too low for the regression")
        root_fd = os.open(deep_root, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
        directory_fds = [root_fd]
        try:
            for _ in range(depth):
                os.mkdir("d", dir_fd=directory_fds[-1])
                directory_fds.append(os.open(
                    "d", os.O_RDONLY | getattr(os, "O_DIRECTORY", 0),
                    dir_fd=directory_fds[-1],
                ))
            file_fd = os.open("leaf", os.O_WRONLY | os.O_CREAT | os.O_EXCL,
                              0o600, dir_fd=directory_fds[-1])
            os.write(file_fd, b"x")
            os.close(file_fd)

            expected_path = "/".join(["d"] * depth + ["leaf"])
            for descriptor in reversed(directory_fds[1:]):
                os.close(descriptor)
            directory_fds = [root_fd]
            resource.setrlimit(resource.RLIMIT_NOFILE, (low_limit, hard_limit))
            try:
                self.assertEqual(
                    VALIDATOR._inventory_fd(root_fd),
                    {expected_path: hashlib.sha256(b"x").hexdigest()},
                )
            finally:
                resource.setrlimit(resource.RLIMIT_NOFILE, (working_limit, hard_limit))
        finally:
            for descriptor in reversed(directory_fds[1:]):
                try:
                    os.close(descriptor)
                except OSError:
                    pass
            directory_fds = [root_fd]
            for _ in range(depth):
                directory_fds.append(os.open(
                    "d", os.O_RDONLY | getattr(os, "O_DIRECTORY", 0),
                    dir_fd=directory_fds[-1],
                ))
            try:
                os.unlink("leaf", dir_fd=directory_fds[-1])
            except FileNotFoundError:
                pass
            for parent_fd in reversed(directory_fds[:-1]):
                os.rmdir("d", dir_fd=parent_fd)
            for descriptor in reversed(directory_fds):
                os.close(descriptor)
            resource.setrlimit(resource.RLIMIT_NOFILE, (original_soft, hard_limit))

    def test_inherited_candidate_root_uses_candidate_fd_and_checks_parent_identity(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            repository = Path(temporary)
            parent = repository / "target/headstart/candidates"
            candidate = parent / "candidate-1"
            candidate.mkdir(parents=True)
            repo_fd = os.open(repository, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
            parent_fd = os.open(parent, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
            candidate_fd = os.open(candidate, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
            try:
                values = {
                    "RUSTUI_P3_REPO_ROOT_FD": str(repo_fd),
                    "RUSTUI_P3_CANDIDATE_PARENT_FD": str(parent_fd),
                    "RUSTUI_P3_CANDIDATE_ROOT_FD": str(candidate_fd),
                    "RUSTUI_P3_CANDIDATE_ID": "candidate-1",
                }
                with mock.patch.dict(os.environ, values):
                    pinned = VALIDATOR._inherited_candidate_root(
                        Path("target/headstart/candidates/candidate-1")
                    )
                try:
                    self.assertEqual(pinned.identity(), (os.fstat(candidate_fd).st_dev,
                                                         os.fstat(candidate_fd).st_ino))
                finally:
                    pinned.close()
                wrong_fd = os.open(parent, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
                try:
                    values["RUSTUI_P3_CANDIDATE_ROOT_FD"] = str(wrong_fd)
                    with mock.patch.dict(os.environ, values), self.assertRaisesRegex(
                        VALIDATOR.SnapshotError, "does not match candidate parent and ID"
                    ):
                        VALIDATOR._inherited_candidate_root(
                            Path("target/headstart/candidates/candidate-1")
                        )
                finally:
                    os.close(wrong_fd)
            finally:
                os.close(repo_fd)
                os.close(parent_fd)
                os.close(candidate_fd)

    def test_inherited_candidate_root_rejects_manifest_candidate_id_mismatch(self) -> None:
        candidate_id = "candidate-1"
        candidate_root = self.root / "target/headstart/candidates" / candidate_id
        snapshots = {}
        for variant, content in (("baseline", "before\n"), ("candidate", "after\n")):
            source = candidate_root / variant / "source"
            package = source / "path-sources/pkg"
            package.mkdir(parents=True)
            (package / "lib.rs").write_text(content, encoding="utf-8")
            files = VALIDATOR._inventory(source.resolve())
            snapshots[variant] = {
                "source": f"{variant}/source",
                "files": files,
                "path_tree_sha256": tree_hash({
                    name.removeprefix("path-sources/pkg/"): value
                    for name, value in files.items()
                    if name.startswith("path-sources/pkg/")
                }),
            }
        manifest = {
            "schema_version": 1,
            "candidate_id": "candidate-2",
            "snapshots": snapshots,
        }
        (candidate_root / "manifest.json").write_text(json.dumps(manifest), encoding="utf-8")

        repo_fd = os.open(self.root, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
        parent = candidate_root.parent
        parent_fd = os.open(parent, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
        candidate_fd = os.open(candidate_root, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
        values = {
            "RUSTUI_P3_REPO_ROOT_FD": str(repo_fd),
            "RUSTUI_P3_CANDIDATE_PARENT_FD": str(parent_fd),
            "RUSTUI_P3_CANDIDATE_ROOT_FD": str(candidate_fd),
            "RUSTUI_P3_CANDIDATE_ID": candidate_id,
        }
        output = io.StringIO()
        try:
            with (mock.patch.dict(os.environ, values),
                  mock.patch.object(sys, "argv", [str(SCRIPT), "manifest.json", "--root",
                                                   f"target/headstart/candidates/{candidate_id}"]),
                  mock.patch.object(VALIDATOR.Path, "cwd", return_value=self.root),
                  redirect_stdout(output)):
                self.assertEqual(VALIDATOR.main(), 1)
        finally:
            os.close(repo_fd)
            os.close(parent_fd)
            os.close(candidate_fd)

        report = json.loads(output.getvalue())
        self.assertEqual(report["status"], "invalid")
        self.assertIn(
            "manifest candidate_id does not match inherited candidate identity",
            report["errors"],
        )

    def test_inherited_candidate_root_rejects_parent_outside_repository_candidates(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            repository = Path(temporary) / "repository"
            repository_candidates = repository / "target/headstart/candidates"
            repository_candidates.mkdir(parents=True)
            external_candidates = Path(temporary) / "external-candidates"
            candidate = external_candidates / "candidate-1"
            candidate.mkdir(parents=True)
            repo_fd = os.open(repository, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
            parent_fd = os.open(external_candidates, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
            candidate_fd = os.open(candidate, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
            try:
                values = {
                    "RUSTUI_P3_REPO_ROOT_FD": str(repo_fd),
                    "RUSTUI_P3_CANDIDATE_PARENT_FD": str(parent_fd),
                    "RUSTUI_P3_CANDIDATE_ROOT_FD": str(candidate_fd),
                    "RUSTUI_P3_CANDIDATE_ID": "candidate-1",
                }
                with mock.patch.dict(os.environ, values), self.assertRaisesRegex(
                    VALIDATOR.SnapshotError,
                    "does not match repository candidates directory",
                ):
                    VALIDATOR._inherited_candidate_root(
                        Path("target/headstart/candidates/candidate-1")
                    )
            finally:
                os.close(repo_fd)
                os.close(parent_fd)
                os.close(candidate_fd)

    def test_inventory_byte_budget_accepts_limit_and_rejects_before_reading_oversize(self) -> None:
        source = self.root / "byte-budget"
        source.mkdir()
        (source / "a").write_bytes(b"ab")
        (source / "b").write_bytes(b"cd")
        with mock.patch.object(VALIDATOR, "MAX_SNAPSHOT_BYTES", 4):
            self.assertEqual(len(VALIDATOR._inventory(source.resolve())), 2)
            (source / "a").write_bytes(b"abcde")
            (source / "b").unlink()
            with (mock.patch.object(VALIDATOR.os, "fdopen") as fdopen,
                  self.assertRaisesRegex(
                      VALIDATOR.SnapshotError,
                      "exceeds 4 regular-file bytes",
                  )):
                VALIDATOR._inventory(source.resolve())
            fdopen.assert_not_called()

    def test_inventory_byte_budget_rejects_growth_after_size_check(self) -> None:
        source = self.root / "growing-file"
        source.mkdir()
        file_path = source / "a"
        file_path.write_bytes(b"ab")
        real_fdopen = VALIDATOR.os.fdopen

        def append_before_read(fd: int, mode: str):
            with file_path.open("ab") as writer:
                writer.write(b"cde")
            return real_fdopen(fd, mode)

        with mock.patch.object(VALIDATOR, "MAX_SNAPSHOT_BYTES", 4):
            with (mock.patch.object(
                      VALIDATOR.os, "fdopen", side_effect=append_before_read
                  ) as fdopen,
                  self.assertRaisesRegex(
                      VALIDATOR.SnapshotError,
                      "exceeds 4 regular-file bytes",
                  )):
                VALIDATOR._inventory(source.resolve())
            fdopen.assert_called_once()

    def test_rejects_file_mutation_during_snapshot_hash(self) -> None:
        source = self.root / "mutating-file"
        source.mkdir()
        file_path = source / "a"
        chunk_size = 1024 * 1024
        file_path.write_bytes(b"a" * chunk_size + b"b" * chunk_size)
        real_fdopen = VALIDATOR.os.fdopen

        def mutate_after_first_read(fd: int, mode: str):
            stream = real_fdopen(fd, mode)

            class MutatingReader:
                reads = 0

                def __enter__(self):
                    return self

                def __exit__(self, *args: Any) -> Any:
                    return stream.__exit__(*args)

                def fileno(self) -> int:
                    return stream.fileno()

                def read(self, size: int = -1) -> bytes:
                    block = stream.read(size)
                    self.reads += 1
                    if self.reads == 1:
                        with file_path.open("r+b") as writer:
                            writer.write(b"x")
                            writer.seek(chunk_size)
                            writer.write(b"c")
                    return block

            return MutatingReader()

        with mock.patch.object(
            VALIDATOR.os, "fdopen", side_effect=mutate_after_first_read
        ):
            with self.assertRaisesRegex(VALIDATOR.SnapshotError, "changed while"):
                VALIDATOR._inventory(source.resolve())

    def test_rejects_directory_entry_added_after_enumeration(self) -> None:
        source = self.root / "changing-directory"
        source.mkdir()
        (source / "before").write_text("before\n", encoding="utf-8")
        added = source / "after"
        real_scandir = VALIDATOR.os.scandir

        class MutatingScandir:
            def __init__(self, directory_fd: int) -> None:
                self.directory_fd = directory_fd

            def __enter__(self):
                self.entries = real_scandir(self.directory_fd)
                self.entries.__enter__()
                return self

            def __exit__(self, *args: Any) -> Any:
                result = self.entries.__exit__(*args)
                added.write_text("after\n", encoding="utf-8")
                return result

            def __iter__(self):
                return iter(self.entries)

        def mutating_scandir(directory_fd: int) -> MutatingScandir:
            return MutatingScandir(directory_fd)

        with mock.patch.object(VALIDATOR.os, "scandir", side_effect=mutating_scandir):
            with self.assertRaisesRegex(VALIDATOR.SnapshotError, "directory changed while"):
                VALIDATOR._inventory(source.resolve())

    def test_rejects_snapshot_root_moved_after_descriptor_open(self) -> None:
        source = self.root / "baseline/source"
        source_parent = self.root / "baseline"
        detached = self.root / "detached-baseline"
        source_identity = (source.stat().st_dev, source.stat().st_ino)
        real_scandir = VALIDATOR.os.scandir
        moved = False

        class MovingScandir:
            def __init__(self, directory_fd: int) -> None:
                self.entries = real_scandir(directory_fd)
                self.directory_fd = directory_fd

            def __enter__(self):
                self.entries.__enter__()
                return self

            def __exit__(self, *args: Any) -> Any:
                nonlocal moved
                result = self.entries.__exit__(*args)
                info = os.fstat(self.directory_fd)
                if not moved and (info.st_dev, info.st_ino) == source_identity:
                    source_parent.rename(detached)
                    moved = True
                return result

            def __iter__(self):
                return iter(self.entries)

        with mock.patch.object(
            VALIDATOR.os, "scandir", side_effect=lambda fd: MovingScandir(fd)
        ):
            report = VALIDATOR.validate(self.manifest, self.root.resolve())

        self.assertTrue(moved)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("source directory identity changed" in error
                            for error in report["errors"]), report["errors"])

    def test_validates_paired_snapshots_and_reports_delta(self) -> None:
        result, report = self.invoke()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["status"], "valid")
        self.assertEqual(report["snapshot_files"], {"baseline": 2, "candidate": 2})
        self.assertEqual(report["source_delta_files"], ["path-sources/gpui-pre-0.3.9/src/scene.rs"])

    def test_rejects_source_path_with_parent_traversal_even_when_it_resolves_in_root(self) -> None:
        self.snapshots["candidate"]["source"] = "candidate/source/../source"
        self.write_manifest()

        result, report = self.invoke()

        self.assertEqual(result.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("source path is unsafe" in error for error in report["errors"]))

    def test_rejects_pristine_path_tree_hash_that_differs_from_baseline(self) -> None:
        baseline_hash = self.snapshots["baseline"]["path_tree_sha256"]
        self.snapshots["baseline"]["pristine_path_tree_sha256"] = baseline_hash
        self.snapshots["candidate"]["pristine_path_tree_sha256"] = "0" * 64
        self.write_manifest()

        result, report = self.invoke()

        self.assertEqual(result.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertIn(
            "snapshots.candidate.pristine_path_tree_sha256 does not match the baseline path-source package",
            report["errors"],
        )

    def test_rejects_snapshot_source_root_nested_in_other_snapshot(self) -> None:
        candidate_source = (
            self.root / "baseline/source/path-sources/gpui-pre-0.3.9/nested/source"
        )
        (candidate_source / "apps").mkdir(parents=True)
        (candidate_source / "apps/input.rs").write_text("same\n", encoding="utf-8")
        package = candidate_source / "path-sources/gpui-pre-0.3.9/src"
        package.mkdir(parents=True)
        (package / "scene.rs").write_text("after\n", encoding="utf-8")
        self.snapshots["candidate"]["source"] = str(candidate_source)

        for variant in ("baseline", "candidate"):
            source = Path(self.snapshots[variant]["source"])
            files = VALIDATOR._inventory(source.resolve())
            self.snapshots[variant]["files"] = files
            package_prefix = "path-sources/gpui-pre-0.3.9/"
            path_tree = {
                name.removeprefix(package_prefix): digest_value
                for name, digest_value in files.items()
                if name.startswith(package_prefix)
            }
            self.snapshots[variant]["path_tree_sha256"] = tree_hash(path_tree)
        self.write_manifest()

        result, report = self.invoke()

        self.assertEqual(result.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("source paths must not overlap" in error for error in report["errors"]))

    def test_rejects_regular_file_hard_linked_across_snapshots(self) -> None:
        baseline = self.root / "baseline/source/apps/input.rs"
        candidate = self.root / "candidate/source/apps/input.rs"
        candidate.unlink()
        os.link(baseline, candidate)
        self.snapshots["candidate"]["files"]["apps/input.rs"] = digest(candidate)
        self.write_manifest()

        result, report = self.invoke()

        self.assertEqual(result.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("shared between baseline and candidate" in error for error in report["errors"]))

    def test_rejects_snapshot_file_hard_linked_outside_both_snapshots(self) -> None:
        baseline = self.root / "baseline/source/apps/input.rs"
        outside = self.root / "outside-input.rs"
        os.link(baseline, outside)
        self.write_manifest()

        result, report = self.invoke()

        self.assertEqual(result.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("multiple hard links in source snapshot" in error
                            for error in report["errors"]))

    def test_rejects_hard_link_created_during_snapshot_hash(self) -> None:
        baseline = self.root / "baseline/source/apps/input.rs"
        outside = self.root / "outside-input-race.rs"
        original_fdopen = VALIDATOR.os.fdopen
        baseline_info = baseline.stat()
        linked = False

        def link_before_read(fd: int, *args: Any, **kwargs: Any):
            nonlocal linked
            info = os.fstat(fd)
            if not linked and (info.st_dev, info.st_ino) == (
                baseline_info.st_dev, baseline_info.st_ino
            ):
                os.link(baseline, outside)
                linked = True
            return original_fdopen(fd, *args, **kwargs)

        with mock.patch.object(VALIDATOR.os, "fdopen", side_effect=link_before_read):
            report = VALIDATOR.validate(self.manifest, self.root.resolve())

        self.assertTrue(linked)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("multiple hard links in source snapshot" in error for error in report["errors"]))

    def test_rejects_missing_unsupported_and_boolean_manifest_schema_versions(self) -> None:
        for version in (None, 2, True):
            with self.subTest(schema_version=version):
                if version is None:
                    self.manifest.pop("schema_version", None)
                else:
                    self.manifest["schema_version"] = version
                self.write_manifest()

                result, report = self.invoke()

                self.assertEqual(result.returncode, 1)
                self.assertEqual(report["status"], "invalid")
                self.assertIn("schema_version must be integer 1", report["errors"])
                self.manifest["schema_version"] = 1

    def test_rejects_snapshot_hash_drift_and_workspace_source_drift(self) -> None:
        scene = self.root / "candidate/source/path-sources/gpui-pre-0.3.9/src/scene.rs"
        scene.write_text("tampered\n", encoding="utf-8")
        result, report = self.invoke()
        self.assertEqual(result.returncode, 1)
        self.assertTrue(any("does not match" in error for error in report["errors"]))

        scene.write_text("after\n", encoding="utf-8")
        workspace = self.root / "candidate/source/apps/input.rs"
        workspace.write_text("changed\n", encoding="utf-8")
        self.snapshots["candidate"]["files"]["apps/input.rs"] = digest(workspace)
        self.write_manifest()
        _, report = self.invoke()
        self.assertTrue(any("only beneath path-sources" in error for error in report["errors"]))

    def test_rejects_missing_path_tree_hash(self) -> None:
        del self.snapshots["candidate"]["path_tree_sha256"]
        self.write_manifest()
        result, report = self.invoke()
        self.assertEqual(result.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertIn(
            "snapshots.candidate.path_tree_sha256 is required",
            report["errors"],
        )

    def test_rejects_path_source_file_outside_package_tree(self) -> None:
        source = self.root / "candidate/source"
        stray = source / "path-sources/stray.rs"
        stray.write_text("unbound package file\n", encoding="utf-8")
        self.snapshots["candidate"]["files"] = VALIDATOR._inventory(source.resolve())
        self.write_manifest()

        result, report = self.invoke()

        self.assertEqual(result.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertIn(
            "snapshots.candidate.path-sources files must be beneath a package directory",
            report["errors"],
        )

    def test_rejects_symlink_inside_snapshot(self) -> None:
        outside = self.root / "outside.rs"
        outside.write_text("external\n", encoding="utf-8")
        (self.root / "candidate/source/path-sources/escape.rs").symlink_to(outside)
        _, report = self.invoke()
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("symlink rejected" in error for error in report["errors"]))

    def test_rejects_symlinked_snapshot_root(self) -> None:
        source = self.root / "candidate/source"
        moved = self.root / "candidate/real-source"
        source.rename(moved)
        source.symlink_to(moved, target_is_directory=True)
        self.snapshots["candidate"]["source"] = "candidate/source"
        self.write_manifest()

        result, report = self.invoke()

        self.assertEqual(result.returncode, 1)
        self.assertTrue(any("source root" in error for error in report["errors"]))

    def test_rejects_symlinked_candidate_root(self) -> None:
        alias = self.root / "candidate-alias"
        alias.symlink_to(self.root, target_is_directory=True)
        for record in self.snapshots.values():
            source = Path(record["source"])
            record["source"] = str(alias / source.relative_to(self.root))
        self.write_manifest()

        result = subprocess.run(
            [sys.executable, str(SCRIPT), "manifest.json", "--root", str(alias)],
            capture_output=True, text=True, check=False,
        )
        report = json.loads(result.stdout)

        self.assertEqual(result.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("unsafe root path component" in error
                            for error in report["errors"]))

    def test_rejects_symlinked_manifest_path(self) -> None:
        alias = self.root / "manifest-alias.json"
        alias.symlink_to(self.manifest_path)

        result = subprocess.run(
            [sys.executable, str(SCRIPT), str(alias), "--root", str(self.root)],
            capture_output=True, text=True, check=False,
        )
        report = json.loads(result.stdout)

        self.assertEqual(result.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("symlink rejected in manifest path" in error for error in report["errors"]))

    def test_rejects_manifest_hard_linked_outside_candidate_root(self) -> None:
        outside = self.root.parent / f"{self.root.name}-manifest.json"
        self.addCleanup(outside.unlink, missing_ok=True)
        os.link(self.manifest_path, outside)

        result, report = self.invoke()

        self.assertEqual(result.returncode, 1)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("manifest has multiple hard links" in error
                            for error in report["errors"]))

    def test_manifest_open_rejects_symlink_swap_after_lexical_check(self) -> None:
        outside = self.root.parent / f"{self.root.name}-outside.json"
        self.addCleanup(outside.unlink, missing_ok=True)
        outside.write_text('{"outside": true}', encoding="utf-8")
        manifest = self.root / "manifest.json"
        manifest.write_text('{"inside": true}', encoding="utf-8")
        self.assertFalse(manifest.is_symlink())

        real_open = os.open
        swapped = False

        def swap_before_open(path, flags, mode=0o777, *, dir_fd=None):
            nonlocal swapped
            if path == "manifest.json" and dir_fd is not None and not swapped:
                manifest.unlink()
                manifest.symlink_to(outside)
                swapped = True
            if dir_fd is None:
                return real_open(path, flags, mode)
            return real_open(path, flags, mode, dir_fd=dir_fd)

        patched_open = mock.Mock(side_effect=swap_before_open)
        with mock.patch.object(VALIDATOR.os, "open", patched_open), \
                mock.patch.object(VALIDATOR.os, "supports_dir_fd", os.supports_dir_fd | {patched_open}):
            with self.assertRaises(OSError):
                VALIDATOR._load_manifest(manifest, self.root, self.root.resolve())

        self.assertTrue(swapped)

    def test_manifest_open_rejects_symlink_swap_in_root_ancestor(self) -> None:
        container = self.root / "container"
        root = container / "candidate"
        root.mkdir(parents=True)
        manifest = root / "manifest.json"
        manifest.write_text('{"inside": true}', encoding="utf-8")
        outside_parent = self.root / "outside-parent"
        outside_root = outside_parent / "candidate"
        outside_root.mkdir(parents=True)
        (outside_root / "manifest.json").write_text(
            '{"outside": true}', encoding="utf-8"
        )
        moved_container = self.root / "moved-container"
        real_open = VALIDATOR.os.open
        swapped = False

        def swap_root_ancestor(path, flags, mode=0o777, *, dir_fd=None):
            nonlocal swapped
            if not swapped and (path == root or path == container.name):
                container.rename(moved_container)
                container.symlink_to(outside_parent, target_is_directory=True)
                swapped = True
            if dir_fd is None:
                return real_open(path, flags, mode)
            return real_open(path, flags, mode, dir_fd=dir_fd)

        patched_open = mock.Mock(side_effect=swap_root_ancestor)
        with (
            mock.patch.object(VALIDATOR.os, "open", patched_open),
            mock.patch.object(
                VALIDATOR.os, "supports_dir_fd", os.supports_dir_fd | {patched_open}
            ),
            self.assertRaises(OSError),
        ):
            VALIDATOR._load_manifest(manifest, root, root.resolve())
        self.assertTrue(swapped)

    def test_manifest_read_rejects_mutation_while_reading(self) -> None:
        manifest = self.root / "manifest.json"
        manifest.write_text('{"inside": true}', encoding="utf-8")
        real_fdopen = VALIDATOR.os.fdopen

        class MutatingReader:
            def __init__(self, stream: Any) -> None:
                self.stream = stream

            def __enter__(self) -> "MutatingReader":
                self.stream.__enter__()
                return self

            def __exit__(self, *args: Any) -> Any:
                return self.stream.__exit__(*args)

            def fileno(self) -> int:
                return self.stream.fileno()

            def read(self, size: int = -1) -> bytes:
                raw = self.stream.read(size)
                with manifest.open("r+b") as writer:
                    writer.seek(raw.index(b"true"))
                    writer.write(b"f")
                return raw

        with mock.patch.object(
            VALIDATOR.os,
            "fdopen",
            side_effect=lambda *args, **kwargs: MutatingReader(
                real_fdopen(*args, **kwargs)
            ),
        ):
            with self.assertRaisesRegex(
                VALIDATOR.SnapshotError, "manifest changed while reading"
            ):
                VALIDATOR._load_manifest(manifest, self.root, self.root.resolve())

    def test_manifest_read_is_bounded_before_json_parse(self) -> None:
        manifest = self.root / "oversized.json"
        manifest.write_bytes(b" " * 18)
        read_sizes: list[int] = []
        real_fdopen = VALIDATOR.os.fdopen

        class ReadSpy:
            def __init__(self, stream: Any) -> None:
                self.stream = stream

            def __enter__(self) -> "ReadSpy":
                self.stream.__enter__()
                return self

            def __exit__(self, *args: Any) -> Any:
                return self.stream.__exit__(*args)

            def fileno(self) -> int:
                return self.stream.fileno()

            def read(self, size: int = -1) -> bytes:
                read_sizes.append(size)
                return self.stream.read(size)

        with (
            mock.patch.object(VALIDATOR, "MAX_MANIFEST_BYTES", 16),
            mock.patch.object(
                VALIDATOR.os,
                "fdopen",
                side_effect=lambda *args, **kwargs: ReadSpy(real_fdopen(*args, **kwargs)),
            ),
        ):
            with self.assertRaisesRegex(VALIDATOR.SnapshotError, "manifest exceeds 16 bytes"):
                VALIDATOR._load_manifest(manifest, self.root, self.root.resolve())

        self.assertEqual(read_sizes, [17])

    def test_snapshot_inventory_rejects_file_symlink_swap_before_open(self) -> None:
        source = self.root / "baseline/source/apps/input.rs"
        outside = self.root.parent / f"{self.root.name}-outside.rs"
        self.addCleanup(outside.unlink, missing_ok=True)
        outside.write_text("same\n", encoding="utf-8")
        self.assertFalse(source.is_symlink())

        real_open = os.open
        swapped = False

        def swap_before_open(path, flags, mode=0o777, *, dir_fd=None):
            nonlocal swapped
            if path == "input.rs" and dir_fd is not None and not swapped:
                source.unlink()
                source.symlink_to(outside)
                swapped = True
            if dir_fd is None:
                return real_open(path, flags, mode)
            return real_open(path, flags, mode, dir_fd=dir_fd)

        patched_open = mock.Mock(side_effect=swap_before_open)
        with mock.patch.object(VALIDATOR.os, "open", patched_open), \
                mock.patch.object(VALIDATOR.os, "supports_dir_fd", os.supports_dir_fd | {patched_open}):
            report = VALIDATOR.validate(self.manifest, self.root.resolve())

        self.assertTrue(swapped)
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("cannot be validated" in error for error in report["errors"]))

    def test_rejects_symlinked_snapshot_parent(self) -> None:
        source = self.root / "candidate/source"
        real_parent = self.root / "candidate/real-parent"
        real_parent.mkdir()
        source.rename(real_parent / "source")
        (self.root / "candidate/alias").symlink_to(real_parent, target_is_directory=True)
        self.snapshots["candidate"]["source"] = "candidate/alias/source"
        self.write_manifest()

        result, report = self.invoke()

        self.assertEqual(result.returncode, 1)
        self.assertTrue(any("symlink rejected in source path" in error for error in report["errors"]))

    def test_rejects_symlinked_path_that_resolves_to_candidate_root(self) -> None:
        alias = self.root / "root-alias"
        alias.symlink_to(self.root, target_is_directory=True)
        self.snapshots["candidate"]["source"] = str(alias / "candidate/source")
        self.write_manifest()

        result, report = self.invoke()

        self.assertEqual(result.returncode, 1)
        self.assertTrue(any("symlink rejected in source path" in error for error in report["errors"]))

    def test_rejects_source_path_escape_and_invalid_identity(self) -> None:
        self.snapshots["candidate"]["source"] = str(self.root.parent)
        self.manifest["candidate_id"] = "../bad"
        self.write_manifest()
        _, report = self.invoke()
        self.assertEqual(report["status"], "invalid")
        self.assertTrue(any("safe 1-64" in error for error in report["errors"]))
        self.assertTrue(any("cannot be validated" in error for error in report["errors"]))


if __name__ == "__main__":
    unittest.main()
