#!/usr/bin/env python3
"""Focused tests for exact, isolated candidate patch application."""

from __future__ import annotations

import hashlib
import json
import os
import resource
import shutil
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

from apply_candidate_patch import (  # noqa: E402
    MAX_PATCH_BYTES,
    PatchApplicationError,
    _cache_patch_at,
    _copy_directory_fd,
    _inventory,
    _inventory_fd,
    _remove_tree_at,
    _tree_hash,
    _strict_apply,
    _patch_series,
    apply_candidate_patch,
)
from p3_candidate_fs import CandidateFSError, PinnedDir  # noqa: E402


PACKAGE = "gpui-pre-0.3.9"
OLD = b"before\nkeep\n"
NEW = b"after\nkeep\n"
MID = b"middle\nkeep\n"
PATCH = (
    b"--- a/vendor/gpui-pre-0.3.9/src/scene.rs\n"
    b"+++ b/vendor/gpui-pre-0.3.9/src/scene.rs\n"
    b"@@ -1,2 +1,2 @@\n"
    b"-before\n"
    b"+after\n"
    b" keep\n"
)


class PatchCacheLimitTests(unittest.TestCase):
    def test_cache_patch_accepts_exact_32_mib_and_rejects_one_byte_more(self) -> None:
        self.assertEqual(MAX_PATCH_BYTES, 32 * 1024 * 1024)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            patch_path = root / "patch.bin"
            cache_path = root / "exact-cache.bin"
            block = bytes(range(256)) * 4096
            digest = hashlib.sha256()
            remaining = MAX_PATCH_BYTES
            with patch_path.open("wb") as patch:
                while remaining:
                    chunk = block[:min(len(block), remaining)]
                    patch.write(chunk)
                    digest.update(chunk)
                    remaining -= len(chunk)

            root_fd = os.open(root, os.O_RDONLY | os.O_DIRECTORY)
            try:
                self.assertEqual(
                    _cache_patch_at(root_fd, patch_path.name, cache_path),
                    digest.hexdigest(),
                )
                self.assertEqual(cache_path.stat().st_size, MAX_PATCH_BYTES)

                with patch_path.open("ab") as patch:
                    patch.write(b"x")
                with self.assertRaisesRegex(
                    PatchApplicationError,
                    f"patch exceeds {MAX_PATCH_BYTES} byte limit",
                ):
                    _cache_patch_at(root_fd, patch_path.name, root / "oversized-cache.bin")
            finally:
                os.close(root_fd)


class CopyDirectoryDescriptorTests(unittest.TestCase):
    def test_copy_deep_directory_chain_with_bounded_descriptors(self) -> None:
        depth = 1100
        directory_flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "source"
            destination = root / "destination"
            source.mkdir()
            destination.mkdir()
            active_fd = os.open(source, directory_flags)
            try:
                for _ in range(depth):
                    os.mkdir("d", dir_fd=active_fd)
                    child_fd = os.open("d", directory_flags, dir_fd=active_fd)
                    os.close(active_fd)
                    active_fd = child_fd
                leaf_fd = os.open("leaf", os.O_WRONLY | os.O_CREAT | os.O_EXCL,
                                  0o600, dir_fd=active_fd)
                try:
                    os.write(leaf_fd, b"x")
                finally:
                    os.close(leaf_fd)
            finally:
                os.close(active_fd)

            source_fd = os.open(source, directory_flags)
            destination_fd = os.open(destination, directory_flags)
            original_soft, hard_limit = resource.getrlimit(resource.RLIMIT_NOFILE)
            limit_changed = False
            try:
                descriptor_count = len(os.listdir("/dev/fd"))
                low_limit = max(32, descriptor_count + 8)
                if hard_limit != resource.RLIM_INFINITY and hard_limit < low_limit:
                    self.skipTest("hard file descriptor limit is too low for the regression")
                resource.setrlimit(resource.RLIMIT_NOFILE, (low_limit, hard_limit))
                limit_changed = True

                _copy_directory_fd(source_fd, destination_fd)

                current_fd = os.dup(destination_fd)
                try:
                    for _ in range(depth):
                        child_fd = os.open("d", directory_flags, dir_fd=current_fd)
                        os.close(current_fd)
                        current_fd = child_fd
                    copied_leaf_fd = os.open("leaf", os.O_RDONLY | os.O_NOFOLLOW,
                                             dir_fd=current_fd)
                    try:
                        self.assertEqual(os.read(copied_leaf_fd, 2), b"x")
                    finally:
                        os.close(copied_leaf_fd)
                finally:
                    os.close(current_fd)
            finally:
                if limit_changed:
                    resource.setrlimit(resource.RLIMIT_NOFILE, (original_soft, hard_limit))
                os.close(source_fd)
                os.close(destination_fd)

            def remove_deep_tree(path: Path) -> None:
                root_fd = os.open(path, directory_flags)
                current = os.dup(root_fd)
                levels = 0
                try:
                    while True:
                        try:
                            child = os.open("d", directory_flags, dir_fd=current)
                        except FileNotFoundError:
                            break
                        os.close(current)
                        current = child
                        levels += 1
                    os.unlink("leaf", dir_fd=current)
                    for _ in range(levels):
                        parent = os.open("..", directory_flags, dir_fd=current)
                        os.rmdir("d", dir_fd=parent)
                        os.close(current)
                        current = parent
                finally:
                    os.close(current)
                    os.close(root_fd)

            remove_deep_tree(destination)

    def test_rollback_removes_deep_tree_with_bounded_descriptors(self) -> None:
        depth = 1100
        directory_flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
        with tempfile.TemporaryDirectory() as temporary:
            root_path = Path(temporary) / "root"
            package = root_path / "package"
            root_path.mkdir()
            package.mkdir()
            active_fd = os.open(package, directory_flags)
            try:
                for _ in range(depth):
                    os.mkdir("d", dir_fd=active_fd)
                    child_fd = os.open("d", directory_flags, dir_fd=active_fd)
                    os.close(active_fd)
                    active_fd = child_fd
                leaf_fd = os.open("leaf", os.O_WRONLY | os.O_CREAT | os.O_EXCL,
                                  0o600, dir_fd=active_fd)
                os.close(leaf_fd)
            finally:
                os.close(active_fd)

            root_fd = os.open(root_path, directory_flags)
            package_fd = os.open("package", directory_flags, dir_fd=root_fd)
            package_info = os.fstat(package_fd)
            os.close(package_fd)
            original_soft, hard_limit = resource.getrlimit(resource.RLIMIT_NOFILE)
            limit_changed = False
            try:
                descriptor_count = len(os.listdir("/dev/fd"))
                low_limit = max(32, descriptor_count + 8)
                if low_limit >= depth:
                    self.skipTest("cannot set a descriptor limit far below the tree depth")
                if hard_limit != resource.RLIM_INFINITY and hard_limit < low_limit:
                    self.skipTest("hard file descriptor limit is too low for the regression")
                resource.setrlimit(resource.RLIMIT_NOFILE, (low_limit, hard_limit))
                limit_changed = True

                _remove_tree_at(root_fd, "package", (package_info.st_dev, package_info.st_ino))

                self.assertFalse(package.exists())
            finally:
                if limit_changed:
                    resource.setrlimit(resource.RLIMIT_NOFILE, (original_soft, hard_limit))
                os.close(root_fd)

    def test_rollback_enforces_snapshot_entry_limit(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root_path = Path(temporary)
            package = root_path / "package"
            package.mkdir()
            (package / "first").write_text("one")
            (package / "second").write_text("two")
            root_fd = os.open(root_path, os.O_RDONLY | os.O_DIRECTORY)
            package_fd = os.open("package", os.O_RDONLY | os.O_DIRECTORY, dir_fd=root_fd)
            package_info = os.fstat(package_fd)
            os.close(package_fd)
            try:
                with mock.patch("apply_candidate_patch.MAX_SNAPSHOT_ENTRIES", 1):
                    with self.assertRaisesRegex(PatchApplicationError, "exceeds 1 entries"):
                        _remove_tree_at(root_fd, "package",
                                        (package_info.st_dev, package_info.st_ino))
                self.assertEqual(sorted(path.name for path in package.iterdir()), [
                    "first", "second",
                ])
            finally:
                os.close(root_fd)

    def test_rollback_preserves_replaced_file_entry(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root_path = Path(temporary)
            package = root_path / "package"
            package.mkdir()
            entry = package / "payload"
            entry.write_text("original")
            root_fd = os.open(root_path, os.O_RDONLY | os.O_DIRECTORY)
            package_fd = os.open("package", os.O_RDONLY | os.O_DIRECTORY, dir_fd=root_fd)
            package_info = os.fstat(package_fd)
            os.close(package_fd)
            original_stat = os.stat
            swapped = False

            def replace_after_observation(path, *args, **kwargs):
                nonlocal swapped
                info = original_stat(path, *args, **kwargs)
                if path == "payload" and kwargs.get("dir_fd") is not None and not swapped:
                    entry.rename(package / "detached")
                    entry.write_text("replacement")
                    swapped = True
                return info

            try:
                with mock.patch("p3_candidate_fs.os.stat", side_effect=replace_after_observation):
                    with self.assertRaisesRegex(PatchApplicationError, "entry changed during rollback"):
                        _remove_tree_at(root_fd, "package",
                                        (package_info.st_dev, package_info.st_ino))
                self.assertTrue(swapped)
                self.assertEqual(entry.read_text(), "replacement")
                self.assertEqual((package / "detached").read_text(), "original")
            finally:
                os.close(root_fd)

    def test_inventory_and_copy_enforce_snapshot_entry_limit(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "source"
            destination = root / "destination"
            source.mkdir()
            destination.mkdir()
            (source / "first").write_bytes(b"one")
            (source / "second").write_bytes(b"two")
            source_fd = os.open(source, os.O_RDONLY | os.O_DIRECTORY)
            destination_fd = os.open(destination, os.O_RDONLY | os.O_DIRECTORY)
            try:
                with mock.patch("apply_candidate_patch.MAX_SNAPSHOT_ENTRIES", 1):
                    with self.assertRaisesRegex(PatchApplicationError, "exceeds 1 entries"):
                        _inventory(source)
                    with self.assertRaisesRegex(PatchApplicationError, "exceeds 1 entries"):
                        _inventory_fd(source_fd)
                    with self.assertRaisesRegex(PatchApplicationError, "exceeds 1 entries"):
                        _copy_directory_fd(source_fd, destination_fd)
                self.assertFalse((destination / "first").exists())
                self.assertFalse((destination / "second").exists())
            finally:
                os.close(source_fd)
                os.close(destination_fd)

    def test_inventory_and_copy_enforce_snapshot_byte_limit(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "source"
            destination = root / "destination"
            source.mkdir()
            destination.mkdir()
            (source / "file").write_bytes(b"four")
            source_fd = os.open(source, os.O_RDONLY | os.O_DIRECTORY)
            destination_fd = os.open(destination, os.O_RDONLY | os.O_DIRECTORY)
            try:
                with mock.patch("apply_candidate_patch.MAX_SNAPSHOT_BYTES", 3):
                    with self.assertRaisesRegex(PatchApplicationError, "regular-file bytes"):
                        _inventory(source)
                    with self.assertRaisesRegex(PatchApplicationError, "regular-file bytes"):
                        _inventory_fd(source_fd)
                    with self.assertRaisesRegex(PatchApplicationError, "regular-file bytes"):
                        _copy_directory_fd(source_fd, destination_fd)
                self.assertFalse((destination / "file").exists())
            finally:
                os.close(source_fd)
                os.close(destination_fd)

    def test_inventory_rejects_in_place_file_mutation_while_hashing(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "source"
            source.mkdir()
            entry = source / "payload"
            chunk_size = 1024 * 1024
            entry.write_bytes(b"a" * (2 * chunk_size))
            before = entry.stat()
            source_fd = os.open(source, os.O_RDONLY | os.O_DIRECTORY)
            original_sha256 = hashlib.sha256
            mutated = False

            class MutatingDigest:
                def __init__(self) -> None:
                    self.digest = original_sha256()

                def update(self, block: bytes) -> None:
                    nonlocal mutated
                    if not mutated:
                        with entry.open("r+b") as stream:
                            stream.seek(chunk_size)
                            stream.write(b"b" * chunk_size)
                        mutated = True
                    self.digest.update(block)

                def hexdigest(self) -> str:
                    return self.digest.hexdigest()

            try:
                with mock.patch("apply_candidate_patch.hashlib.sha256", MutatingDigest):
                    with self.assertRaisesRegex(PatchApplicationError, "changed.*inventory"):
                        _inventory_fd(source_fd)
                after = entry.stat()
                self.assertTrue(mutated)
                self.assertEqual((after.st_dev, after.st_ino), (before.st_dev, before.st_ino))
                self.assertEqual(after.st_size, before.st_size)
            finally:
                os.close(source_fd)

    def test_inventory_handles_deep_directory_chain_with_bounded_descriptors(self) -> None:
        depth = 1100
        directory_flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "source"
            source.mkdir()
            active_fd = os.open(source, directory_flags)
            try:
                for _ in range(depth):
                    os.mkdir("d", dir_fd=active_fd)
                    child_fd = os.open("d", directory_flags, dir_fd=active_fd)
                    os.close(active_fd)
                    active_fd = child_fd
                leaf_fd = os.open(
                    "leaf", os.O_WRONLY | os.O_CREAT | os.O_EXCL,
                    0o600, dir_fd=active_fd,
                )
                try:
                    os.write(leaf_fd, b"x")
                finally:
                    os.close(leaf_fd)
            finally:
                os.close(active_fd)

            source_fd = os.open(source, directory_flags)
            original_soft, hard_limit = resource.getrlimit(resource.RLIMIT_NOFILE)
            limit_changed = False
            try:
                descriptor_count = len(os.listdir("/dev/fd"))
                low_limit = max(32, descriptor_count + 8)
                if hard_limit != resource.RLIM_INFINITY and hard_limit < low_limit:
                    self.skipTest("hard file descriptor limit is too low for the regression")
                resource.setrlimit(resource.RLIMIT_NOFILE, (low_limit, hard_limit))
                limit_changed = True

                inventory = _inventory_fd(source_fd)

                expected_path = "/".join(["d"] * depth + ["leaf"])
                self.assertEqual(inventory, {expected_path: hashlib.sha256(b"x").hexdigest()})
            finally:
                if limit_changed:
                    resource.setrlimit(resource.RLIMIT_NOFILE, (original_soft, hard_limit))
                os.close(source_fd)

    def test_inventory_preserves_p3k_hardlink_behavior(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            package = root / "package"
            package.mkdir()
            original = package / "original"
            alias = package / "alias"
            original.write_bytes(b"shared package bytes")
            try:
                os.link(original, alias)
            except OSError as error:
                self.skipTest(f"hard links are unavailable: {error}")

            expected_hash = hashlib.sha256(b"shared package bytes").hexdigest()
            expected = {"alias": expected_hash, "original": expected_hash}
            package_fd = os.open(package, os.O_RDONLY | os.O_DIRECTORY)
            try:
                self.assertEqual(_inventory(package), expected)
                self.assertEqual(_inventory_fd(package_fd), expected)
                with PinnedDir.from_fd(package_fd) as pinned:
                    with self.assertRaisesRegex(CandidateFSError, "hard-linked file rejected"):
                        pinned.inventory()
            finally:
                os.close(package_fd)

    def test_copy_failure_preserves_reused_destination_descriptor(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "source"
            destination = root / "destination"
            source.mkdir()
            destination.mkdir()
            (source / "file").write_bytes(b"source")
            source_fd = os.open(source, os.O_RDONLY | os.O_DIRECTORY)
            destination_fd = os.open(destination, os.O_RDONLY | os.O_DIRECTORY)
            fdopen = os.fdopen
            retained: list[int] = []
            written: list[int] = []

            class DestinationStream:
                def __init__(self, stream):
                    self.stream = stream

                def __enter__(self):
                    self.stream.__enter__()
                    return self

                def write(self, _contents):
                    raise OSError("copy failed")

                def flush(self):
                    self.stream.flush()

                def fileno(self):
                    return self.stream.fileno()

                def __exit__(self, *arguments):
                    result = self.stream.__exit__(*arguments)
                    retained.append(os.open(root / "retained", os.O_RDWR | os.O_CREAT, 0o600))
                    return result

            def wrap_fdopen(fd, mode):
                stream = fdopen(fd, mode)
                if mode == "wb":
                    written.append(fd)
                    return DestinationStream(stream)
                return stream

            try:
                with mock.patch("apply_candidate_patch.os.fdopen", side_effect=wrap_fdopen):
                    with self.assertRaisesRegex(OSError, "copy failed"):
                        _copy_directory_fd(source_fd, destination_fd)
                self.assertEqual(retained, written)
                os.fstat(retained[0])
            finally:
                for fd in retained:
                    try:
                        os.close(fd)
                    except OSError:
                        pass
                os.close(source_fd)
                os.close(destination_fd)


class StrictPatchParserTests(unittest.TestCase):
    def test_applies_lf_diff_with_unicode_next_line_character_in_source(self) -> None:
        source = "before\u0085middle\nkeep\n".encode()
        patch = (
            "--- a/vendor/gpui-pre-0.3.9/src/scene.rs\n"
            "+++ b/vendor/gpui-pre-0.3.9/src/scene.rs\n"
            "@@ -1,2 +1,2 @@\n"
            "-before\u0085middle\n"
            "+after\u0085middle\n"
            " keep\n"
        ).encode()

        self.assertEqual(
            _strict_apply(source, patch),
            "after\u0085middle\nkeep\n".encode(),
        )

    def test_rejects_empty_hunk_before_valid_change(self) -> None:
        headers = (
            b"--- a/vendor/gpui-pre-0.3.9/src/scene.rs\n"
            b"+++ b/vendor/gpui-pre-0.3.9/src/scene.rs\n"
        )
        malformed = headers + b"@@ -0,0 +0,0 @@\n" + PATCH[len(headers):]

        with self.assertRaisesRegex(PatchApplicationError, "empty unified-diff hunk"):
            _strict_apply(OLD, malformed)


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def replace_patch(before: bytes, after: bytes) -> bytes:
    return (
        b"--- a/vendor/gpui-pre-0.3.9/src/scene.rs\n"
        b"+++ b/vendor/gpui-pre-0.3.9/src/scene.rs\n"
        b"@@ -1,2 +1,2 @@\n-"
        + before.splitlines(keepends=True)[0]
        + b"+"
        + after.splitlines(keepends=True)[0]
        + b" "
        + before.splitlines(keepends=True)[1]
    )


def replace_package_file_patch(path: str, before: bytes, after: bytes) -> bytes:
    target = f"vendor/{PACKAGE}/{path}".encode()
    old_lines = before.splitlines(keepends=True)
    new_lines = after.splitlines(keepends=True)
    return (b"--- a/" + target + b"\n+++ b/" + target + b"\n"
            + f"@@ -1,{len(old_lines)} +1,{len(new_lines)} @@\n".encode()
            + b"".join(b"-" + line for line in old_lines)
            + b"".join(b"+" + line for line in new_lines))


def write_package(path: Path, scene: bytes) -> dict[str, str]:
    target = path / "src/scene.rs"
    target.parent.mkdir(parents=True)
    target.write_bytes(scene)
    (path / "Cargo.toml").write_text("[package]\nname = \"gpui-pre\"\n")
    files = {
        item.relative_to(path).as_posix(): digest(item.read_bytes())
        for item in path.rglob("*")
        if item.is_file()
    }
    return dict(sorted(files.items()))


class CandidatePatchApplicationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.repo = Path(self.temporary.name)
        self.candidate_root = self.repo / "target/headstart/candidates/sample-candidate"
        self.candidate_root.mkdir(parents=True)
        self.baseline_source = self.candidate_root / "baseline/source"
        self.candidate_source = self.candidate_root / "candidate/source"
        self.baseline_package = self.baseline_source / "path-sources" / PACKAGE
        self.candidate_package = self.candidate_source / "path-sources" / PACKAGE
        baseline_inventory = write_package(self.baseline_package, OLD)
        candidate_inventory = write_package(self.candidate_package, NEW)
        self.patch_path = self.candidate_root / "approved.patch"
        self.patch_path.write_bytes(PATCH)
        self.manifest_path = self.candidate_root / "manifest.json"
        self.manifest = {
            "schema_version": 1,
            "candidate_id": "sample-candidate",
            "patch_sha256": digest(PATCH),
            "preimage_sha256": digest(OLD),
            "postimage_sha256": digest(NEW),
            "snapshots": {
                "baseline": {
                    "source": str(self.baseline_source),
                    "path_tree_sha256": _tree_hash(baseline_inventory),
                },
                "candidate": {
                    "source": str(self.candidate_source),
                    "path_tree_sha256": _tree_hash(candidate_inventory),
                },
            },
        }
        self.write_manifest()

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def write_manifest(self) -> None:
        self.manifest_path.write_text(json.dumps(self.manifest), encoding="utf-8")

    def run_patch(self, run_id: str = "check-1") -> dict[str, object]:
        return apply_candidate_patch(self.manifest_path, run_id, self.repo)

    def run_patch_with_path_swap(
        self,
        patch_path: Path,
        validated_patch: bytes,
        replacement_patch: bytes,
    ) -> tuple[dict[str, object], list[bytes]]:
        module = sys.modules["apply_candidate_patch"]
        replacement_path = self.repo / "replacement.patch"
        replacement_path.write_bytes(replacement_patch)
        original_open_directory = module._open_or_create_directory_at
        original_apply = module._strict_apply
        applied_patches: list[bytes] = []

        def swap_after_validation(parent_fd: int, name: str, label: str) -> int:
            descriptor = original_open_directory(parent_fd, name, label)
            if name == "patch-checks":
                patch_path.unlink()
                patch_path.symlink_to(replacement_path)
            return descriptor

        def apply_and_restore(source: bytes, patch_contents: bytes, *args: object) -> bytes:
            applied_patches.append(patch_contents)
            try:
                return original_apply(source, patch_contents, *args)
            finally:
                if patch_path.is_symlink():
                    patch_path.unlink()
                    patch_path.write_bytes(validated_patch)

        with mock.patch(
            "apply_candidate_patch._open_or_create_directory_at",
            side_effect=swap_after_validation,
        ), mock.patch(
            "apply_candidate_patch._strict_apply",
            side_effect=apply_and_restore,
        ):
            report = self.run_patch()
        return report, applied_patches

    def set_patch_series(self, series: list[tuple[str, bytes, bytes, bytes]]) -> None:
        records = []
        descriptors = []
        for name, patch, before, after in series:
            (self.candidate_root / name).write_bytes(patch)
            patch_hash = digest(patch)
            descriptors.append(f"{name}:{patch_hash}")
            records.append({
                "path": name,
                "sha256": patch_hash,
                "preimage_sha256": digest(before),
                "postimage_sha256": digest(after),
            })
        self.manifest["schema_version"] = 2
        self.manifest["patch_sha256"] = digest("\n".join(descriptors).encode())
        self.manifest["patches"] = records
        self.write_manifest()

    def set_multi_file_series(self, series: list[tuple[str, str, bytes, bytes]]) -> None:
        records = []
        descriptors = []
        for name, package_file, before, after in series:
            patch = replace_package_file_patch(package_file, before, after)
            (self.candidate_root / name).write_bytes(patch)
            patch_hash = digest(patch)
            descriptors.append(f"{name}:{patch_hash}")
            records.append({
                "path": name,
                "file": package_file,
                "sha256": patch_hash,
                "preimage_sha256": digest(before),
                "postimage_sha256": digest(after),
            })
        extra = "src/other.rs"
        (self.baseline_package / extra).write_bytes(b"old-other\n")
        (self.candidate_package / extra).write_bytes(b"new-other\n")
        baseline = {
            path.relative_to(self.baseline_package).as_posix(): digest(path.read_bytes())
            for path in self.baseline_package.rglob("*") if path.is_file()
        }
        candidate = {
            path.relative_to(self.candidate_package).as_posix(): digest(path.read_bytes())
            for path in self.candidate_package.rglob("*") if path.is_file()
        }
        self.manifest["snapshots"]["baseline"]["path_tree_sha256"] = _tree_hash(baseline)
        self.manifest["snapshots"]["candidate"]["path_tree_sha256"] = _tree_hash(candidate)
        self.manifest.pop("preimage_sha256", None)
        self.manifest.pop("postimage_sha256", None)
        self.manifest["schema_version"] = 3
        self.manifest["patch_sha256"] = digest("\n".join(descriptors).encode())
        self.manifest["patches"] = records
        self.write_manifest()

    def test_applies_exact_patch_to_fresh_copy_and_removes_patch_cache(self) -> None:
        before_baseline = (self.baseline_package / "src/scene.rs").read_bytes()
        before_candidate = (self.candidate_package / "src/scene.rs").read_bytes()
        module = sys.modules["apply_candidate_patch"]
        original_temporary_directory = module.tempfile.TemporaryDirectory
        cache_dirs: list[Path] = []

        def capture_cache(*args: object, **kwargs: object) -> tempfile.TemporaryDirectory:
            cache = original_temporary_directory(*args, **kwargs)
            cache_dirs.append(Path(cache.name))
            return cache

        with mock.patch.object(
            module.tempfile, "TemporaryDirectory", side_effect=capture_cache
        ):
            report = self.run_patch()

        self.assertEqual(report["status"], "passed")
        self.assertEqual([path.exists() for path in cache_dirs], [False])
        output = Path(str(report["output_package"])) / "src/scene.rs"
        self.assertEqual(output.read_bytes(), NEW)
        self.assertEqual((self.baseline_package / "src/scene.rs").read_bytes(), before_baseline)
        self.assertEqual((self.candidate_package / "src/scene.rs").read_bytes(), before_candidate)
        self.assertTrue((output.parents[2] / "result.json").exists())

    def test_applies_patch_to_read_only_baseline_file_and_preserves_mode(self) -> None:
        baseline_scene = self.baseline_package / "src/scene.rs"
        baseline_scene.chmod(0o444)

        report = self.run_patch()

        output = Path(str(report["output_package"])) / "src/scene.rs"
        self.assertEqual(output.read_bytes(), NEW)
        self.assertEqual(output.stat().st_mode & 0o777, 0o444)
        self.assertEqual(baseline_scene.stat().st_mode & 0o777, 0o444)

    def test_rejects_unpatched_file_drift_in_single_file_output_copy(self) -> None:
        module = sys.modules["apply_candidate_patch"]
        original_copy = module._copy_directory_fd

        def alter_unpatched_file(source_fd: int, destination_fd: int) -> None:
            original_copy(source_fd, destination_fd)
            try:
                fd = module._open_file_at(
                    destination_fd, "Cargo.toml", module.os.O_WRONLY | module.os.O_TRUNC
                )
            except PatchApplicationError:
                return
            with module.os.fdopen(fd, "wb") as stream:
                stream.write(b'[package]\\nname = "tampered-copy"\\n')

        with mock.patch(
            "apply_candidate_patch._copy_directory_fd", side_effect=alter_unpatched_file
        ):
            with self.assertRaisesRegex(
                PatchApplicationError, "copied postimage inventory differs from candidate"
            ):
                self.run_patch()

        result = json.loads(
            (self.candidate_root / "patch-checks/check-1/result.json").read_text()
        )
        self.assertEqual(result["status"], "rejected")
        self.assertEqual(
            (Path(result["output_package"]) / "src/scene.rs").read_bytes(), NEW
        )
        self.assertNotEqual(
            (Path(result["output_package"]) / "Cargo.toml").read_bytes(),
            (self.candidate_package / "Cargo.toml").read_bytes(),
        )
    def test_rejects_symlinked_baseline_package_root(self) -> None:
        external = self.repo / "external-baseline-package"
        self.baseline_package.rename(external)
        self.baseline_package.symlink_to(external, target_is_directory=True)

        with self.assertRaisesRegex(PatchApplicationError, "package root must be a real directory"):
            self.run_patch()
        self.assertFalse((self.candidate_root / "patch-checks/check-1").exists())

    def test_rejects_manifest_symlink_inserted_after_path_resolution(self) -> None:
        external_manifest = self.repo / "external-manifest.json"
        external_manifest.write_bytes(self.manifest_path.read_bytes())
        original_resolve = sys.modules["apply_candidate_patch"]._resolve_input

        def resolve_then_swap(path: Path, root: Path, lexical_root: Path) -> Path:
            resolved = original_resolve(path, root, lexical_root)
            if path == self.manifest_path:
                self.manifest_path.unlink()
                self.manifest_path.symlink_to(external_manifest)
            return resolved

        with mock.patch(
            "apply_candidate_patch._resolve_input", side_effect=resolve_then_swap
        ):
            with self.assertRaisesRegex(PatchApplicationError, "file cannot be opened safely"):
                self.run_patch()

        self.assertTrue(self.manifest_path.is_symlink())
        self.assertFalse((self.candidate_root / "patch-checks/check-1").exists())

    def test_postcheck_rejects_manifest_symlink_swap(self) -> None:
        module = sys.modules["apply_candidate_patch"]
        external_manifest = self.repo / "external-postcheck-manifest.json"
        external_manifest.write_bytes(self.manifest_path.read_bytes())
        original_apply = module._strict_apply

        def apply_then_swap_manifest(source: bytes, patch: bytes, *args: object) -> bytes:
            applied = original_apply(source, patch, *args)
            self.manifest_path.unlink()
            self.manifest_path.symlink_to(external_manifest)
            return applied

        with mock.patch(
            "apply_candidate_patch._strict_apply", side_effect=apply_then_swap_manifest
        ):
            with self.assertRaisesRegex(PatchApplicationError, "post-check integrity verification failed"):
                self.run_patch()

        result = json.loads(
            (self.candidate_root / "patch-checks/check-1/result.json").read_text()
        )
        self.assertEqual(result["status"], "rejected")
        self.assertIn("file cannot be opened safely", result["error"])

    def test_rejects_symlinked_candidate_package_root(self) -> None:
        external = self.repo / "external-candidate-package"
        self.candidate_package.rename(external)
        self.candidate_package.symlink_to(external, target_is_directory=True)

        with self.assertRaisesRegex(PatchApplicationError, "package root must be a real directory"):
            self.run_patch()
        self.assertFalse((self.candidate_root / "patch-checks/check-1").exists())

    def test_rejects_symlinked_package_source_parent(self) -> None:
        for source_root, label in (
            (self.baseline_source, "baseline"),
            (self.candidate_source, "candidate"),
        ):
            with self.subTest(source=label):
                source_parent = source_root / "path-sources"
                external = self.repo / f"external-{label}-path-sources"
                source_parent.rename(external)
                source_parent.symlink_to(external, target_is_directory=True)

                with self.assertRaisesRegex(
                    PatchApplicationError, "package root must be a real directory"
                ):
                    self.run_patch()
                self.assertFalse((self.candidate_root / "patch-checks/check-1").exists())

                source_parent.unlink()
                external.rename(source_parent)

    def test_interrupted_result_write_never_publishes_partial_json(self) -> None:
        run_id = "atomic-result"

        def interrupted_dump(_result, stream, **_kwargs):
            stream.write('{"schema_version":')
            raise OSError("simulated interrupted result write")

        with mock.patch("apply_candidate_patch.json.dump", side_effect=interrupted_dump):
            with self.assertRaisesRegex(OSError, "interrupted result write"):
                apply_candidate_patch(self.manifest_path, run_id, self.repo)

        output_root = self.candidate_root / "patch-checks" / run_id
        self.assertFalse((output_root / "result.json").exists())
        self.assertEqual(list(output_root.glob(".result.json.*.tmp")), [])

    def test_rejects_patch_hash_mismatch_before_creating_output(self) -> None:
        self.manifest["patch_sha256"] = "0" * 64
        self.write_manifest()

        with self.assertRaisesRegex(PatchApplicationError, "patch hash"):
            self.run_patch()
        self.assertFalse((self.candidate_root / "patch-checks/check-1").exists())

    def test_rejects_boolean_schema_version(self) -> None:
        self.manifest["schema_version"] = True
        self.write_manifest()

        with self.assertRaisesRegex(PatchApplicationError, "unsupported or malformed"):
            self.run_patch()
        self.assertFalse((self.candidate_root / "patch-checks/check-1").exists())

    def test_rejects_nonstandard_json_constants(self) -> None:
        self.manifest["unused_metadata"] = float("nan")
        self.write_manifest()

        with self.assertRaisesRegex(PatchApplicationError, "non-standard JSON constant: NaN"):
            self.run_patch()
        self.assertFalse((self.candidate_root / "patch-checks/check-1").exists())

    def test_rejects_preimage_mismatch_and_records_failure(self) -> None:
        (self.baseline_package / "src/scene.rs").write_bytes(b"drifted\nkeep\n")
        inventory = {
            item.relative_to(self.baseline_package).as_posix(): digest(item.read_bytes())
            for item in self.baseline_package.rglob("*")
            if item.is_file()
        }
        self.manifest["snapshots"]["baseline"]["path_tree_sha256"] = _tree_hash(inventory)
        self.write_manifest()

        with self.assertRaisesRegex(PatchApplicationError, "baseline scene preimage"):
            self.run_patch()

    def test_rejects_offset_or_fuzzy_hunk_match(self) -> None:
        shifted = PATCH.replace(b"@@ -1,2 +1,2 @@", b"@@ -2,2 +2,2 @@")
        self.patch_path.write_bytes(shifted)
        self.manifest["patch_sha256"] = digest(shifted)
        self.write_manifest()

        with self.assertRaisesRegex(PatchApplicationError, "exact preimage mismatch"):
            self.run_patch()

    def test_rejects_noop_patch_and_records_failure(self) -> None:
        noop = (
            b"--- a/vendor/gpui-pre-0.3.9/src/scene.rs\n"
            b"+++ b/vendor/gpui-pre-0.3.9/src/scene.rs\n"
            b"@@ -1,2 +1,2 @@\n"
            b" before\n"
            b" keep\n"
        )
        target = self.candidate_package / "src/scene.rs"
        target.write_bytes(OLD)
        candidate_inventory = {
            item.relative_to(self.candidate_package).as_posix(): digest(item.read_bytes())
            for item in self.candidate_package.rglob("*") if item.is_file()
        }
        self.manifest["snapshots"]["candidate"]["path_tree_sha256"] = _tree_hash(
            candidate_inventory
        )
        self.manifest["patch_sha256"] = digest(noop)
        self.manifest["postimage_sha256"] = digest(OLD)
        self.patch_path.write_bytes(noop)
        self.write_manifest()

        with self.assertRaisesRegex(PatchApplicationError, "patch does not change its target"):
            self.run_patch()

        result = json.loads(
            (self.candidate_root / "patch-checks/check-1/result.json").read_text()
        )
        self.assertEqual(result["status"], "rejected")
        self.assertIn("patch does not change its target", result["error"])

    def test_rejects_unterminated_final_diff_row_and_records_failure(self) -> None:
        patch = PATCH[:-1]
        target = self.candidate_package / "src/scene.rs"
        target.write_bytes(b"after\nkeep")
        candidate_inventory = {
            item.relative_to(self.candidate_package).as_posix(): digest(item.read_bytes())
            for item in self.candidate_package.rglob("*") if item.is_file()
        }
        self.manifest["snapshots"]["candidate"]["path_tree_sha256"] = _tree_hash(
            candidate_inventory
        )
        self.manifest["patch_sha256"] = digest(patch)
        self.manifest["postimage_sha256"] = digest(b"after\nkeep")
        self.patch_path.write_bytes(patch)
        self.write_manifest()

        with self.assertRaisesRegex(PatchApplicationError, "unterminated unified-diff line"):
            self.run_patch()

        result = json.loads(
            (self.candidate_root / "patch-checks/check-1/result.json").read_text()
        )
        self.assertEqual(result["status"], "rejected")
        self.assertIn("unterminated unified-diff line", result["error"])

    def test_rejects_oversized_hunk_number_and_records_failure(self) -> None:
        oversized = PATCH.replace(
            b"@@ -1,2 +1,2 @@", b"@@ -" + b"9" * 5000 + b",2 +1,2 @@"
        )
        self.patch_path.write_bytes(oversized)
        self.manifest["patch_sha256"] = digest(oversized)
        self.write_manifest()

        with self.assertRaisesRegex(PatchApplicationError, "line number or count is too large"):
            self.run_patch()

        result = json.loads(
            (self.candidate_root / "patch-checks/check-1/result.json").read_text()
        )
        self.assertEqual(result["status"], "rejected")
        self.assertIn("line number or count is too large", result["error"])

    def test_rejects_non_ascii_hunk_digits_and_records_failure(self) -> None:
        non_ascii_digits = PATCH.replace(b"@@ -1,2 +1,2 @@", "@@ -١,2 +١,2 @@".encode())
        self.patch_path.write_bytes(non_ascii_digits)
        self.manifest["patch_sha256"] = digest(non_ascii_digits)
        self.write_manifest()

        with self.assertRaisesRegex(PatchApplicationError, "unexpected unified-diff line"):
            self.run_patch()

        result = json.loads(
            (self.candidate_root / "patch-checks/check-1/result.json").read_text()
        )
        self.assertEqual(result["status"], "rejected")
        self.assertIn("unexpected unified-diff line", result["error"])

    def test_rejects_manifest_postimage_mismatch_before_copy(self) -> None:
        self.manifest["postimage_sha256"] = "0" * 64
        self.write_manifest()

        with self.assertRaisesRegex(PatchApplicationError, "candidate scene postimage"):
            self.run_patch()
        self.assertFalse((self.candidate_root / "patch-checks/check-1").exists())
        self.assertEqual((self.baseline_package / "src/scene.rs").read_bytes(), OLD)
        self.assertEqual((self.candidate_package / "src/scene.rs").read_bytes(), NEW)

    def test_rejects_applied_bytes_that_miss_candidate_postimage(self) -> None:
        wrong_patch = PATCH.replace(b"+after\n", b"+later\n")
        self.patch_path.write_bytes(wrong_patch)
        self.manifest["patch_sha256"] = digest(wrong_patch)
        self.write_manifest()

        with self.assertRaisesRegex(PatchApplicationError, "postimage hash"):
            self.run_patch()

        run_root = self.candidate_root / "patch-checks/check-1"
        result = json.loads((run_root / "result.json").read_text())
        self.assertEqual(result["status"], "rejected")
        self.assertEqual((run_root / "package/src/scene.rs").read_bytes(), b"later\nkeep\n")
        self.assertEqual((self.baseline_package / "src/scene.rs").read_bytes(), OLD)
        self.assertEqual((self.candidate_package / "src/scene.rs").read_bytes(), NEW)

    def test_rejects_reused_run_id_and_removes_patch_cache(self) -> None:
        self.run_patch()
        module = sys.modules["apply_candidate_patch"]
        original_temporary_directory = module.tempfile.TemporaryDirectory
        cache_dirs: list[Path] = []

        def capture_cache(*args: object, **kwargs: object) -> tempfile.TemporaryDirectory:
            cache = original_temporary_directory(*args, **kwargs)
            cache_dirs.append(Path(cache.name))
            return cache

        with mock.patch.object(
            module.tempfile, "TemporaryDirectory", side_effect=capture_cache
        ):
            with self.assertRaisesRegex(PatchApplicationError, "already exists"):
                self.run_patch()

        self.assertEqual([path.exists() for path in cache_dirs], [False])

    def test_rejects_patch_check_root_swap_after_open(self) -> None:
        module = sys.modules["apply_candidate_patch"]
        outside = self.repo / "outside-output"
        outside.mkdir()
        patch_checks = self.candidate_root / "patch-checks"
        moved = self.candidate_root / "patch-checks-moved"
        original_apply = module._strict_apply

        def apply_then_swap_output_root(source: bytes, patch: bytes,
                                        *args: object) -> bytes:
            applied = original_apply(source, patch, *args)
            patch_checks.rename(moved)
            patch_checks.symlink_to(outside, target_is_directory=True)
            return applied

        with mock.patch(
            "apply_candidate_patch._strict_apply",
            side_effect=apply_then_swap_output_root,
        ):
            with self.assertRaisesRegex(PatchApplicationError,
                                        "patch-check destination changed"):
                self.run_patch()

        result = json.loads((moved / "check-1/result.json").read_text())
        self.assertEqual(result["status"], "rejected")
        self.assertTrue(patch_checks.is_symlink())
        self.assertFalse((outside / "check-1").exists())

    def test_schema_two_applies_validated_patch_bytes_after_path_swap(self) -> None:
        validated_patch = replace_patch(OLD, NEW)
        replacement_patch = replace_patch(OLD, MID)
        self.set_patch_series([
            ("01-scene.patch", validated_patch, OLD, NEW),
        ])
        patch_path = self.candidate_root / "01-scene.patch"

        report, applied_patches = self.run_patch_with_path_swap(
            patch_path,
            validated_patch,
            replacement_patch,
        )

        self.assertEqual(report["status"], "passed")
        self.assertEqual(applied_patches, [validated_patch])
        self.assertFalse(patch_path.is_symlink())
        output = Path(str(report["output_package"])) / "src/scene.rs"
        self.assertEqual(output.read_bytes(), NEW)

    def test_schema_three_applies_validated_patch_bytes_after_path_swap(self) -> None:
        self.set_multi_file_series([
            ("01-scene.patch", "src/scene.rs", OLD, NEW),
            ("02-other.patch", "src/other.rs", b"old-other\n", b"new-other\n"),
        ])
        patch_path = self.candidate_root / "01-scene.patch"
        validated_patch = patch_path.read_bytes()
        replacement_patch = replace_package_file_patch("src/scene.rs", OLD, MID)

        report, applied_patches = self.run_patch_with_path_swap(
            patch_path,
            validated_patch,
            replacement_patch,
        )

        self.assertEqual(report["status"], "passed")
        self.assertEqual(applied_patches[0], validated_patch)
        self.assertFalse(patch_path.is_symlink())
        output = Path(str(report["output_package"]))
        self.assertEqual((output / "src/scene.rs").read_bytes(), NEW)
        self.assertEqual((output / "src/other.rs").read_bytes(), b"new-other\n")

    def test_inherited_descriptors_keep_output_under_pinned_candidate_after_parent_swap(self) -> None:
        candidates_parent = self.repo / "target/headstart/candidates"
        pinned_parent_path = candidates_parent.with_name("candidates-pinned")
        decoy_parent = self.repo / "decoy-candidates"
        decoy_parent.mkdir()
        self.manifest["snapshots"]["baseline"]["source"] = "baseline/source"
        self.manifest["snapshots"]["candidate"]["source"] = "candidate/source"
        self.write_manifest()

        repo_fd = os.open(self.repo, os.O_RDONLY | os.O_DIRECTORY)
        parent_fd = os.open(candidates_parent, os.O_RDONLY | os.O_DIRECTORY)
        candidate_fd = os.open(self.candidate_root, os.O_RDONLY | os.O_DIRECTORY)
        environment = {
            "RUSTUI_P3_REPO_ROOT_FD": str(repo_fd),
            "RUSTUI_P3_CANDIDATE_PARENT_FD": str(parent_fd),
            "RUSTUI_P3_CANDIDATE_ROOT_FD": str(candidate_fd),
            "RUSTUI_P3_CANDIDATE_ID": "sample-candidate",
        }
        try:
            candidates_parent.rename(pinned_parent_path)
            candidates_parent.symlink_to(decoy_parent, target_is_directory=True)
            with mock.patch.dict(os.environ, environment, clear=False):
                report = apply_candidate_patch(
                    Path("target/headstart/candidates/sample-candidate/manifest.json"),
                    "fd-pinned-check",
                    self.repo,
                )
            output = pinned_parent_path / "sample-candidate/patch-checks/fd-pinned-check/package"
            self.assertEqual(report["status"], "passed")
            self.assertEqual((output / "src/scene.rs").read_bytes(), NEW)
            self.assertEqual(list(decoy_parent.iterdir()), [])
        finally:
            os.close(candidate_fd)
            os.close(parent_fd)
            os.close(repo_fd)

    def test_inherited_candidate_id_cannot_escape_candidates_root(self) -> None:
        candidates_parent = self.candidate_root.parent
        escaped_candidate = candidates_parent.parent / "decoy-candidate"
        shutil.copytree(self.candidate_root, escaped_candidate)
        escaped_manifest = escaped_candidate / "manifest.json"
        manifest = json.loads(escaped_manifest.read_text())
        manifest["candidate_id"] = "../decoy-candidate"
        manifest["snapshots"]["baseline"]["source"] = "baseline/source"
        manifest["snapshots"]["candidate"]["source"] = "candidate/source"
        escaped_manifest.write_text(json.dumps(manifest), encoding="utf-8")

        repo_fd = os.open(self.repo, os.O_RDONLY | os.O_DIRECTORY)
        parent_fd = os.open(candidates_parent, os.O_RDONLY | os.O_DIRECTORY)
        candidate_fd = os.open(escaped_candidate, os.O_RDONLY | os.O_DIRECTORY)
        environment = {
            "RUSTUI_P3_REPO_ROOT_FD": str(repo_fd),
            "RUSTUI_P3_CANDIDATE_PARENT_FD": str(parent_fd),
            "RUSTUI_P3_CANDIDATE_ROOT_FD": str(candidate_fd),
            "RUSTUI_P3_CANDIDATE_ID": "../decoy-candidate",
        }
        try:
            with mock.patch.dict(os.environ, environment, clear=False):
                with self.assertRaisesRegex(PatchApplicationError, "identity is unsafe"):
                    apply_candidate_patch(
                        Path("target/headstart/candidates/../decoy-candidate/manifest.json"),
                        "escape-check",
                        self.repo,
                    )
        finally:
            os.close(candidate_fd)
            os.close(parent_fd)
            os.close(repo_fd)

        self.assertFalse((escaped_candidate / "patch-checks").exists())

    def test_applies_ordered_hash_pinned_patch_series(self) -> None:
        first = replace_patch(OLD, MID)
        second = replace_patch(MID, NEW)
        self.set_patch_series([
            ("01-first.patch", first, OLD, MID),
            ("02-second.patch", second, MID, NEW),
        ])

        report = self.run_patch()

        self.assertEqual(report["status"], "passed")
        self.assertEqual(len(report["patches"]), 2)
        self.assertIn("ordered patch series", report["scope"])
        output = Path(str(report["output_package"])) / "src/scene.rs"
        self.assertEqual(output.read_bytes(), NEW)
        self.assertEqual((self.baseline_package / "src/scene.rs").read_bytes(), OLD)
        self.assertEqual((self.candidate_package / "src/scene.rs").read_bytes(), NEW)

    def test_patch_series_caches_verified_bodies_outside_metadata(self) -> None:
        first = replace_patch(OLD, MID)
        second = replace_patch(MID, NEW)
        self.set_patch_series([
            ("01-first.patch", first, OLD, MID),
            ("02-second.patch", second, MID, NEW),
        ])
        module = sys.modules["apply_candidate_patch"]
        original_temporary_directory = tempfile.TemporaryDirectory
        cache_dirs: list[Path] = []

        def track_cache(*args: object, **kwargs: object) -> tempfile.TemporaryDirectory:
            cache = original_temporary_directory(*args, **kwargs)
            cache_dirs.append(Path(cache.name))
            return cache

        with mock.patch.object(
            module.tempfile, "TemporaryDirectory", side_effect=track_cache
        ):
            patches, cache = _patch_series(self.manifest, self.candidate_root.resolve())
            try:
                self.assertEqual(len(patches), 2)
                self.assertTrue(all("contents" not in patch for patch in patches))
                self.assertEqual(
                    [patch["cache_path"].read_bytes() for patch in patches],
                    [first, second],
                )
            finally:
                cache.cleanup()
        self.assertEqual(len(cache_dirs), 1)
        self.assertFalse(cache_dirs[0].exists())

    def test_apply_failure_closes_cached_patch_stream_and_removes_cache(self) -> None:
        self.set_patch_series([("01-first.patch", replace_patch(OLD, NEW), OLD, NEW)])
        module = sys.modules["apply_candidate_patch"]
        original_temporary_directory = module.tempfile.TemporaryDirectory
        original_path_open = Path.open
        cache_dirs: list[Path] = []
        cached_streams = []

        def capture_cache(*args: object, **kwargs: object) -> tempfile.TemporaryDirectory:
            cache = original_temporary_directory(*args, **kwargs)
            cache_dirs.append(Path(cache.name))
            return cache

        def capture_cached_stream(path: Path, mode: str = "r", *args: object,
                                  **kwargs: object):
            stream = original_path_open(path, mode, *args, **kwargs)
            if path.parent.name.startswith("rustui-p3k-patches-") and mode == "rb":
                cached_streams.append(stream)
            return stream

        with mock.patch.object(
            module.tempfile, "TemporaryDirectory", side_effect=capture_cache
        ), mock.patch.object(Path, "open", new=capture_cached_stream), mock.patch(
            "apply_candidate_patch._strict_apply",
            side_effect=OSError("injected patch application failure"),
        ):
            with self.assertRaisesRegex(PatchApplicationError, "injected patch application failure"):
                self.run_patch()

        self.assertEqual(len(cache_dirs), 1)
        self.assertFalse(cache_dirs[0].exists())
        self.assertEqual(len(cached_streams), 1)
        self.assertTrue(cached_streams[0].closed)

    def test_patch_series_closes_owned_descriptor_if_cache_creation_fails(self) -> None:
        module = sys.modules["apply_candidate_patch"]
        opened: list[int] = []
        original_open = module._open_directory

        def capture_open(path: Path) -> int:
            descriptor = original_open(path)
            opened.append(descriptor)
            return descriptor

        def assert_closed(descriptor: int) -> None:
            try:
                os.fstat(descriptor)
            except OSError:
                return
            os.close(descriptor)
            self.fail("_patch_series leaked its candidate directory descriptor")

        record = {"schema_version": module.SCHEMA_VERSION}
        candidate_root = self.candidate_root.resolve()
        with mock.patch.object(module, "_open_directory", side_effect=capture_open), mock.patch.object(
            module.tempfile,
            "TemporaryDirectory",
            side_effect=OSError("injected cache creation failure"),
        ):
            with self.assertRaisesRegex(OSError, "injected cache creation failure"):
                _patch_series(record, candidate_root)
        self.assertEqual(len(opened), 1)
        assert_closed(opened[0])

        pinned_fd = os.open(candidate_root, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
        duplicated: list[int] = []
        original_dup = module.os.dup

        def capture_dup(descriptor: int) -> int:
            duplicate = original_dup(descriptor)
            duplicated.append(duplicate)
            return duplicate

        try:
            with mock.patch.object(module.os, "dup", side_effect=capture_dup), mock.patch.object(
                module.tempfile,
                "TemporaryDirectory",
                side_effect=OSError("injected cache creation failure"),
            ):
                with self.assertRaisesRegex(OSError, "injected cache creation failure"):
                    _patch_series(record, candidate_root, pinned_fd)
            self.assertEqual(len(duplicated), 1)
            assert_closed(duplicated[0])
            os.fstat(pinned_fd)
        finally:
            os.close(pinned_fd)

    def test_patch_cache_cleanup_failure_still_closes_apply_descriptors(self) -> None:
        self.set_patch_series([("01-first.patch", replace_patch(OLD, NEW), OLD, NEW)])
        module = sys.modules["apply_candidate_patch"]
        original_patch_series = module._patch_series
        original_verify_directory = module._verify_directory_path
        patch_caches: list[tempfile.TemporaryDirectory] = []
        output_fds: list[int] = []

        def capture_cache(*args: object, **kwargs: object):
            patches, cache = original_patch_series(*args, **kwargs)
            cache.cleanup = mock.Mock(
                side_effect=OSError("injected patch-cache cleanup failure")
            )
            patch_caches.append(cache)
            return patches, cache

        def capture_output_fd(path: Path, descriptor: int, label: str) -> None:
            if label == "patch-check output":
                output_fds.append(descriptor)
            original_verify_directory(path, descriptor, label)

        try:
            with mock.patch.object(module, "_patch_series", side_effect=capture_cache), \
                    mock.patch.object(
                        module,
                        "_verify_directory_path",
                        side_effect=capture_output_fd,
                    ):
                with self.assertRaisesRegex(OSError, "injected patch-cache cleanup failure"):
                    self.run_patch()

            self.assertEqual(len(output_fds), 1)
            with self.assertRaises(OSError):
                os.fstat(output_fds[0])
        finally:
            for cache in patch_caches:
                tempfile.TemporaryDirectory.cleanup(cache)
            for descriptor in output_fds:
                try:
                    os.close(descriptor)
                except OSError:
                    pass

    def test_oversized_patch_is_rejected_before_patch_copy_creation(self) -> None:
        patch = replace_patch(OLD, NEW)
        self.set_patch_series([("01-large.patch", patch, OLD, NEW)])

        with mock.patch("apply_candidate_patch.MAX_PATCH_BYTES", len(patch) - 1):
            with self.assertRaisesRegex(PatchApplicationError, "patch exceeds .* byte limit"):
                self.run_patch()

        self.assertFalse((self.candidate_root / "patch-checks/check-1").exists())

    def test_rejects_out_of_order_patch_series_at_each_patch_preimage(self) -> None:
        first = replace_patch(OLD, MID)
        second = replace_patch(MID, NEW)
        self.set_patch_series([
            ("01-first.patch", second, OLD, MID),
            ("02-second.patch", first, MID, NEW),
        ])

        with self.assertRaisesRegex(PatchApplicationError, "exact preimage mismatch"):
            self.run_patch()

        result = json.loads(
            (self.candidate_root / "patch-checks/check-1/result.json").read_text()
        )
        self.assertEqual(result["status"], "rejected")
        self.assertEqual((self.baseline_package / "src/scene.rs").read_bytes(), OLD)
        self.assertEqual((self.candidate_package / "src/scene.rs").read_bytes(), NEW)

    def test_rejects_patch_series_entry_hash_mismatch_before_copy(self) -> None:
        first = replace_patch(OLD, MID)
        second = replace_patch(MID, NEW)
        self.set_patch_series([
            ("01-first.patch", first, OLD, MID),
            ("02-second.patch", second, MID, NEW),
        ])
        self.manifest["patches"][1]["sha256"] = "0" * 64
        self.write_manifest()
        module = sys.modules["apply_candidate_patch"]
        original_temporary_directory = module.tempfile.TemporaryDirectory
        cache_dirs: list[Path] = []

        def capture_cache(*args: object, **kwargs: object) -> tempfile.TemporaryDirectory:
            cache = original_temporary_directory(*args, **kwargs)
            cache_dirs.append(Path(cache.name))
            return cache

        with mock.patch.object(
            module.tempfile, "TemporaryDirectory", side_effect=capture_cache
        ):
            with self.assertRaisesRegex(PatchApplicationError, "patch hash differs"):
                self.run_patch()

        self.assertEqual([path.exists() for path in cache_dirs], [False])
        self.assertFalse((self.candidate_root / "patch-checks/check-1").exists())

    def test_rejects_unknown_patch_series_entry_fields(self) -> None:
        first = replace_patch(OLD, MID)
        second = replace_patch(MID, NEW)
        self.set_patch_series([
            ("01-first.patch", first, OLD, MID),
            ("02-second.patch", second, MID, NEW),
        ])
        self.manifest["patches"][0]["unexpected"] = "typo"
        self.write_manifest()

        with self.assertRaisesRegex(PatchApplicationError, "unknown fields"):
            self.run_patch()
        self.assertFalse((self.candidate_root / "patch-checks/check-1").exists())

    def test_rejects_intermediate_postimage_mismatch_and_retains_failed_copy(self) -> None:
        first = replace_patch(OLD, MID)
        second = replace_patch(MID, NEW)
        self.set_patch_series([
            ("01-first.patch", first, OLD, NEW),
            ("02-second.patch", second, MID, NEW),
        ])

        with self.assertRaisesRegex(PatchApplicationError, "patch postimage hash"):
            self.run_patch()

        run_root = self.candidate_root / "patch-checks/check-1"
        result = json.loads((run_root / "result.json").read_text())
        self.assertEqual(result["status"], "rejected")
        self.assertEqual((run_root / "package/src/scene.rs").read_bytes(), MID)
        self.assertEqual((self.baseline_package / "src/scene.rs").read_bytes(), OLD)
        self.assertEqual((self.candidate_package / "src/scene.rs").read_bytes(), NEW)

    def test_postcheck_failure_keeps_rejection_result(self) -> None:
        original_manifest = self.manifest_path

        def change_manifest_after_read(source: bytes, patch: bytes) -> bytes:
            original_manifest.write_bytes(original_manifest.read_bytes() + b" ")
            return _strict_apply(source, patch)

        with mock.patch(
            "apply_candidate_patch._strict_apply",
            side_effect=change_manifest_after_read,
        ):
            with self.assertRaisesRegex(PatchApplicationError, "candidate manifest changed"):
                self.run_patch()

        run_root = self.candidate_root / "patch-checks/check-1"
        result = json.loads((run_root / "result.json").read_text())
        self.assertEqual(result["status"], "rejected")
        self.assertIn("candidate manifest changed", result["error"])
        self.assertEqual((run_root / "package/src/scene.rs").read_bytes(), NEW)

    def test_schema_three_applies_ordered_multi_file_patches_with_file_hash_evidence(self) -> None:
        self.set_multi_file_series([
            ("01-scene.patch", "src/scene.rs", OLD, NEW),
            ("02-other.patch", "src/other.rs", b"old-other\n", b"new-other\n"),
        ])

        report = self.run_patch()

        self.assertEqual(report["status"], "passed")
        self.assertEqual(report["rollback_on_failure"], True)
        self.assertEqual(report["rollback_status"], "not_required")
        self.assertEqual(report["files"], [
            {"path": "src/other.rs", "preimage_sha256": digest(b"old-other\n"),
             "postimage_sha256": digest(b"new-other\n")},
            {"path": "src/scene.rs", "preimage_sha256": digest(OLD),
             "postimage_sha256": digest(NEW)},
        ])
        self.assertEqual(len(report["patches"]), 2)
        self.assertEqual(report["patches"][0]["file"], "src/scene.rs")
        output = Path(str(report["output_package"]))
        self.assertEqual((output / "src/scene.rs").read_bytes(), NEW)
        self.assertEqual((output / "src/other.rs").read_bytes(), b"new-other\n")

    def test_schema_three_rejects_late_failure_and_rolls_back_candidate_copy(self) -> None:
        self.set_multi_file_series([
            ("01-scene.patch", "src/scene.rs", OLD, NEW),
            ("02-other.patch", "src/other.rs", b"old-other\n", b"new-other\n"),
        ])
        bad_patch = (self.candidate_root / "02-other.patch").read_bytes().replace(
            b"-old-other\n", b"-unexpected\n"
        )
        (self.candidate_root / "02-other.patch").write_bytes(bad_patch)
        self.manifest["patches"][1]["sha256"] = digest(bad_patch)
        descriptors = [f"{entry['path']}:{entry['sha256']}" for entry in self.manifest["patches"]]
        self.manifest["patch_sha256"] = digest("\n".join(descriptors).encode())
        self.write_manifest()
        module = sys.modules["apply_candidate_patch"]
        original_temporary_directory = module.tempfile.TemporaryDirectory
        cache_dirs: list[Path] = []

        def capture_cache(*args: object, **kwargs: object) -> tempfile.TemporaryDirectory:
            cache = original_temporary_directory(*args, **kwargs)
            cache_dirs.append(Path(cache.name))
            return cache

        with mock.patch.object(
            module.tempfile, "TemporaryDirectory", side_effect=capture_cache
        ):
            with self.assertRaisesRegex(PatchApplicationError, "exact preimage mismatch"):
                self.run_patch()

        self.assertEqual(len(cache_dirs), 1)
        self.assertFalse(cache_dirs[0].exists())
        run_root = self.candidate_root / "patch-checks/check-1"
        result = json.loads((run_root / "result.json").read_text())
        self.assertEqual(result["status"], "rejected")
        self.assertIsNone(result["output_package"])
        self.assertTrue(result["rollback_on_failure"])
        self.assertEqual(result["rollback_status"], "completed")
        self.assertFalse((run_root / "package").exists())
        self.assertEqual((self.baseline_package / "src/scene.rs").read_bytes(), OLD)
        self.assertEqual((self.candidate_package / "src/scene.rs").read_bytes(), NEW)

    def test_schema_three_rollback_preserves_replaced_package_directory(self) -> None:
        self.set_multi_file_series([
            ("01-scene.patch", "src/scene.rs", OLD, NEW),
            ("02-other.patch", "src/other.rs", b"old-other\n", b"new-other\n"),
        ])
        bad_patch = (self.candidate_root / "02-other.patch").read_bytes().replace(
            b"-old-other\n", b"-unexpected\n"
        )
        (self.candidate_root / "02-other.patch").write_bytes(bad_patch)
        self.manifest["patches"][1]["sha256"] = digest(bad_patch)
        descriptors = [f"{entry['path']}:{entry['sha256']}" for entry in self.manifest["patches"]]
        self.manifest["patch_sha256"] = digest("\n".join(descriptors).encode())
        self.write_manifest()

        run_root = self.candidate_root / "patch-checks/check-1"
        module = sys.modules["apply_candidate_patch"]
        original_remove = module._remove_tree_at

        def replace_package_then_remove(parent_fd, name, expected_identity=None):
            if name == "package":
                package = run_root / "package"
                package.rename(run_root / "package-detached")
                package.mkdir()
                (package / "replacement.txt").write_text("preserve me")
            return original_remove(parent_fd, name, expected_identity)

        with mock.patch("apply_candidate_patch._remove_tree_at", side_effect=replace_package_then_remove):
            with self.assertRaisesRegex(PatchApplicationError, "exact preimage mismatch"):
                self.run_patch()

        result = json.loads((run_root / "result.json").read_text())
        self.assertEqual(result["rollback_status"], "failed")
        self.assertIn("directory changed during rollback", result["error"])
        self.assertEqual((run_root / "package/replacement.txt").read_text(), "preserve me")
        self.assertTrue((run_root / "package-detached/src/scene.rs").exists())

    def test_schema_three_rejects_broken_per_file_hash_chain_before_copy(self) -> None:
        middle = b"middle\nkeep\n"
        self.set_multi_file_series([
            ("01-scene.patch", "src/scene.rs", OLD, middle),
            ("02-scene.patch", "src/scene.rs", middle, NEW),
            ("03-other.patch", "src/other.rs", b"old-other\n", b"new-other\n"),
        ])
        # The second patch's declared preimage must be the first patch's postimage.
        self.manifest["patches"][1]["preimage_sha256"] = digest(OLD)
        self.write_manifest()

        with self.assertRaisesRegex(PatchApplicationError, "patch preimage hash differs"):
            self.run_patch()

        self.assertFalse((self.candidate_root / "patch-checks/check-1").exists())
        self.assertEqual((self.baseline_package / "src/scene.rs").read_bytes(), OLD)
        self.assertEqual((self.candidate_package / "src/scene.rs").read_bytes(), NEW)

    def test_schema_three_rolls_back_after_staged_package_write_failure(self) -> None:
        self.set_multi_file_series([
            ("01-scene.patch", "src/scene.rs", OLD, NEW),
            ("02-other.patch", "src/other.rs", b"old-other\n", b"new-other\n"),
        ])
        run_root = self.candidate_root / "patch-checks/check-1"
        module = sys.modules["apply_candidate_patch"]
        original_write = module._write_file_at

        def fail_second_package_write(root_fd: int, relative: str, contents: bytes) -> None:
            if relative == "src/other.rs":
                raise OSError("injected package write failure")
            original_write(root_fd, relative, contents)

        with mock.patch(
            "apply_candidate_patch._write_file_at", side_effect=fail_second_package_write
        ):
            with self.assertRaisesRegex(PatchApplicationError, "injected package write failure"):
                self.run_patch()

        result = json.loads((run_root / "result.json").read_text())
        self.assertEqual(result["status"], "rejected")
        self.assertEqual(result["rollback_status"], "completed")
        self.assertIsNone(result["output_package"])
        self.assertFalse((run_root / "package").exists())
        self.assertEqual((self.baseline_package / "src/scene.rs").read_bytes(), OLD)
        self.assertEqual((self.candidate_package / "src/scene.rs").read_bytes(), NEW)

    def test_pinned_schema_three_omits_output_after_successful_rollback(self) -> None:
        self.set_multi_file_series([
            ("01-scene.patch", "src/scene.rs", OLD, NEW),
            ("02-other.patch", "src/other.rs", b"old-other\n", b"new-other\n"),
        ])
        self.manifest["snapshots"]["baseline"]["source"] = "baseline/source"
        self.manifest["snapshots"]["candidate"]["source"] = "candidate/source"
        self.write_manifest()
        repo_fd = os.open(self.repo, os.O_RDONLY | os.O_DIRECTORY)
        parent_fd = os.open(self.candidate_root.parent, os.O_RDONLY | os.O_DIRECTORY)
        candidate_fd = os.open(self.candidate_root, os.O_RDONLY | os.O_DIRECTORY)
        environment = {
            "RUSTUI_P3_REPO_ROOT_FD": str(repo_fd),
            "RUSTUI_P3_CANDIDATE_PARENT_FD": str(parent_fd),
            "RUSTUI_P3_CANDIDATE_ROOT_FD": str(candidate_fd),
            "RUSTUI_P3_CANDIDATE_ID": "sample-candidate",
        }
        run_root = self.candidate_root / "patch-checks/pinned-rollback"
        module = sys.modules["apply_candidate_patch"]
        original_write = module._write_file_at

        def fail_second_package_write(root_fd: int, relative: str, contents: bytes) -> None:
            if relative == "src/other.rs":
                raise OSError("injected package write failure")
            original_write(root_fd, relative, contents)

        try:
            with mock.patch.dict(os.environ, environment, clear=False), mock.patch(
                "apply_candidate_patch._write_file_at", side_effect=fail_second_package_write
            ):
                with self.assertRaisesRegex(PatchApplicationError, "injected package write failure"):
                    apply_candidate_patch(
                        Path("target/headstart/candidates/sample-candidate/manifest.json"),
                        "pinned-rollback",
                        self.repo,
                    )
        finally:
            os.close(candidate_fd)
            os.close(parent_fd)
            os.close(repo_fd)

        result = json.loads((run_root / "result.json").read_text())
        self.assertEqual(result["status"], "rejected")
        self.assertEqual(result["rollback_status"], "completed")
        self.assertIsNone(result["output_package"])
        self.assertFalse((run_root / "package").exists())

    def test_schema_three_rejects_patch_file_changed_during_application(self) -> None:
        self.set_multi_file_series([
            ("01-scene.patch", "src/scene.rs", OLD, NEW),
            ("02-other.patch", "src/other.rs", b"old-other\n", b"new-other\n"),
        ])
        patch_path = self.candidate_root / "01-scene.patch"
        module = sys.modules["apply_candidate_patch"]
        original_apply = module._strict_apply
        original_read = module._read_file_at

        def apply_then_change_patch(source: bytes, patch: bytes, *args: object) -> bytes:
            applied = original_apply(source, patch, *args)
            patch_path.write_bytes(patch_path.read_bytes() + b"# changed after validation\n")
            return applied

        with (
            mock.patch("apply_candidate_patch._strict_apply", side_effect=apply_then_change_patch),
            mock.patch("apply_candidate_patch._read_file_at", wraps=original_read) as read_file,
        ):
            with self.assertRaisesRegex(
                PatchApplicationError, "approved patch series changed during patch check"
            ):
                self.run_patch()
        self.assertNotIn(
            patch_path.name,
            [Path(str(call.args[1])).name for call in read_file.call_args_list],
        )

        run_root = self.candidate_root / "patch-checks/check-1"
        result = json.loads((run_root / "result.json").read_text())
        self.assertEqual(result["status"], "rejected")
        self.assertEqual(result["rollback_status"], "completed")
        self.assertIn("approved patch series changed during patch check", result["error"])
        self.assertFalse((run_root / "package").exists())

    def test_schema_three_rejects_path_traversal_before_copy(self) -> None:
        self.set_multi_file_series([
            ("01-scene.patch", "src/scene.rs", OLD, NEW),
            ("02-other.patch", "src/other.rs", b"old-other\n", b"new-other\n"),
        ])
        self.manifest["patches"][1]["file"] = "../outside.rs"
        self.write_manifest()

        with self.assertRaisesRegex(PatchApplicationError, "safe package-relative"):
            self.run_patch()
        self.assertFalse((self.candidate_root / "patch-checks/check-1").exists())

    def test_schema_three_rejects_symlinked_patch_checks_root(self) -> None:
        self.set_multi_file_series([
            ("01-scene.patch", "src/scene.rs", OLD, NEW),
            ("02-other.patch", "src/other.rs", b"old-other\n", b"new-other\n"),
        ])
        outside = self.repo / "outside-output"
        outside.mkdir()
        patch_checks = self.candidate_root / "patch-checks"
        patch_checks.symlink_to(outside, target_is_directory=True)

        with self.assertRaisesRegex(PatchApplicationError, "destination root cannot be a symlink"):
            self.run_patch("symlink-escape")

        self.assertEqual(list(outside.iterdir()), [])
        self.assertTrue(patch_checks.is_symlink())

    def test_schema_three_rejects_patch_checks_symlink_swap_before_open(self) -> None:
        self.set_multi_file_series([
            ("01-scene.patch", "src/scene.rs", OLD, NEW),
            ("02-other.patch", "src/other.rs", b"old-other\n", b"new-other\n"),
        ])
        outside = self.repo / "outside-output-race"
        outside.mkdir()
        patch_checks = self.candidate_root / "patch-checks"
        original_open_or_create = sys.modules["apply_candidate_patch"]._open_or_create_directory_at

        def swap_before_open(parent_fd: int, name: str, label: str) -> int:
            if name == "patch-checks":
                patch_checks.symlink_to(outside, target_is_directory=True)
            return original_open_or_create(parent_fd, name, label)

        with mock.patch(
            "apply_candidate_patch._open_or_create_directory_at",
            side_effect=swap_before_open,
        ):
            with self.assertRaisesRegex(
                PatchApplicationError, "destination root cannot be a symlink"
            ):
                self.run_patch("symlink-race")

        self.assertTrue(patch_checks.is_symlink())
        self.assertEqual(list(outside.iterdir()), [])
        self.assertFalse((outside / "symlink-race").exists())

    def test_schema_three_rejects_diff_header_for_another_file_and_rolls_back(self) -> None:
        self.set_multi_file_series([
            ("01-scene.patch", "src/scene.rs", OLD, NEW),
            ("02-other.patch", "src/other.rs", b"old-other\n", b"new-other\n"),
        ])
        patch_path = self.candidate_root / "01-scene.patch"
        wrong_target = patch_path.read_bytes().replace(b"src/scene.rs", b"src/other.rs")
        patch_path.write_bytes(wrong_target)
        self.manifest["patches"][0]["sha256"] = digest(wrong_target)
        descriptors = [f"{entry['path']}:{entry['sha256']}" for entry in self.manifest["patches"]]
        self.manifest["patch_sha256"] = digest("\n".join(descriptors).encode())
        self.write_manifest()

        with self.assertRaisesRegex(PatchApplicationError, "patch must target only package file"):
            self.run_patch()

        run_root = self.candidate_root / "patch-checks/check-1"
        result = json.loads((run_root / "result.json").read_text())
        self.assertEqual(result["status"], "rejected")
        self.assertEqual(result["rollback_status"], "completed")
        self.assertFalse((run_root / "package").exists())

    def test_pinned_preflight_rejection_closes_package_descriptors(self) -> None:
        module = sys.modules["apply_candidate_patch"]
        self.manifest["snapshots"]["baseline"]["source"] = "baseline/source"
        self.manifest["snapshots"]["candidate"]["source"] = "candidate/source"
        self.manifest["snapshots"]["baseline"]["path_tree_sha256"] = "0" * 64
        self.write_manifest()

        repo_fd = os.open(self.repo, os.O_RDONLY | os.O_DIRECTORY)
        parent_fd = os.open(self.candidate_root.parent, os.O_RDONLY | os.O_DIRECTORY)
        candidate_fd = os.open(self.candidate_root, os.O_RDONLY | os.O_DIRECTORY)
        environment = {
            "RUSTUI_P3_REPO_ROOT_FD": str(repo_fd),
            "RUSTUI_P3_CANDIDATE_PARENT_FD": str(parent_fd),
            "RUSTUI_P3_CANDIDATE_ROOT_FD": str(candidate_fd),
            "RUSTUI_P3_CANDIDATE_ID": "sample-candidate",
        }
        opened: list[int] = []
        original_open = module._open_directory_at

        def capture_open(parent: int, relative: str) -> int:
            descriptor = original_open(parent, relative)
            if relative.endswith(f"path-sources/{PACKAGE}"):
                opened.append(descriptor)
            return descriptor

        try:
            with mock.patch.dict(os.environ, environment, clear=True), mock.patch(
                "apply_candidate_patch._open_directory_at", side_effect=capture_open
            ):
                with self.assertRaisesRegex(PatchApplicationError, "baseline package tree"):
                    apply_candidate_patch(
                        Path("target/headstart/candidates/sample-candidate/manifest.json"),
                        "preflight-rejection",
                        self.repo,
                    )
            for descriptor in opened:
                with self.assertRaises(OSError):
                    os.fstat(descriptor)
        finally:
            for descriptor in opened:
                try:
                    os.close(descriptor)
                except OSError:
                    pass
            os.close(candidate_fd)
            os.close(parent_fd)
            os.close(repo_fd)

    def test_pinned_patch_check_setup_failure_closes_package_descriptors(self) -> None:
        module = sys.modules["apply_candidate_patch"]
        self.manifest["snapshots"]["baseline"]["source"] = "baseline/source"
        self.manifest["snapshots"]["candidate"]["source"] = "candidate/source"
        self.write_manifest()

        repo_fd = os.open(self.repo, os.O_RDONLY | os.O_DIRECTORY)
        parent_fd = os.open(self.candidate_root.parent, os.O_RDONLY | os.O_DIRECTORY)
        candidate_fd = os.open(self.candidate_root, os.O_RDONLY | os.O_DIRECTORY)
        environment = {
            "RUSTUI_P3_REPO_ROOT_FD": str(repo_fd),
            "RUSTUI_P3_CANDIDATE_PARENT_FD": str(parent_fd),
            "RUSTUI_P3_CANDIDATE_ROOT_FD": str(candidate_fd),
            "RUSTUI_P3_CANDIDATE_ID": "sample-candidate",
        }
        opened: list[int] = []
        original_open = module._open_directory_at

        def capture_open(parent: int, relative: str) -> int:
            descriptor = original_open(parent, relative)
            if relative.endswith(f"path-sources/{PACKAGE}"):
                opened.append(descriptor)
            return descriptor

        try:
            with mock.patch.dict(os.environ, environment, clear=True), mock.patch(
                "apply_candidate_patch._open_directory_at", side_effect=capture_open
            ), mock.patch(
                "apply_candidate_patch._open_or_create_directory_at",
                side_effect=PatchApplicationError("simulated patch-check setup failure"),
            ):
                with self.assertRaisesRegex(PatchApplicationError, "setup failure"):
                    apply_candidate_patch(
                        Path("target/headstart/candidates/sample-candidate/manifest.json"),
                        "setup-rejection",
                        self.repo,
                    )
            for descriptor in opened:
                with self.assertRaises(OSError):
                    os.fstat(descriptor)
        finally:
            for descriptor in opened:
                try:
                    os.close(descriptor)
                except OSError:
                    pass
            os.close(candidate_fd)
            os.close(parent_fd)
            os.close(repo_fd)


if __name__ == "__main__":
    unittest.main()
