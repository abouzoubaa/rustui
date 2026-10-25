from __future__ import annotations

import hashlib
import os
import resource
import stat
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from p3_candidate_fs import CandidateFSError, PinnedDir, relative_parts


def require_pinned_fs(test: unittest.TestCase) -> None:
    if not hasattr(os, "O_NOFOLLOW") or os.open not in os.supports_dir_fd:
        test.skipTest("requires O_NOFOLLOW and descriptor-relative os.open")


class CandidateFilesystemTests(unittest.TestCase):
    def setUp(self) -> None:
        require_pinned_fs(self)

    def test_rename_invalid_destination_closes_source_parent(self) -> None:
        with tempfile.TemporaryDirectory() as temporary, PinnedDir.open_root(temporary) as root:
            opened = []
            original_parent_and_name = root._parent_and_name

            def record_parent(relative):
                result = original_parent_and_name(relative)
                opened.append(result[0])
                return result

            with patch.object(root, "_parent_and_name", side_effect=record_parent):
                with self.assertRaises(CandidateFSError):
                    root.rename("source", root, "../escape")
            self.assertEqual(len(opened), 1)
            try:
                with self.assertRaises(OSError):
                    os.fstat(opened[0])
            finally:
                # Also release the leaked descriptor when this regression fails.
                try:
                    os.close(opened[0])
                except OSError:
                    pass

    def test_relative_parts_rejects_unsafe_forms(self) -> None:
        self.assertEqual(relative_parts("a/b.txt"), ("a", "b.txt"))
        for value in ("", ".", "a/.", "a/..", "../a", "/a", "C:/a", "a\\b", "a//b", "a/"):
            with self.subTest(value=value), self.assertRaises(CandidateFSError):
                relative_parts(value)

    def test_open_and_create_reject_symlinks_and_parent_swap(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            root_path, outside = base / "root", base / "outside"
            root_path.mkdir()
            outside.mkdir()
            (outside / "secret").write_bytes(b"outside")
            (root_path / "link").symlink_to(outside, target_is_directory=True)
            with PinnedDir.open_root(root_path) as root:
                for action in (lambda: PinnedDir.open_child(root, "link"),
                               lambda: PinnedDir.create_child(root, "link/created")):
                    with self.subTest(action=action), self.assertRaises(CandidateFSError):
                        action()
                with self.assertRaises(CandidateFSError):
                    root.open_file("link/secret", os.O_RDONLY)

                (root_path / "parent").mkdir()
                anchored = PinnedDir.open_child(root, "parent")
                try:
                    saved = root_path / "parent-saved"
                    (root_path / "parent").rename(saved)
                    (root_path / "parent").symlink_to(outside, target_is_directory=True)
                    anchored.atomic_write("written", b"anchored")
                    self.assertEqual((saved / "written").read_bytes(), b"anchored")
                    self.assertFalse((outside / "written").exists())
                finally:
                    anchored.close()

    def test_open_root_stays_anchored_after_rename_and_replacement(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            selected, moved, replacement = base / "selected", base / "moved", base / "selected"
            selected.mkdir()
            (selected / "value").write_bytes(b"original")
            root = PinnedDir.open_root(selected)
            try:
                selected.rename(moved)
                replacement.mkdir()
                (replacement / "value").write_bytes(b"replacement")
                self.assertEqual(root.read_bytes("value"), b"original")
                root.atomic_write("new", b"on original")
                self.assertEqual((moved / "new").read_bytes(), b"on original")
                self.assertFalse((replacement / "new").exists())
            finally:
                root.close()

    def test_atomic_write_replaces_and_refuses_existing_destination(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            with PinnedDir.open_root(temporary) as root:
                root.atomic_write("value", b"first")
                root.atomic_write("value", b"second")
                self.assertEqual(root.read_bytes("value"), b"second")
                with self.assertRaises(CandidateFSError):
                    root.atomic_write("value", b"third", replace=False)
                self.assertEqual(root.read_bytes("value"), b"second")
                root.atomic_write("new", b"created", replace=False)
                self.assertEqual(root.read_bytes("new"), b"created")

    def test_copy_and_remove_reject_links_without_following_them(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            source_path, destination_path, outside = base / "source", base / "destination", base / "outside"
            source_path.mkdir()
            destination_path.mkdir()
            outside.mkdir()
            (outside / "keep").write_bytes(b"outside")
            (source_path / "nested").mkdir()
            (source_path / "nested/file").write_bytes(b"contents")
            (source_path / "escape").symlink_to(outside, target_is_directory=True)
            with PinnedDir.open_root(source_path) as source, PinnedDir.open_root(destination_path) as destination:
                with self.assertRaisesRegex(CandidateFSError, "symlink"):
                    destination.copy_tree_from(source)
                destination.remove_tree("nested")
                self.assertFalse((outside / "keep").is_symlink())
                self.assertEqual((outside / "keep").read_bytes(), b"outside")
                source.unlink("escape")
                destination.copy_tree_from(source)
                self.assertEqual(destination.read_bytes("nested/file"), b"contents")
                destination.remove_tree("nested")
                self.assertEqual(destination.inventory(), {})

    def test_copy_tree_handles_deep_directory_chain_with_bounded_descriptors(self) -> None:
        depth = 1100
        directory_flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            source_path, destination_path = base / "source", base / "destination"
            source_path.mkdir()
            destination_path.mkdir()

            def remove_deep_chain(root: Path) -> None:
                root_fd = os.open(root, directory_flags)
                current_fd = os.dup(root_fd)
                levels = 0
                try:
                    while True:
                        try:
                            child_fd = os.open("d", directory_flags, dir_fd=current_fd)
                        except FileNotFoundError:
                            break
                        os.close(current_fd)
                        current_fd = child_fd
                        levels += 1
                    try:
                        os.unlink("leaf", dir_fd=current_fd)
                    except FileNotFoundError:
                        pass
                    for _ in range(levels):
                        parent_fd = os.open("..", directory_flags, dir_fd=current_fd)
                        os.rmdir("d", dir_fd=parent_fd)
                        os.close(current_fd)
                        current_fd = parent_fd
                finally:
                    os.close(current_fd)
                    os.close(root_fd)

            original_soft, hard_limit = resource.getrlimit(resource.RLIMIT_NOFILE)
            limit_changed = False
            active_fd = os.open(source_path, directory_flags)
            try:
                for _ in range(depth):
                    os.mkdir("d", dir_fd=active_fd)
                    child_fd = os.open("d", directory_flags, dir_fd=active_fd)
                    os.close(active_fd)
                    active_fd = child_fd
                file_fd = os.open("leaf", os.O_WRONLY | os.O_CREAT | os.O_EXCL,
                                  0o600, dir_fd=active_fd)
                try:
                    os.write(file_fd, b"x")
                finally:
                    os.close(file_fd)
            finally:
                os.close(active_fd)

            try:
                with PinnedDir.open_root(source_path) as source, \
                        PinnedDir.open_root(destination_path) as destination:
                    descriptor_count = len(os.listdir("/dev/fd"))
                    low_limit = max(32, descriptor_count + 8)
                    if low_limit >= depth:
                        self.skipTest("cannot set a descriptor limit far below the tree depth")
                    if hard_limit != resource.RLIM_INFINITY and hard_limit < low_limit:
                        self.skipTest("hard file descriptor limit is too low for the regression")
                    resource.setrlimit(resource.RLIMIT_NOFILE, (low_limit, hard_limit))
                    limit_changed = True

                    destination.copy_tree_from(source, max_entries=depth + 1, max_bytes=1)

                    current_fd = os.dup(destination.fd)
                    try:
                        for _ in range(depth):
                            child_fd = os.open("d", directory_flags, dir_fd=current_fd)
                            os.close(current_fd)
                            current_fd = child_fd
                        leaf_fd = os.open("leaf", os.O_RDONLY | os.O_NOFOLLOW,
                                          dir_fd=current_fd)
                        try:
                            self.assertEqual(os.read(leaf_fd, 2), b"x")
                        finally:
                            os.close(leaf_fd)
                    finally:
                        os.close(current_fd)
            finally:
                if limit_changed:
                    resource.setrlimit(resource.RLIMIT_NOFILE, (original_soft, hard_limit))
                remove_deep_chain(source_path)
                remove_deep_chain(destination_path)

    def test_inventory_handles_deep_directory_chain_with_bounded_descriptors(self) -> None:
        depth = 1100
        directory_flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
        with tempfile.TemporaryDirectory() as temporary:
            root_path = Path(temporary) / "root"
            root_path.mkdir()

            def remove_deep_chain(root: Path) -> None:
                root_fd = os.open(root, directory_flags)
                current_fd = os.dup(root_fd)
                levels = 0
                try:
                    while True:
                        try:
                            child_fd = os.open("d", directory_flags, dir_fd=current_fd)
                        except FileNotFoundError:
                            break
                        os.close(current_fd)
                        current_fd = child_fd
                        levels += 1
                    try:
                        os.unlink("leaf", dir_fd=current_fd)
                    except FileNotFoundError:
                        pass
                    for _ in range(levels):
                        parent_fd = os.open("..", directory_flags, dir_fd=current_fd)
                        os.rmdir("d", dir_fd=parent_fd)
                        os.close(current_fd)
                        current_fd = parent_fd
                finally:
                    os.close(current_fd)
                    os.close(root_fd)

            current_fd = os.open(root_path, directory_flags)
            try:
                for _ in range(depth):
                    os.mkdir("d", dir_fd=current_fd)
                    child_fd = os.open("d", directory_flags, dir_fd=current_fd)
                    os.close(current_fd)
                    current_fd = child_fd
                file_fd = os.open("leaf", os.O_WRONLY | os.O_CREAT | os.O_EXCL,
                                  0o600, dir_fd=current_fd)
                try:
                    os.write(file_fd, b"inventory")
                finally:
                    os.close(file_fd)
            finally:
                os.close(current_fd)

            original_soft, hard_limit = resource.getrlimit(resource.RLIMIT_NOFILE)
            limit_changed = False
            try:
                with PinnedDir.open_root(root_path) as root:
                    descriptor_count = len(os.listdir("/dev/fd"))
                    low_limit = max(32, descriptor_count + 8)
                    if low_limit >= depth:
                        self.skipTest("cannot set a descriptor limit far below the tree depth")
                    if hard_limit != resource.RLIM_INFINITY and hard_limit < low_limit:
                        self.skipTest("hard file descriptor limit is too low for the regression")
                    resource.setrlimit(resource.RLIMIT_NOFILE, (low_limit, hard_limit))
                    limit_changed = True
                    relative = "/".join(["d"] * depth + ["leaf"])
                    self.assertEqual(
                        root.inventory(max_entries=depth + 1, max_bytes=9),
                        {relative: hashlib.sha256(b"inventory").hexdigest()},
                    )
            finally:
                if limit_changed:
                    resource.setrlimit(resource.RLIMIT_NOFILE, (original_soft, hard_limit))
                remove_deep_chain(root_path)

    def test_inventory_enforces_budgets_and_rejects_symlinks_and_hard_links(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            root_path, outside = base / "root", base / "outside"
            root_path.mkdir()
            (root_path / "nested").mkdir()
            (root_path / "a").write_bytes(b"ab")
            (root_path / "nested" / "b").write_bytes(b"cde")
            outside.write_bytes(b"outside")

            with PinnedDir.open_root(root_path) as root:
                self.assertEqual(
                    root.inventory(max_entries=3, max_bytes=5),
                    {
                        "a": hashlib.sha256(b"ab").hexdigest(),
                        "nested/b": hashlib.sha256(b"cde").hexdigest(),
                    },
                )
                with self.assertRaisesRegex(CandidateFSError, "exceeds 2 entries"):
                    root.inventory(max_entries=2)
                with self.assertRaisesRegex(CandidateFSError, "exceeds 4 regular-file bytes"):
                    root.inventory(max_bytes=4)

                (root_path / "link").symlink_to(outside)
                with self.assertRaisesRegex(CandidateFSError, "symlink rejected: link"):
                    root.inventory()
                (root_path / "link").unlink()
                os.link(outside, root_path / "hard-link")
                with self.assertRaisesRegex(CandidateFSError, "hard-linked file rejected"):
                    root.inventory()

    def test_inventory_rejects_same_inode_file_mutation_during_hash(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            source_path = Path(temporary) / "source"
            source_path.mkdir()
            file_path = source_path / "payload"
            file_path.write_bytes(b"a" * (2 * 1024 * 1024))
            original_identity = file_path.stat().st_ino
            real_fdopen = os.fdopen

            def mutate_after_first_read(fd: int, mode: str):
                stream = real_fdopen(fd, mode)

                class MutatingReader:
                    mutated = False

                    def __enter__(self):
                        return self

                    def __exit__(self, *args):
                        return stream.__exit__(*args)

                    def fileno(self) -> int:
                        return stream.fileno()

                    def read(self, size: int = -1) -> bytes:
                        block = stream.read(size)
                        if block and not self.mutated:
                            with file_path.open("r+b") as writer:
                                writer.write(b"x")
                            self.mutated = True
                        return block

                return MutatingReader()

            with PinnedDir.open_root(source_path) as source:
                with patch.object(os, "fdopen", side_effect=mutate_after_first_read):
                    with self.assertRaisesRegex(CandidateFSError, "file changed during inventory"):
                        source.inventory()

            self.assertNotEqual(file_path.read_bytes()[0:1], b"a")
            self.assertEqual(file_path.stat().st_ino, original_identity)

    def test_inventory_rejects_directory_replacement_on_ascent(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root_path = Path(temporary) / "root"
            child_path = root_path / "child"
            root_path.mkdir()
            child_path.mkdir()
            (child_path / "payload").write_bytes(b"pinned contents")
            replacement = root_path / "replacement"
            replacement.mkdir()
            (replacement / "outside").write_bytes(b"replacement contents")

            with PinnedDir.open_root(root_path) as root:
                original_listdir = os.listdir
                swapped = False

                def replace_after_listing(path):
                    nonlocal swapped
                    names = original_listdir(path)
                    if isinstance(path, int) and not swapped:
                        info = os.fstat(path)
                        child_info = child_path.stat()
                        if (info.st_dev, info.st_ino) == (child_info.st_dev, child_info.st_ino):
                            child_path.rename(root_path / "child-saved")
                            replacement.rename(child_path)
                            swapped = True
                    return names

                with patch("p3_candidate_fs.os.listdir", side_effect=replace_after_listing):
                    with self.assertRaisesRegex(CandidateFSError, "directory changed during inventory"):
                        root.inventory()
                self.assertTrue(swapped)

    def test_inventory_rejects_directory_replacement_between_stat_and_open(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root_path = Path(temporary) / "root"
            child_path = root_path / "child"
            replacement = Path(temporary) / "replacement"
            root_path.mkdir()
            child_path.mkdir()
            (child_path / "payload").write_bytes(b"pinned contents")
            replacement.mkdir()
            (replacement / "outside").write_bytes(b"replacement contents")

            with PinnedDir.open_root(root_path) as root:
                original_open = os.open
                root_identity = root.identity()
                swapped = False

                def replace_before_open(path, flags, *args, **kwargs):
                    nonlocal swapped
                    dir_fd = kwargs.get("dir_fd")
                    parent_identity = (None if dir_fd is None else
                                       (os.fstat(dir_fd).st_dev, os.fstat(dir_fd).st_ino))
                    if (path == "child" and not swapped and parent_identity == root_identity):
                        child_path.rename(root_path / "child-saved")
                        replacement.rename(child_path)
                        swapped = True
                    return original_open(path, flags, *args, **kwargs)

                with patch("p3_candidate_fs.os.open", side_effect=replace_before_open):
                    with self.assertRaisesRegex(
                        CandidateFSError, "directory changed during inventory: child"
                    ):
                        root.inventory()
                self.assertTrue(swapped)

    def test_copy_rejects_overlapping_roots(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            source_path = base / "source"
            destination_path = source_path / "nested-destination"
            source_path.mkdir()
            destination_path.mkdir()
            with PinnedDir.open_root(source_path) as source, \
                    PinnedDir.open_root(destination_path) as destination:
                with self.assertRaisesRegex(CandidateFSError, "roots overlap"):
                    destination.copy_tree_from(source)

            destination_path = base / "destination"
            source_path = destination_path / "nested-source"
            destination_path.mkdir()
            source_path.mkdir()
            with PinnedDir.open_root(source_path) as source, \
                    PinnedDir.open_root(destination_path) as destination:
                with self.assertRaisesRegex(CandidateFSError, "roots overlap"):
                    destination.copy_tree_from(source)

    def test_copy_closes_source_duplicate_when_destination_dup_fails(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            source_path, destination_path = base / "source", base / "destination"
            source_path.mkdir()
            destination_path.mkdir()
            with PinnedDir.open_root(source_path) as source, \
                    PinnedDir.open_root(destination_path) as destination:
                duplicate = []
                original_dup = os.dup

                def fail_destination_dup(fd: int) -> int:
                    if fd == source.fd:
                        duplicate.append(original_dup(fd))
                        return duplicate[-1]
                    if fd == destination.fd:
                        raise OSError("simulated descriptor exhaustion")
                    return original_dup(fd)

                with patch("p3_candidate_fs.os.dup", side_effect=fail_destination_dup):
                    with self.assertRaises(OSError):
                        destination.copy_tree_from(source)
                with self.assertRaises(OSError):
                    os.fstat(duplicate[0])

    def test_copy_rejects_destination_directory_replacement(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            source_path, destination_path = base / "source", base / "destination"
            source_path.mkdir()
            destination_path.mkdir()
            (source_path / "nested").mkdir()
            (source_path / "nested" / "payload").write_bytes(b"source")

            with PinnedDir.open_root(source_path) as source, \
                    PinnedDir.open_root(destination_path) as destination:
                destination_info = os.fstat(destination.fd)
                destination_id = (destination_info.st_dev, destination_info.st_ino)
                original_open = os.open
                replaced = False

                def replace_before_destination_open(path, flags, *args, **kwargs):
                    nonlocal replaced
                    dir_fd = kwargs.get("dir_fd")
                    if path == "nested" and dir_fd is not None and not replaced:
                        parent_info = os.fstat(dir_fd)
                        if (parent_info.st_dev, parent_info.st_ino) == destination_id:
                            os.rename("nested", "created", src_dir_fd=dir_fd,
                                      dst_dir_fd=dir_fd)
                            os.mkdir("nested", dir_fd=dir_fd)
                            replaced = True
                    return original_open(path, flags, *args, **kwargs)

                with patch("p3_candidate_fs.os.open", side_effect=replace_before_destination_open):
                    with self.assertRaisesRegex(CandidateFSError, "destination directory changed"):
                        destination.copy_tree_from(source)
                self.assertTrue(replaced)
                self.assertFalse((destination_path / "nested" / "payload").exists())

    def test_copy_sanitizes_source_directory_modes(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            source_path, destination_path = base / "source", base / "destination"
            source_path.mkdir()
            destination_path.mkdir()
            nested = source_path / "nested"
            nested.mkdir()
            nested.chmod(0o777)
            (nested / "payload").write_bytes(b"source")

            with PinnedDir.open_root(source_path) as source, \
                    PinnedDir.open_root(destination_path) as destination:
                destination.copy_tree_from(source)

            self.assertEqual(stat.S_IMODE((destination_path / "nested").stat().st_mode), 0o755)

    def test_copy_rejects_hard_link_replacement_between_stat_and_open(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            source_path = base / "source"
            destination_path = base / "destination"
            source_path.mkdir()
            destination_path.mkdir()
            (source_path / "payload").write_bytes(b"original source bytes")
            outside_file = base / "outside"
            outside_file.write_bytes(b"outside secret bytes")

            with PinnedDir.open_root(source_path) as source, \
                    PinnedDir.open_root(destination_path) as destination:
                original_open = os.open
                source_identity = source.identity()
                swapped = False

                def is_source_descriptor(descriptor):
                    info = os.fstat(descriptor)
                    return (info.st_dev, info.st_ino) == source_identity

                def replace_before_open(path, flags, *args, **kwargs):
                    nonlocal swapped
                    dir_fd = kwargs.get("dir_fd")
                    if (path == "payload" and dir_fd is not None and not swapped
                            and is_source_descriptor(dir_fd)):
                        (source_path / "payload").unlink()
                        os.link(outside_file, source_path / "payload")
                        swapped = True
                    return original_open(path, flags, *args, **kwargs)

                with patch.object(os, "open", side_effect=replace_before_open):
                    with self.assertRaisesRegex(CandidateFSError, "hard-linked source file rejected"):
                        destination.copy_tree_from(source)

                self.assertTrue(swapped)
                self.assertEqual(outside_file.read_bytes(), b"outside secret bytes")
                self.assertFalse((destination_path / "payload").exists())
                self.assertEqual(destination.inventory(), {})

    def test_inventory_rejects_hard_link_replacement_between_stat_and_open(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            source_path = base / "source"
            source_path.mkdir()
            (source_path / "payload").write_bytes(b"original source bytes")
            outside_file = base / "outside"
            outside_file.write_bytes(b"outside secret bytes")

            with PinnedDir.open_root(source_path) as source:
                original_open = os.open
                swapped = False

                def replace_before_open(path, flags, *args, **kwargs):
                    nonlocal swapped
                    if path == "payload" and not swapped:
                        (source_path / "payload").unlink()
                        os.link(outside_file, source_path / "payload")
                        swapped = True
                    return original_open(path, flags, *args, **kwargs)

                with patch.object(os, "open", side_effect=replace_before_open):
                    with self.assertRaisesRegex(CandidateFSError, "hard-linked file rejected"):
                        source.inventory()

                self.assertTrue(swapped)
                self.assertEqual(outside_file.read_bytes(), b"outside secret bytes")

    def test_copy_rejects_directory_replacement_between_stat_and_open(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            source_path = base / "source"
            destination_path = base / "destination"
            outside_path = base / "outside"
            (source_path / "child").mkdir(parents=True)
            destination_path.mkdir()
            (outside_path / "child").mkdir(parents=True)
            (source_path / "child" / "original.txt").write_bytes(b"original source")
            (outside_path / "child" / "secret.txt").write_bytes(b"outside secret")

            with PinnedDir.open_root(source_path) as source, \
                    PinnedDir.open_root(destination_path) as destination:
                original_open = os.open
                source_identity = source.identity()
                swapped = False

                def is_source_descriptor(descriptor):
                    info = os.fstat(descriptor)
                    return (info.st_dev, info.st_ino) == source_identity

                def replace_before_open(path, flags, *args, **kwargs):
                    nonlocal swapped
                    dir_fd = kwargs.get("dir_fd")
                    if (path == "child" and dir_fd is not None and not swapped
                            and is_source_descriptor(dir_fd)):
                        (source_path / "child").rename(source_path / "child-saved")
                        (outside_path / "child").rename(source_path / "child")
                        swapped = True
                    return original_open(path, flags, *args, **kwargs)

                with patch.object(os, "open", side_effect=replace_before_open):
                    with self.assertRaisesRegex(CandidateFSError, "source directory changed"):
                        destination.copy_tree_from(source)

                self.assertTrue(swapped)
                self.assertFalse((destination_path / "child" / "secret.txt").exists())
                self.assertEqual(destination.inventory(), {})

    def test_copy_bounds_source_growth_after_file_stat_and_removes_partial_file(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            source_path, destination_path = base / "source", base / "destination"
            source_path.mkdir()
            destination_path.mkdir()
            source_file = source_path / "payload"
            source_file.write_bytes(b"123")

            with PinnedDir.open_root(source_path) as source, \
                    PinnedDir.open_root(destination_path) as destination:
                original_open = os.open
                grew = False

                def grow_before_destination_open(path, flags, *args, **kwargs):
                    nonlocal grew
                    if path == "payload" and flags & os.O_CREAT and not grew:
                        source_file.write_bytes(b"12345")
                        grew = True
                    return original_open(path, flags, *args, **kwargs)

                with patch.object(os, "open", side_effect=grow_before_destination_open):
                    with self.assertRaisesRegex(CandidateFSError, "exceeds 4 copied bytes"):
                        destination.copy_tree_from(source, max_entries=10, max_bytes=4)

                self.assertTrue(grew)
                self.assertEqual(destination.inventory(), {})

    def test_remove_tree_rejects_directory_replacement_before_recursive_open(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root_path = Path(temporary)
            victim = root_path / "victim"
            nested = victim / "nested"
            nested.mkdir(parents=True)
            (nested / "original.txt").write_text("original contents")

            with PinnedDir.open_root(root_path) as root:
                original_open = os.open
                swapped = False

                def replace_before_open(path, flags, *args, **kwargs):
                    nonlocal swapped
                    if path == "nested" and kwargs.get("dir_fd") != root.fd and not swapped:
                        nested.rename(victim / "detached")
                        nested.mkdir()
                        (nested / "replacement.txt").write_text("replacement contents")
                        swapped = True
                    return original_open(path, flags, *args, **kwargs)

                with patch.object(os, "open", side_effect=replace_before_open):
                    with self.assertRaisesRegex(CandidateFSError, "directory changed during removal"):
                        root.remove_tree("victim")

                self.assertTrue(swapped)
                self.assertEqual((victim / "detached/original.txt").read_text(),
                                 "original contents")
                self.assertEqual((victim / "nested/replacement.txt").read_text(),
                                 "replacement contents")

    def test_remove_tree_handles_deep_directory_chain_with_bounded_descriptors(self) -> None:
        depth = 1100
        directory_flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
        with tempfile.TemporaryDirectory() as temporary:
            root_path = Path(temporary) / "root"
            root_path.mkdir()
            active_fd = os.open(root_path, directory_flags)
            try:
                os.mkdir("victim", dir_fd=active_fd)
                child_fd = os.open("victim", directory_flags, dir_fd=active_fd)
                os.close(active_fd)
                active_fd = child_fd
                for _ in range(depth):
                    os.mkdir("d", dir_fd=active_fd)
                    child_fd = os.open("d", directory_flags, dir_fd=active_fd)
                    os.close(active_fd)
                    active_fd = child_fd
                file_fd = os.open("leaf", os.O_WRONLY | os.O_CREAT | os.O_EXCL,
                                  0o600, dir_fd=active_fd)
                os.close(file_fd)
            finally:
                os.close(active_fd)

            original_soft, hard_limit = resource.getrlimit(resource.RLIMIT_NOFILE)
            limit_changed = False
            try:
                with PinnedDir.open_root(root_path) as root:
                    descriptor_count = len(os.listdir("/dev/fd"))
                    low_limit = max(32, descriptor_count + 8)
                    if low_limit >= depth:
                        self.skipTest("cannot set a descriptor limit far below the tree depth")
                    if hard_limit != resource.RLIM_INFINITY and hard_limit < low_limit:
                        self.skipTest("hard file descriptor limit is too low for the regression")
                    resource.setrlimit(resource.RLIMIT_NOFILE, (low_limit, hard_limit))
                    limit_changed = True
                    root.remove_tree("victim")
                    self.assertFalse((root_path / "victim").exists())
            finally:
                if limit_changed:
                    resource.setrlimit(resource.RLIMIT_NOFILE, (original_soft, hard_limit))
                if (root_path / "victim").exists():
                    root_fd = os.open(root_path, directory_flags)
                    current_fd = os.open("victim", directory_flags, dir_fd=root_fd)
                    try:
                        for _ in range(depth):
                            child_fd = os.open("d", directory_flags, dir_fd=current_fd)
                            os.close(current_fd)
                            current_fd = child_fd
                        try:
                            os.unlink("leaf", dir_fd=current_fd)
                        except FileNotFoundError:
                            pass
                        for _ in range(depth):
                            parent_fd = os.open("..", directory_flags, dir_fd=current_fd)
                            os.rmdir("d", dir_fd=parent_fd)
                            os.close(current_fd)
                            current_fd = parent_fd
                        os.close(current_fd)
                        current_fd = -1
                        os.rmdir("victim", dir_fd=root_fd)
                    finally:
                        if current_fd >= 0:
                            os.close(current_fd)
                        os.close(root_fd)

    def test_descriptor_ownership_and_close_behavior(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            original = PinnedDir.open_root(temporary)
            duplicate = PinnedDir.from_fd(original.fd)
            duplicate_fd = duplicate.fd
            duplicate.close()
            duplicate.close()
            with self.assertRaises(OSError):
                os.fstat(duplicate_fd)
            self.assertEqual(os.fstat(original.fd).st_mode & 0o170000, 0o040000)
            original.close()
            with self.assertRaises(OSError):
                os.fstat(original.fd)

            inherited_fd = os.open(temporary, os.O_RDONLY | os.O_DIRECTORY)
            wrapper = PinnedDir.from_fd(inherited_fd, duplicate=False)
            wrapper.close()
            with self.assertRaises(OSError):
                os.fstat(inherited_fd)

            with PinnedDir.open_root(temporary) as root:
                self.assertEqual(root.identity(), (os.fstat(root.fd).st_dev, os.fstat(root.fd).st_ino))
                created = PinnedDir.create_child(root, "made/nested")
                try:
                    self.assertEqual(created.identity(), root.child_identity("made/nested"))
                    created.mkdir("subdir")
                    created.atomic_write("subdir/item", b"move me")
                    created.rename("subdir/item", created, "subdir/moved")
                    self.assertEqual(created.read_bytes("subdir/moved"), b"move me")
                    created.unlink("subdir/moved")
                    with self.assertRaises(CandidateFSError):
                        created.read_bytes("subdir/moved")
                finally:
                    created.close()
                child_fd = root.open_file("owned", os.O_WRONLY | os.O_CREAT, 0o600)
                os.write(child_fd, b"data")
                os.close(child_fd)
                self.assertEqual(root.read_bytes("owned"), b"data")


if __name__ == "__main__":
    unittest.main()
