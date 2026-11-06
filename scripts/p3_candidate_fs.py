"""Descriptor-relative filesystem helpers for isolated P3 candidates.

All paths accepted by this module are slash-separated paths relative to an
already pinned directory descriptor. Descendants are opened without following
symlinks, so replacing a named parent after it has been opened cannot redirect
subsequent operations to a different tree.
"""

from __future__ import annotations

import os
import secrets
import stat
import hashlib
from pathlib import Path, PurePosixPath, PureWindowsPath
from typing import BinaryIO


class CandidateFSError(ValueError):
    """A descriptor-relative candidate filesystem operation was rejected."""


_HAS_DIR_FD_OPEN = os.open in os.supports_dir_fd


def relative_parts(value: str | os.PathLike[str]) -> tuple[str, ...]:
    raw = os.fspath(value)
    if not isinstance(raw, str) or not raw or "\x00" in raw or "\\" in raw:
        raise CandidateFSError("path must be a nonempty slash-separated relative path")
    posix, windows = PurePosixPath(raw), PureWindowsPath(raw)
    parts = tuple(raw.split("/"))
    if (posix.is_absolute() or windows.is_absolute() or windows.drive
            or any(part in ("", ".", "..") for part in parts)):
        raise CandidateFSError("path must not be absolute or contain dot components")
    return parts


def _directory_flags() -> int:
    if not hasattr(os, "O_NOFOLLOW") or not _HAS_DIR_FD_OPEN:
        raise CandidateFSError("platform lacks descriptor-relative no-follow directory operations")
    return os.O_RDONLY | getattr(os, "O_CLOEXEC", 0) | getattr(os, "O_DIRECTORY", 0) | os.O_NOFOLLOW


class PinnedDir:
    """An owned directory descriptor used as the anchor for relative paths."""

    def __init__(self, fd: int, *, owned: bool = True):
        self.fd = fd
        self._owned = owned
        mode = os.fstat(fd).st_mode
        if not stat.S_ISDIR(mode):
            if owned:
                os.close(fd)
            raise CandidateFSError("pinned descriptor is not a directory")

    @classmethod
    def open_root(cls, path: str | os.PathLike[str]) -> "PinnedDir":
        """Open the user-selected repository root as the trust anchor."""
        try:
            fd = os.open(os.fspath(path), _directory_flags())
        except OSError as error:
            raise CandidateFSError(f"cannot open repository root: {path}") from error
        return cls(fd)

    @classmethod
    def from_fd(cls, fd: int, *, duplicate: bool = True) -> "PinnedDir":
        """Wrap an inherited directory FD, duplicating it by default."""
        try:
            owned_fd = os.dup(fd) if duplicate else fd
            return cls(owned_fd)
        except OSError as error:
            raise CandidateFSError("cannot duplicate inherited directory descriptor") from error

    @classmethod
    def open_child(cls, parent: "PinnedDir", relative: str | os.PathLike[str]) -> "PinnedDir":
        current = os.dup(parent.fd)
        try:
            for part in relative_parts(relative):
                child = os.open(part, _directory_flags(), dir_fd=current)
                os.close(current)
                current = child
            result = current
            current = -1
            return cls(result)
        except OSError as error:
            raise CandidateFSError(f"cannot open pinned child directory: {relative}") from error
        finally:
            if current >= 0:
                os.close(current)

    @classmethod
    def create_child(cls, parent: "PinnedDir", relative: str | os.PathLike[str],
                     *, mode: int = 0o700) -> "PinnedDir":
        """Create missing path components beneath parent and return the pinned leaf."""
        current = os.dup(parent.fd)
        try:
            for part in relative_parts(relative):
                try:
                    child = os.open(part, _directory_flags(), dir_fd=current)
                except FileNotFoundError:
                    try:
                        os.mkdir(part, mode, dir_fd=current)
                    except FileExistsError:
                        pass
                    child = os.open(part, _directory_flags(), dir_fd=current)
                os.close(current)
                current = child
            result = current
            current = -1
            return cls(result)
        except OSError as error:
            raise CandidateFSError(f"cannot create pinned child directory: {relative}") from error
        finally:
            if current >= 0:
                os.close(current)

    def _parent_and_name(self, relative: str | os.PathLike[str]) -> tuple[int, str]:
        parts = relative_parts(relative)
        current = os.dup(self.fd)
        try:
            for part in parts[:-1]:
                child = os.open(part, _directory_flags(), dir_fd=current)
                os.close(current)
                current = child
            result = current
            current = -1
            return result, parts[-1]
        except OSError as error:
            raise CandidateFSError(f"cannot open parent of relative path: {relative}") from error
        finally:
            if current >= 0:
                os.close(current)

    def open_file(self, relative: str | os.PathLike[str], flags: int,
                  mode: int = 0o600) -> int:
        parent_fd, name = self._parent_and_name(relative)
        try:
            descriptor = os.open(name, flags | getattr(os, "O_NOFOLLOW", 0)
                                 | getattr(os, "O_CLOEXEC", 0), mode, dir_fd=parent_fd)
            if not stat.S_ISREG(os.fstat(descriptor).st_mode):
                os.close(descriptor)
                raise CandidateFSError(f"relative path is not a regular file: {relative}")
            return descriptor
        except OSError as error:
            raise CandidateFSError(f"cannot open relative file safely: {relative}") from error
        finally:
            os.close(parent_fd)

    def read_bytes(self, relative: str | os.PathLike[str], *, limit: int | None = None) -> bytes:
        fd = self.open_file(relative, os.O_RDONLY | getattr(os, "O_NONBLOCK", 0))
        with os.fdopen(fd, "rb") as stream:
            contents = stream.read(None if limit is None else limit + 1)
        if limit is not None and len(contents) > limit:
            raise CandidateFSError(f"relative file exceeds size limit: {relative}")
        return contents

    def inventory(self, relative: str | os.PathLike[str] = ".", *,
                  max_entries: int | None = None,
                  max_bytes: int | None = None,
                  reject_hardlinks: bool = True) -> dict[str, str]:
        """Hash a regular-file tree through pinned, no-follow descriptors."""
        root = self if os.fspath(relative) == "." else self.open_child(self, relative)
        current_fd = -1
        try:
            result: dict[str, str] = {}
            budget = {"entries": 0, "bytes": 0}

            def identity(info: os.stat_result) -> tuple[int, int]:
                return info.st_dev, info.st_ino

            def directory_frame(fd: int, name: str | None,
                                parent_identity: tuple[int, int] | None,
                                expected: os.stat_result | None = None) -> dict:
                before = os.fstat(fd)
                if not stat.S_ISDIR(before.st_mode):
                    raise CandidateFSError("non-directory entry during inventory")
                if expected is not None and identity(before) != identity(expected):
                    raise CandidateFSError(f"directory changed during inventory: {name}")
                names = os.listdir(fd)
                for child_name in names:
                    if (child_name in (".", "..") or "/" in child_name
                            or "\x00" in child_name):
                        raise CandidateFSError("directory returned an invalid entry name")
                budget["entries"] += len(names)
                if max_entries is not None and budget["entries"] > max_entries:
                    raise CandidateFSError(f"source tree exceeds {max_entries} entries")
                return {
                    "name": name,
                    "identity": identity(before),
                    "before": (before.st_mtime_ns, before.st_ctime_ns),
                    "parent_identity": parent_identity,
                    "names": names,
                    "index": 0,
                }

            current_fd = os.dup(root.fd)
            stack = [directory_frame(current_fd, None, None)]
            directory_flags = _directory_flags()
            while stack:
                frame = stack[-1]
                if frame["index"] == len(frame["names"]):
                    finished = os.fstat(current_fd)
                    if (identity(finished) != frame["identity"]
                            or (finished.st_mtime_ns, finished.st_ctime_ns) != frame["before"]):
                        raise CandidateFSError(
                            f"directory changed during inventory: {frame['name'] or '.'}"
                        )
                    if len(stack) == 1:
                        break

                    child_name = frame["name"]
                    child_identity = frame["identity"]
                    parent_fd = os.open("..", directory_flags, dir_fd=current_fd)
                    try:
                        parent_info = os.fstat(parent_fd)
                        if identity(parent_info) != frame["parent_identity"]:
                            raise CandidateFSError("directory parent changed during inventory")
                        named_child = os.stat(child_name, dir_fd=parent_fd,
                                              follow_symlinks=False)
                        if (not stat.S_ISDIR(named_child.st_mode)
                                or identity(named_child) != child_identity):
                            raise CandidateFSError(
                                f"directory changed during inventory: {child_name}"
                            )
                    except BaseException:
                        os.close(parent_fd)
                        raise
                    os.close(current_fd)
                    current_fd = parent_fd
                    stack.pop()
                    continue

                name = frame["names"][frame["index"]]
                frame["index"] += 1
                entry = os.stat(name, dir_fd=current_fd, follow_symlinks=False)
                if stat.S_ISDIR(entry.st_mode):
                    try:
                        child_fd = os.open(name, directory_flags, dir_fd=current_fd)
                    except OSError as error:
                        raise CandidateFSError(
                            f"cannot open directory safely during inventory: {name}"
                        ) from error
                    try:
                        child_frame = directory_frame(
                            child_fd, name, frame["identity"], expected=entry
                        )
                    except BaseException:
                        os.close(child_fd)
                        raise
                    os.close(current_fd)
                    current_fd = child_fd
                    stack.append(child_frame)
                elif stat.S_ISREG(entry.st_mode):
                    path = "/".join((*[item["name"] for item in stack[1:]], name))
                    if reject_hardlinks and entry.st_nlink != 1:
                        raise CandidateFSError(f"hard-linked file rejected: {path}")
                    try:
                        fd = os.open(name, os.O_RDONLY | getattr(os, "O_NONBLOCK", 0)
                                     | getattr(os, "O_NOFOLLOW", 0)
                                     | getattr(os, "O_CLOEXEC", 0), dir_fd=current_fd)
                    except OSError as error:
                        raise CandidateFSError(
                            f"cannot open relative file safely: {path}"
                        ) from error
                    digest = hashlib.sha256()
                    with os.fdopen(fd, "rb") as stream:
                        opened = os.fstat(stream.fileno())
                        def file_version(info: os.stat_result) -> tuple[int, int, int, int, int]:
                            return (info.st_dev, info.st_ino, info.st_size,
                                    info.st_mtime_ns, info.st_ctime_ns)

                        if not stat.S_ISREG(opened.st_mode):
                            raise CandidateFSError(f"non-regular file rejected: {path}")
                        if reject_hardlinks and opened.st_nlink != 1:
                            raise CandidateFSError(f"hard-linked file rejected: {path}")
                        opened_version = file_version(opened)
                        if opened_version != file_version(entry):
                            raise CandidateFSError(f"file changed during inventory: {path}")
                        if max_bytes is not None and budget["bytes"] + opened.st_size > max_bytes:
                            raise CandidateFSError(
                                f"source tree exceeds {max_bytes} regular-file bytes"
                            )
                        for block in iter(lambda: stream.read(1024 * 1024), b""):
                            if max_bytes is not None and budget["bytes"] + len(block) > max_bytes:
                                raise CandidateFSError(
                                    f"source tree exceeds {max_bytes} regular-file bytes"
                                )
                            budget["bytes"] += len(block)
                            digest.update(block)
                        final = os.fstat(stream.fileno())
                        current = os.stat(name, dir_fd=current_fd, follow_symlinks=False)
                        if (not stat.S_ISREG(final.st_mode)
                                or (reject_hardlinks and final.st_nlink != 1)
                                or file_version(final) != opened_version):
                            raise CandidateFSError(f"file changed during inventory: {path}")
                        if (not stat.S_ISREG(current.st_mode)
                                or (reject_hardlinks and current.st_nlink != 1)
                                or file_version(current) != opened_version):
                            if (reject_hardlinks and stat.S_ISREG(current.st_mode)
                                    and current.st_nlink != 1):
                                raise CandidateFSError(f"hard-linked file rejected: {path}")
                            raise CandidateFSError(f"file changed during inventory: {path}")
                    result[path] = digest.hexdigest()
                elif stat.S_ISLNK(entry.st_mode):
                    path = "/".join((*[item["name"] for item in stack[1:]], name))
                    raise CandidateFSError(f"symlink rejected: {path}")
                else:
                    path = "/".join((*[item["name"] for item in stack[1:]], name))
                    raise CandidateFSError(f"non-regular file rejected: {path}")

            return dict(sorted(result.items()))
        finally:
            if current_fd >= 0:
                os.close(current_fd)
            if root is not self:
                root.close()

    def identity(self) -> tuple[int, int]:
        info = os.fstat(self.fd)
        return info.st_dev, info.st_ino

    def child_identity(self, relative: str | os.PathLike[str]) -> tuple[int, int]:
        child = self.open_child(self, relative)
        try:
            return child.identity()
        finally:
            child.close()

    def mkdir(self, relative: str | os.PathLike[str], *, mode: int = 0o700) -> None:
        current = os.dup(self.fd)
        try:
            for index, part in enumerate(relative_parts(relative)):
                if index == len(relative_parts(relative)) - 1:
                    try:
                        os.mkdir(part, mode, dir_fd=current)
                    except FileExistsError:
                        raise CandidateFSError(f"relative directory already exists: {relative}")
                    return
                child = os.open(part, _directory_flags(), dir_fd=current)
                os.close(current)
                current = child
        except OSError as error:
            raise CandidateFSError(f"cannot create relative directory: {relative}") from error
        finally:
            os.close(current)

    def unlink(self, relative: str | os.PathLike[str], *, missing_ok: bool = False) -> None:
        parent_fd, name = self._parent_and_name(relative)
        try:
            os.unlink(name, dir_fd=parent_fd)
        except FileNotFoundError:
            if not missing_ok:
                raise
        finally:
            os.close(parent_fd)

    def rename(self, source: str | os.PathLike[str], destination_dir: "PinnedDir",
               destination: str | os.PathLike[str], *, replace: bool = True) -> None:
        source_fd, source_name = self._parent_and_name(source)
        destination_fd = None
        try:
            destination_fd, destination_name = destination_dir._parent_and_name(destination)
            if replace:
                os.replace(source_name, destination_name, src_dir_fd=source_fd,
                           dst_dir_fd=destination_fd)
            else:
                os.rename(source_name, destination_name, src_dir_fd=source_fd,
                          dst_dir_fd=destination_fd)
        except OSError as error:
            raise CandidateFSError(f"cannot rename relative path: {source}") from error
        finally:
            os.close(source_fd)
            if destination_fd is not None:
                os.close(destination_fd)

    def atomic_write(self, relative: str | os.PathLike[str], contents: bytes,
                     *, mode: int = 0o600, replace: bool = True) -> None:
        parent_fd, name = self._parent_and_name(relative)
        temporary = f".{name}.{secrets.token_hex(12)}.tmp"
        fd: int | None = None
        try:
            fd = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL
                         | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_CLOEXEC", 0),
                         0o600, dir_fd=parent_fd)
            with os.fdopen(fd, "wb") as stream:
                fd = None
                stream.write(contents)
                stream.flush()
                os.fchmod(stream.fileno(), mode)
                os.fsync(stream.fileno())
            if replace:
                os.replace(temporary, name, src_dir_fd=parent_fd, dst_dir_fd=parent_fd)
            else:
                os.link(temporary, name, src_dir_fd=parent_fd, dst_dir_fd=parent_fd,
                        follow_symlinks=False)
                os.unlink(temporary, dir_fd=parent_fd)
            os.fsync(parent_fd)
        except OSError as error:
            raise CandidateFSError(f"cannot atomically write relative file: {relative}") from error
        finally:
            if fd is not None:
                os.close(fd)
            try:
                os.unlink(temporary, dir_fd=parent_fd)
            except FileNotFoundError:
                pass
            os.close(parent_fd)

    def copy_tree_from(self, source: "PinnedDir", *, max_entries: int | None = None,
                       max_bytes: int | None = None) -> None:
        """Copy a pinned source tree, optionally bounding entries and copied bytes."""
        directory_flags = _directory_flags()
        budget = {"entries": 0, "bytes": 0}

        def begin_copy(source_fd: int, destination_fd: int, name: str | None = None,
                       expected_source: os.stat_result | None = None,
                       expected_destination: os.stat_result | None = None) -> dict:
            source_info = os.fstat(source_fd)
            destination_info = os.fstat(destination_fd)
            if (not stat.S_ISDIR(source_info.st_mode)
                    or not stat.S_ISDIR(destination_info.st_mode)):
                raise CandidateFSError("source and destination copy entries must be directories")
            if (expected_source is not None
                    and (source_info.st_dev, source_info.st_ino)
                    != (expected_source.st_dev, expected_source.st_ino)):
                raise CandidateFSError(f"source directory changed during copy: {name}")
            if (expected_destination is not None
                    and (destination_info.st_dev, destination_info.st_ino)
                    != (expected_destination.st_dev, expected_destination.st_ino)):
                raise CandidateFSError(f"destination directory changed during copy: {name}")
            if (source_info.st_dev, source_info.st_ino) == destination_root_identity:
                raise CandidateFSError("source and destination copy roots overlap")
            frame = {
                "name": name,
                "source_identity": (source_info.st_dev, source_info.st_ino),
                "destination_identity": (destination_info.st_dev, destination_info.st_ino),
                "source_before": (source_info.st_dev, source_info.st_ino,
                                  source_info.st_mtime_ns, source_info.st_ctime_ns),
                "names": [],
                "index": 0,
            }
            with os.scandir(source_fd) as entries:
                for entry in entries:
                    name = entry.name
                    if name in (".", "..") or "/" in name or "\x00" in name:
                        raise CandidateFSError("source directory returned an invalid entry name")
                    budget["entries"] += 1
                    if max_entries is not None and budget["entries"] > max_entries:
                        raise CandidateFSError(f"source tree exceeds {max_entries} entries")
                    frame["names"].append(name)
            return frame

        source_fd = destination_fd = None
        stack = []
        try:
            source_fd = os.dup(source.fd)
            destination_fd = os.dup(self.fd)
            source_info = os.fstat(source_fd)
            destination_info = os.fstat(destination_fd)
            source_identity = (source_info.st_dev, source_info.st_ino)
            destination_root_identity = (destination_info.st_dev, destination_info.st_ino)

            def contains(ancestor_identity: tuple[int, int], descendant_fd: int) -> bool:
                current_fd = os.dup(descendant_fd)
                try:
                    while True:
                        current_info = os.fstat(current_fd)
                        current_identity = (current_info.st_dev, current_info.st_ino)
                        if current_identity == ancestor_identity:
                            return True
                        parent_fd = os.open("..", directory_flags, dir_fd=current_fd)
                        try:
                            parent_info = os.fstat(parent_fd)
                            parent_identity = (parent_info.st_dev, parent_info.st_ino)
                        except BaseException:
                            os.close(parent_fd)
                            raise
                        if parent_identity == current_identity:
                            os.close(parent_fd)
                            return False
                        os.close(current_fd)
                        current_fd = parent_fd
                except OSError as error:
                    raise CandidateFSError(
                        "cannot safely determine whether copy roots overlap"
                    ) from error
                finally:
                    os.close(current_fd)

            if (contains(source_identity, destination_fd)
                    or contains(destination_root_identity, source_fd)):
                raise CandidateFSError("source and destination copy roots overlap")

            stack.append(begin_copy(source_fd, destination_fd))
            while stack:
                frame = stack[-1]
                if frame["index"] == len(frame["names"]):
                    source_info = os.fstat(source_fd)
                    source_after = (source_info.st_dev, source_info.st_ino,
                                    source_info.st_mtime_ns, source_info.st_ctime_ns)
                    if source_after != frame["source_before"]:
                        raise CandidateFSError(
                            f"source directory changed during copy: {frame['name'] or '.'}"
                        )
                    if len(stack) == 1:
                        break

                    child_frame = frame
                    parent_frame = stack[-2]
                    source_parent = os.open("..", directory_flags, dir_fd=source_fd)
                    try:
                        destination_parent = os.open("..", directory_flags, dir_fd=destination_fd)
                    except BaseException:
                        os.close(source_parent)
                        raise
                    try:
                        source_parent_info = os.fstat(source_parent)
                        destination_parent_info = os.fstat(destination_parent)
                        if ((source_parent_info.st_dev, source_parent_info.st_ino)
                                != parent_frame["source_identity"]):
                            raise CandidateFSError("source directory parent changed during copy")
                        if ((destination_parent_info.st_dev, destination_parent_info.st_ino)
                                != parent_frame["destination_identity"]):
                            raise CandidateFSError("destination directory parent changed during copy")

                        named_source_info = os.stat(
                            child_frame["name"], dir_fd=source_parent, follow_symlinks=False
                        )
                        if stat.S_ISLNK(named_source_info.st_mode):
                            raise CandidateFSError(
                                f"symlink rejected during source copy: {child_frame['name']}"
                            )
                        named_source = os.open(
                            child_frame["name"], directory_flags, dir_fd=source_parent
                        )
                        try:
                            named_destination = os.open(
                                child_frame["name"], directory_flags, dir_fd=destination_parent
                            )
                            try:
                                source_child_info = os.fstat(named_source)
                                destination_child_info = os.fstat(named_destination)
                                if ((source_child_info.st_dev, source_child_info.st_ino)
                                        != child_frame["source_identity"]):
                                    raise CandidateFSError("source directory changed during copy")
                                if ((destination_child_info.st_dev, destination_child_info.st_ino)
                                        != child_frame["destination_identity"]):
                                    raise CandidateFSError("destination directory changed during copy")
                            finally:
                                os.close(named_destination)
                        finally:
                            os.close(named_source)
                        os.fchmod(destination_fd, child_frame["source_mode"])
                    except BaseException as error:
                        os.close(source_parent)
                        os.close(destination_parent)
                        if isinstance(error, OSError):
                            raise CandidateFSError("source or destination directory changed during copy") from error
                        raise

                    os.close(source_fd)
                    os.close(destination_fd)
                    source_fd, destination_fd = source_parent, destination_parent
                    stack.pop()
                    continue

                name = frame["names"][frame["index"]]
                frame["index"] += 1
                if name in (".", "..") or "/" in name or "\x00" in name:
                    raise CandidateFSError("source directory returned an invalid entry name")

                info = os.stat(name, dir_fd=source_fd, follow_symlinks=False)
                if stat.S_ISDIR(info.st_mode):
                    try:
                        os.mkdir(name, stat.S_IMODE(info.st_mode) & 0o755 or 0o700,
                                 dir_fd=destination_fd)
                    except FileExistsError as error:
                        raise CandidateFSError(f"destination already contains: {name}") from error
                    destination_info = os.stat(
                        name, dir_fd=destination_fd, follow_symlinks=False
                    )
                    if not stat.S_ISDIR(destination_info.st_mode):
                        raise CandidateFSError(
                            f"destination directory changed during copy: {name}"
                        )
                    try:
                        source_child = os.open(name, directory_flags, dir_fd=source_fd)
                    except OSError as error:
                        current = os.stat(name, dir_fd=source_fd, follow_symlinks=False)
                        if stat.S_ISLNK(current.st_mode):
                            raise CandidateFSError(f"symlink rejected during source copy: {name}") from error
                        raise CandidateFSError(f"source directory changed during copy: {name}") from error
                    try:
                        destination_child = os.open(name, directory_flags, dir_fd=destination_fd)
                        try:
                            child_frame = begin_copy(
                                source_child, destination_child, name, info, destination_info
                            )
                            child_frame["source_mode"] = stat.S_IMODE(info.st_mode) & 0o755
                        except BaseException:
                            os.close(destination_child)
                            raise
                    except BaseException:
                        os.close(source_child)
                        raise
                    os.close(source_fd)
                    os.close(destination_fd)
                    source_fd, destination_fd = source_child, destination_child
                    stack.append(child_frame)
                elif stat.S_ISREG(info.st_mode):
                    if info.st_nlink != 1:
                        raise CandidateFSError(f"hard-linked source file rejected: {name}")
                    try:
                        source_file = os.open(name, os.O_RDONLY | os.O_NOFOLLOW
                                              | getattr(os, "O_NONBLOCK", 0), dir_fd=source_fd)
                    except OSError as error:
                        current = os.stat(name, dir_fd=source_fd, follow_symlinks=False)
                        if stat.S_ISLNK(current.st_mode):
                            raise CandidateFSError(f"symlink rejected during source copy: {name}") from error
                        if not stat.S_ISREG(current.st_mode):
                            raise CandidateFSError(f"source contains a non-regular file: {name}") from error
                        raise CandidateFSError(f"source file changed during copy: {name}") from error
                    try:
                        opened_info = os.fstat(source_file)
                        if not stat.S_ISREG(opened_info.st_mode):
                            raise CandidateFSError(f"source changed to a non-regular file: {name}")
                        if opened_info.st_nlink != 1:
                            raise CandidateFSError(f"hard-linked source file rejected: {name}")
                        if (opened_info.st_dev, opened_info.st_ino) != (info.st_dev, info.st_ino):
                            raise CandidateFSError(f"source file changed during copy: {name}")
                        if max_bytes is not None and budget["bytes"] + opened_info.st_size > max_bytes:
                            raise CandidateFSError(f"source tree exceeds {max_bytes} copied bytes")
                        destination_file = os.open(
                            name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                            stat.S_IMODE(info.st_mode) & 0o777, dir_fd=destination_fd,
                        )
                        try:
                            with os.fdopen(os.dup(source_file), "rb") as input_stream, \
                                    os.fdopen(destination_file, "wb") as output_stream:
                                copied = 0
                                while True:
                                    remaining = (None if max_bytes is None else
                                                 max_bytes - budget["bytes"] - copied)
                                    block = input_stream.read(1024 * 1024 if remaining is None
                                                              else min(1024 * 1024, remaining + 1))
                                    if not block:
                                        break
                                    if remaining is not None and len(block) > remaining:
                                        raise CandidateFSError(
                                            f"source tree exceeds {max_bytes} copied bytes"
                                        )
                                    output_stream.write(block)
                                    copied += len(block)
                                budget["bytes"] += copied
                        except BaseException:
                            try:
                                os.unlink(name, dir_fd=destination_fd)
                            except OSError:
                                pass
                            raise
                    finally:
                        os.close(source_file)
                else:
                    if stat.S_ISLNK(info.st_mode):
                        raise CandidateFSError(f"symlink rejected during source copy: {name}")
                    raise CandidateFSError(f"source contains a non-regular file: {name}")
        finally:
            if source_fd is not None:
                os.close(source_fd)
            if destination_fd is not None:
                os.close(destination_fd)

    def remove_tree(self, relative: str | os.PathLike[str], *,
                    expected_identity: tuple[int, int] | None = None,
                    max_entries: int | None = None) -> None:
        """Remove a descendant tree without following links or named parents."""
        parent_fd, name = self._parent_and_name(relative)
        try:
            _remove_directory_at(parent_fd, name, expected_identity, max_entries)
        finally:
            os.close(parent_fd)

    def close(self) -> None:
        if self._owned:
            os.close(self.fd)
            self._owned = False

    def __enter__(self) -> "PinnedDir":
        return self

    def __exit__(self, *_: object) -> None:
        self.close()


def _remove_directory_at(parent_fd: int, name: str,
                         expected_identity: tuple[int, int] | None = None,
                         max_entries: int | None = None) -> None:
    if expected_identity is None:
        try:
            observed = os.stat(name, dir_fd=parent_fd, follow_symlinks=False)
        except FileNotFoundError:
            return
        expected_identity = observed.st_dev, observed.st_ino
    directory_flags = _directory_flags()
    try:
        current_fd = os.open(name, directory_flags, dir_fd=parent_fd)
    except FileNotFoundError:
        return

    def identity(info: os.stat_result) -> tuple[int, int]:
        return info.st_dev, info.st_ino

    budget = 0

    def names_for(fd: int) -> list[str]:
        nonlocal budget
        names = []
        with os.scandir(fd) as entries:
            for entry in entries:
                budget += 1
                if max_entries is not None and budget > max_entries:
                    raise CandidateFSError(f"tree exceeds {max_entries} entries during removal")
                names.append(entry.name)
        return names

    stack = []
    try:
        opened = os.fstat(current_fd)
        if not stat.S_ISDIR(opened.st_mode) or identity(opened) != expected_identity:
            raise CandidateFSError(f"directory changed during removal: {name}")
        stack.append({
            "name": name,
            "identity": identity(opened),
            "parent_identity": identity(os.fstat(parent_fd)),
            "names": names_for(current_fd),
            "index": 0,
        })

        # Keep one active directory descriptor regardless of tree depth.
        while stack:
            frame = stack[-1]
            if frame["index"] == len(frame["names"]):
                if len(stack) == 1:
                    named = os.stat(name, dir_fd=parent_fd, follow_symlinks=False)
                    if not stat.S_ISDIR(named.st_mode) or identity(named) != frame["identity"]:
                        raise CandidateFSError(f"directory changed during removal: {name}")
                    os.close(current_fd)
                    current_fd = -1
                    os.rmdir(name, dir_fd=parent_fd)
                    break

                child_frame = frame
                parent_frame = stack[-2]
                ancestor_fd = os.open("..", directory_flags, dir_fd=current_fd)
                try:
                    if identity(os.fstat(ancestor_fd)) != parent_frame["identity"]:
                        raise CandidateFSError("directory parent changed during removal")
                    named_child = os.stat(child_frame["name"], dir_fd=ancestor_fd,
                                          follow_symlinks=False)
                    if (not stat.S_ISDIR(named_child.st_mode)
                            or identity(named_child) != child_frame["identity"]):
                        raise CandidateFSError(
                            f"directory changed during removal: {child_frame['name']}"
                        )
                    os.close(current_fd)
                    current_fd = ancestor_fd
                    ancestor_fd = -1
                    os.rmdir(child_frame["name"], dir_fd=current_fd)
                    stack.pop()
                finally:
                    if ancestor_fd >= 0:
                        os.close(ancestor_fd)
                continue

            entry = frame["names"][frame["index"]]
            frame["index"] += 1
            info = os.stat(entry, dir_fd=current_fd, follow_symlinks=False)
            if not stat.S_ISDIR(info.st_mode):
                named = os.stat(entry, dir_fd=current_fd, follow_symlinks=False)
                if identity(named) != identity(info):
                    raise CandidateFSError(f"entry changed during removal: {entry}")
                os.unlink(entry, dir_fd=current_fd)
                continue

            child_fd = os.open(entry, directory_flags, dir_fd=current_fd)
            try:
                child = os.fstat(child_fd)
                if not stat.S_ISDIR(child.st_mode) or identity(child) != identity(info):
                    raise CandidateFSError(f"directory changed during removal: {entry}")
                stack.append({
                    "name": entry,
                    "identity": identity(child),
                    "parent_identity": frame["identity"],
                    "names": names_for(child_fd),
                    "index": 0,
                })
            except BaseException:
                os.close(child_fd)
                raise
            os.close(current_fd)
            current_fd = child_fd
    finally:
        if current_fd >= 0:
            os.close(current_fd)
