#!/usr/bin/env python3
"""Apply a hash-pinned unified diff series to a disposable candidate source copy."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import stat
import sys
import tempfile
import uuid
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath, PureWindowsPath
from typing import Any

from validate_candidate_snapshot import MAX_SNAPSHOT_BYTES, MAX_SNAPSHOT_ENTRIES
from p3_candidate_fs import CandidateFSError, PinnedDir


HUNK_RE = re.compile(r"^@@ -([0-9]+)(?:,([0-9]+))? \+([0-9]+)(?:,([0-9]+))? @@(?:[^\n]*)\n$")
RUN_ID_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$")
PACKAGE_DIR = "gpui-pre-0.3.9"
SCHEMA_VERSION = 1
PATCH_SERIES_SCHEMA_VERSION = 2
MULTI_FILE_PATCH_SCHEMA_VERSION = 3
PATCH_NAME_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,127}\.patch$")
MAX_PATCH_BYTES = 32 * 1024 * 1024
PATCH_COPY_CHUNK_BYTES = 1024 * 1024


class PatchApplicationError(ValueError):
    """Prepared candidate identity or exact patch application is invalid."""


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def digest_bytes(contents: bytes) -> str:
    return hashlib.sha256(contents).hexdigest()


def _json_unique(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise PatchApplicationError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def _reject_json_constant(value: str) -> None:
    raise PatchApplicationError(f"non-standard JSON constant: {value}")


def _parse_manifest(contents: bytes) -> tuple[dict[str, Any], str]:
    try:
        data = json.loads(
            contents.decode("utf-8"),
            object_pairs_hook=_json_unique,
            parse_constant=_reject_json_constant,
        )
    except (UnicodeError, json.JSONDecodeError) as error:
        raise PatchApplicationError(f"cannot read candidate manifest: {error}") from error
    if (
        not isinstance(data, dict)
        or type(data.get("schema_version")) is not int
        or data["schema_version"] not in (
            SCHEMA_VERSION, PATCH_SERIES_SCHEMA_VERSION, MULTI_FILE_PATCH_SCHEMA_VERSION
        )
    ):
        raise PatchApplicationError("unsupported or malformed candidate manifest")
    if not isinstance(data.get("candidate_id"), str):
        raise PatchApplicationError("manifest candidate_id is missing")
    required_hashes = [] if data["schema_version"] == MULTI_FILE_PATCH_SCHEMA_VERSION else [
        "preimage_sha256", "postimage_sha256"
    ]
    if data["schema_version"] in (SCHEMA_VERSION, MULTI_FILE_PATCH_SCHEMA_VERSION):
        required_hashes.append("patch_sha256")
    for key in required_hashes:
        value = data.get(key)
        if not isinstance(value, str) or re.fullmatch(r"[0-9a-f]{64}", value) is None:
            raise PatchApplicationError(f"manifest {key} is missing or malformed")
    return data, hashlib.sha256(contents).hexdigest()


def _patch_series(record: dict[str, Any], candidate_root: Path,
                  pinned_candidate_fd: int | None = None
                  ) -> tuple[list[dict[str, Any]], tempfile.TemporaryDirectory]:
    candidate_fd = (os.dup(pinned_candidate_fd) if pinned_candidate_fd is not None
                    else _open_directory(candidate_root))
    cache: tempfile.TemporaryDirectory | None = None
    try:
        cache = tempfile.TemporaryDirectory(prefix="rustui-p3k-patches-")
        cache_root = Path(cache.name)
        if record["schema_version"] == SCHEMA_VERSION:
            cache_path = cache_root / "00000000.patch"
            patch_hash = _cache_patch_at(candidate_fd, "approved.patch", cache_path)
            if patch_hash != record["patch_sha256"]:
                raise PatchApplicationError("approved patch hash differs from manifest")
            return [{
                "path": candidate_root / "approved.patch",
                "cache_path": cache_path,
                "sha256": record["patch_sha256"],
                "preimage_sha256": record.get("preimage_sha256"),
                "postimage_sha256": record.get("postimage_sha256"),
            }], cache

        entries = record.get("patches")
        if not isinstance(entries, list) or not entries:
            raise PatchApplicationError("patch series must contain at least one patch")
        names: set[str] = set()
        patches: list[dict[str, Any]] = []
        descriptors: list[str] = []
        for entry in entries:
            if not isinstance(entry, dict):
                raise PatchApplicationError("patch series entry must be an object")
            expected_fields = {"path", "sha256", "preimage_sha256", "postimage_sha256"}
            if record["schema_version"] == MULTI_FILE_PATCH_SCHEMA_VERSION:
                expected_fields.add("file")
            if set(entry) != expected_fields:
                raise PatchApplicationError("patch series entry has missing or unknown fields")
            name = entry.get("path")
            if not isinstance(name, str) or PATCH_NAME_RE.fullmatch(name) is None or name in names:
                raise PatchApplicationError("patch series paths must be unique safe patch filenames")
            names.add(name)
            hashes: dict[str, str] = {}
            for key in ("sha256", "preimage_sha256", "postimage_sha256"):
                value = entry.get(key)
                if not isinstance(value, str) or re.fullmatch(r"[0-9a-f]{64}", value) is None:
                    raise PatchApplicationError(f"patch series {key} is missing or malformed")
                hashes[key] = value
            package_path = "src/scene.rs"
            if record["schema_version"] == MULTI_FILE_PATCH_SCHEMA_VERSION:
                package_path = _safe_package_path(entry.get("file"))
            cache_path = cache_root / f"{len(patches):08d}.patch"
            patch_hash = _cache_patch_at(candidate_fd, name, cache_path)
            if patch_hash != hashes["sha256"]:
                raise PatchApplicationError(f"patch hash differs from manifest: {name}")
            descriptors.append(f"{name}:{hashes['sha256']}")
            patches.append({
                "path": candidate_root / name,
                "cache_path": cache_path,
                "file": package_path,
                **hashes,
            })
        series_hash = hashlib.sha256("\n".join(descriptors).encode("utf-8")).hexdigest()
        if record.get("patch_sha256") != series_hash:
            raise PatchApplicationError("patch series hash differs from manifest")
        if record["schema_version"] != MULTI_FILE_PATCH_SCHEMA_VERSION:
            if patches[0]["preimage_sha256"] != record["preimage_sha256"]:
                raise PatchApplicationError("patch series initial preimage differs from manifest")
            if patches[-1]["postimage_sha256"] != record["postimage_sha256"]:
                raise PatchApplicationError("patch series final postimage hash differs from manifest")
        return patches, cache
    except BaseException:
        if cache is not None:
            cache.cleanup()
        raise
    finally:
        os.close(candidate_fd)


def _cache_patch_at(root_fd: int, relative: str, cache_path: Path) -> str:
    """Hash and cache exact patch bytes without retaining the whole series in RAM."""
    fd = _open_file_at(root_fd, relative, os.O_RDONLY | getattr(os, "O_NONBLOCK", 0))
    digest = hashlib.sha256()
    with os.fdopen(fd, "rb") as source, cache_path.open("xb") as cached:
        before = os.fstat(source.fileno())
        if before.st_size > MAX_PATCH_BYTES:
            raise PatchApplicationError(
                f"patch exceeds {MAX_PATCH_BYTES} byte limit: {relative}"
            )
        total = 0
        for block in iter(lambda: source.read(PATCH_COPY_CHUNK_BYTES), b""):
            total += len(block)
            if total > MAX_PATCH_BYTES:
                raise PatchApplicationError(
                    f"patch exceeds {MAX_PATCH_BYTES} byte limit: {relative}"
                )
            digest.update(block)
            cached.write(block)
        after = os.fstat(source.fileno())
    identity = (before.st_dev, before.st_ino)
    version = (before.st_size, before.st_mtime_ns, before.st_ctime_ns)
    if (identity != (after.st_dev, after.st_ino)
            or version != (after.st_size, after.st_mtime_ns, after.st_ctime_ns)
            or total != before.st_size):
        raise PatchApplicationError(f"patch changed while being read: {relative}")
    return digest.hexdigest()


def _read_cached_patch(patch: dict[str, Any]) -> bytes:
    with patch["cache_path"].open("rb") as stream:
        contents = stream.read(MAX_PATCH_BYTES + 1)
    if len(contents) > MAX_PATCH_BYTES:
        raise PatchApplicationError("cached patch exceeds size limit")
    if digest_bytes(contents) != patch["sha256"]:
        raise PatchApplicationError("cached patch hash differs from manifest")
    return contents


def _safe_package_path(value: Any) -> str:
    if not isinstance(value, str) or not value or "\\" in value or "\x00" in value:
        raise PatchApplicationError("patch file must be a safe package-relative path")
    posix = PurePosixPath(value)
    windows = PureWindowsPath(value)
    if (posix.is_absolute() or windows.is_absolute() or windows.drive
            or any(part in ("", ".", "..") for part in value.split("/"))):
        raise PatchApplicationError("patch file must be a safe package-relative path")
    return posix.as_posix()


def _inventory(root: Path) -> dict[str, str]:
    if root.is_symlink() or root.parent.is_symlink() or not root.is_dir():
        raise PatchApplicationError(f"candidate package root must be a real directory: {root}")
    try:
        root_fd = os.open(root, _directory_flags())
    except OSError as error:
        raise PatchApplicationError(f"candidate package root must be a real directory: {root}") from error
    try:
        return dict(sorted(_inventory_fd(root_fd).items()))
    finally:
        os.close(root_fd)


def _tree_hash(files: dict[str, str]) -> str:
    # The preparation manifest used json.dumps(sort_keys=True) with default separators.
    return hashlib.sha256(json.dumps(files, sort_keys=True).encode()).hexdigest()


def _directory_flags() -> int:
    required = (os.open, os.mkdir, os.stat, os.unlink, os.rmdir, os.link)
    if not hasattr(os, "O_NOFOLLOW") or any(call not in os.supports_dir_fd for call in required):
        raise PatchApplicationError("platform lacks descriptor-relative no-follow directory operations")
    return (os.O_RDONLY | getattr(os, "O_NONBLOCK", 0)
            | getattr(os, "O_DIRECTORY", 0) | os.O_NOFOLLOW)


def _open_directory(path: Path) -> int:
    """Open an absolute directory by walking every component without symlinks."""
    flags = _directory_flags()
    absolute = Path(os.path.abspath(path))
    current_fd: int | None = None
    try:
        parts = absolute.parts[1:]
        if (len(parts) >= 2 and parts[0] == "dev" and parts[1] == "fd"
                and len(parts) >= 3 and parts[2].isdigit()):
            current_fd = os.dup(int(parts[2]))
            parts = parts[3:]
        else:
            current_fd = os.open(absolute.anchor, flags)
        for part in parts:
            child_fd = os.open(part, flags, dir_fd=current_fd)
            os.close(current_fd)
            current_fd = child_fd
            if not stat.S_ISDIR(os.fstat(current_fd).st_mode):
                raise PatchApplicationError(f"directory path contains a non-directory: {part}")
        result_fd = current_fd
        current_fd = None
        return result_fd
    except PatchApplicationError:
        raise
    except OSError as error:
        raise PatchApplicationError(f"directory cannot be opened without symlinks: {path}") from error
    finally:
        if current_fd is not None:
            os.close(current_fd)


def _open_directory_at(parent_fd: int, relative: str) -> int:
    """Open a descendant directory using only no-follow descriptor operations."""
    parts = relative.split("/")
    if not parts or any(part in ("", ".", "..") for part in parts):
        raise PatchApplicationError("directory path must be safe and relative")
    current_fd = os.dup(parent_fd)
    try:
        for part in parts:
            child_fd = os.open(part, _directory_flags(), dir_fd=current_fd)
            os.close(current_fd)
            current_fd = child_fd
        result_fd = current_fd
        current_fd = -1
        return result_fd
    except OSError as error:
        raise PatchApplicationError(f"directory cannot be opened safely: {relative}") from error
    finally:
        if current_fd >= 0:
            os.close(current_fd)


def _verify_directory_path(path: Path, expected_fd: int, label: str) -> None:
    """Reject a pathname that no longer names its pinned directory."""
    try:
        named_fd = _open_directory(path)
    except PatchApplicationError as error:
        raise PatchApplicationError(f"{label} changed during patch check") from error
    try:
        named = os.fstat(named_fd)
        expected = os.fstat(expected_fd)
        if (named.st_dev, named.st_ino) != (expected.st_dev, expected.st_ino):
            raise PatchApplicationError(f"{label} changed during patch check")
    finally:
        os.close(named_fd)


def _verify_child_identity(parent_fd: int, name: str, expected_fd: int, label: str) -> None:
    try:
        named_fd = os.open(name, _directory_flags(), dir_fd=parent_fd)
    except OSError as error:
        raise PatchApplicationError(f"{label} changed during patch check") from error
    try:
        named = os.fstat(named_fd)
        expected = os.fstat(expected_fd)
        if (named.st_dev, named.st_ino) != (expected.st_dev, expected.st_ino):
            raise PatchApplicationError(f"{label} changed during patch check")
    finally:
        os.close(named_fd)


def _open_or_create_directory_at(parent_fd: int, name: str, label: str) -> int:
    flags = _directory_flags()
    try:
        child_fd = os.open(name, flags, dir_fd=parent_fd)
    except FileNotFoundError:
        try:
            os.mkdir(name, 0o755, dir_fd=parent_fd)
        except FileExistsError:
            # A racing entry is accepted only if the following no-follow open
            # proves it is a real directory.
            pass
        try:
            child_fd = os.open(name, flags, dir_fd=parent_fd)
        except OSError as error:
            raise PatchApplicationError(f"{label} cannot be a symlink or non-directory") from error
    except OSError as error:
        raise PatchApplicationError(f"{label} cannot be a symlink or non-directory") from error
    if not stat.S_ISDIR(os.fstat(child_fd).st_mode):
        os.close(child_fd)
        raise PatchApplicationError(f"{label} must be a directory")
    return child_fd


def _open_file_at(root_fd: int, relative: str, flags: int) -> int:
    parts = relative.split("/")
    current_fd = os.dup(root_fd)
    file_fd: int | None = None
    try:
        for part in parts[:-1]:
            child_fd = os.open(part, _directory_flags(), dir_fd=current_fd)
            os.close(current_fd)
            current_fd = child_fd
        file_fd = os.open(parts[-1], flags | os.O_NOFOLLOW, dir_fd=current_fd)
        if not stat.S_ISREG(os.fstat(file_fd).st_mode):
            raise PatchApplicationError(f"copied patch target is not a regular file: {relative}")
        result_fd = file_fd
        file_fd = None
        return result_fd
    except PatchApplicationError:
        raise
    except OSError as error:
        raise PatchApplicationError(f"file cannot be opened safely: {relative}") from error
    finally:
        if file_fd is not None:
            os.close(file_fd)
        os.close(current_fd)


def _read_file_at(root_fd: int, relative: str) -> bytes:
    fd = _open_file_at(root_fd, relative, os.O_RDONLY | getattr(os, "O_NONBLOCK", 0))
    with os.fdopen(fd, "rb") as stream:
        return stream.read()


def _hash_file_at(root_fd: int, relative: str) -> str:
    digest = hashlib.sha256()
    fd = _open_file_at(root_fd, relative, os.O_RDONLY | getattr(os, "O_NONBLOCK", 0))
    with os.fdopen(fd, "rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def _write_file_at(root_fd: int, relative: str, contents: bytes) -> None:
    mode_fd = _open_file_at(root_fd, relative, os.O_RDONLY)
    try:
        original_mode = stat.S_IMODE(os.fstat(mode_fd).st_mode)
        needs_write_bit = not original_mode & stat.S_IWUSR
        if needs_write_bit:
            os.fchmod(mode_fd, original_mode | stat.S_IWUSR)
    finally:
        os.close(mode_fd)

    try:
        fd = _open_file_at(root_fd, relative, os.O_WRONLY | os.O_TRUNC)
        with os.fdopen(fd, "wb") as stream:
            stream.write(contents)
            stream.flush()
    finally:
        if needs_write_bit:
            mode_fd = _open_file_at(root_fd, relative, os.O_RDONLY)
            try:
                os.fchmod(mode_fd, original_mode)
            finally:
                os.close(mode_fd)


def _copy_directory_fd(source_fd: int, destination_fd: int,
                       budget: dict[str, int] | None = None) -> None:
    """Copy a regular-file tree into a pinned destination directory."""
    if budget is None:
        budget = {"entries": 0, "bytes": 0}
    directory_flags = _directory_flags()

    def frame(fd: int, parent_identity: tuple[int, int] | None = None,
              destination_parent_identity: tuple[int, int] | None = None,
              name: str | None = None) -> dict[str, Any]:
        info = os.fstat(fd)
        if not stat.S_ISDIR(info.st_mode):
            raise PatchApplicationError("source package entry is not a directory")
        names = os.listdir(fd)
        budget["entries"] += len(names)
        if budget["entries"] > MAX_SNAPSHOT_ENTRIES:
            raise PatchApplicationError(
                f"package copy exceeds {MAX_SNAPSHOT_ENTRIES} entries"
            )
        return {
            "identity": (info.st_dev, info.st_ino),
            "metadata": (info.st_mtime_ns, info.st_ctime_ns),
            "parent_identity": parent_identity,
            "destination_parent_identity": destination_parent_identity,
            "name": name,
            "names": names,
            "index": 0,
            "mode": stat.S_IMODE(info.st_mode),
        }

    source_current = os.dup(source_fd)
    destination_current = os.dup(destination_fd)
    stack = []
    try:
        stack.append(frame(source_current))
        while stack:
            current = stack[-1]
            if current["index"] == len(current["names"]):
                source_info = os.fstat(source_current)
                if ((source_info.st_dev, source_info.st_ino) != current["identity"]
                        or (source_info.st_mtime_ns, source_info.st_ctime_ns)
                        != current["metadata"]):
                    raise PatchApplicationError("source package directory changed during copy")
                if stack[-1]["name"] is not None:
                    os.fchmod(destination_current, current["mode"])
                stack.pop()
                if not stack:
                    break

                name = current["name"]
                source_parent = os.open("..", directory_flags, dir_fd=source_current)
                try:
                    destination_parent = os.open("..", directory_flags, dir_fd=destination_current)
                except BaseException:
                    os.close(source_parent)
                    raise
                try:
                    source_parent_info = os.fstat(source_parent)
                    destination_parent_info = os.fstat(destination_parent)
                    if ((source_parent_info.st_dev, source_parent_info.st_ino)
                            != current["parent_identity"]):
                        raise PatchApplicationError("source package parent changed during copy")
                    if ((destination_parent_info.st_dev, destination_parent_info.st_ino)
                            != current["destination_parent_identity"]):
                        raise PatchApplicationError("destination package parent changed during copy")
                    named_source = os.stat(name, dir_fd=source_parent, follow_symlinks=False)
                    named_destination = os.stat(name, dir_fd=destination_parent,
                                                follow_symlinks=False)
                    if (not stat.S_ISDIR(named_source.st_mode)
                            or (named_source.st_dev, named_source.st_ino) != current["identity"]
                            or not stat.S_ISDIR(named_destination.st_mode)
                            or (named_destination.st_dev, named_destination.st_ino)
                            != current["destination_identity"]):
                        raise PatchApplicationError("package directory changed during copy")
                except BaseException:
                    os.close(source_parent)
                    os.close(destination_parent)
                    raise
                os.close(source_current)
                os.close(destination_current)
                source_current, destination_current = source_parent, destination_parent
                continue

            name = current["names"][current["index"]]
            current["index"] += 1
            try:
                observed = os.stat(name, dir_fd=source_current, follow_symlinks=False)
            except OSError as error:
                raise PatchApplicationError(f"source package entry changed during copy: {name}") from error
            if stat.S_ISDIR(observed.st_mode):
                child_source = os.open(name, directory_flags, dir_fd=source_current)
                try:
                    opened = os.fstat(child_source)
                    if (opened.st_dev, opened.st_ino) != (observed.st_dev, observed.st_ino):
                        raise PatchApplicationError(f"source package directory changed during copy: {name}")
                    os.mkdir(name, 0o755, dir_fd=destination_current)
                    child_destination = os.open(name, directory_flags, dir_fd=destination_current)
                    try:
                        destination_info = os.fstat(child_destination)
                        destination_parent_info = os.fstat(destination_current)
                        child_frame = frame(
                            child_source, current["identity"],
                            (destination_parent_info.st_dev, destination_parent_info.st_ino), name,
                        )
                        child_frame["destination_identity"] = (
                            destination_info.st_dev, destination_info.st_ino
                        )
                        stack.append(child_frame)
                    except BaseException:
                        os.close(child_destination)
                        raise
                except BaseException:
                    os.close(child_source)
                    raise
                os.close(source_current)
                os.close(destination_current)
                source_current, destination_current = child_source, child_destination
                continue

            source_file_fd = os.open(
                name, os.O_RDONLY | getattr(os, "O_NONBLOCK", 0) | os.O_NOFOLLOW,
                dir_fd=source_current,
            )
            try:
                source_mode = os.fstat(source_file_fd).st_mode
                if (not stat.S_ISREG(source_mode)
                        or (os.fstat(source_file_fd).st_dev, os.fstat(source_file_fd).st_ino)
                        != (observed.st_dev, observed.st_ino)):
                    raise PatchApplicationError(f"source package entry is not a regular file: {name}")
                source_size = os.fstat(source_file_fd).st_size
                if budget["bytes"] + source_size > MAX_SNAPSHOT_BYTES:
                    raise PatchApplicationError(
                        f"package copy exceeds {MAX_SNAPSHOT_BYTES} regular-file bytes"
                    )
                destination_file_fd = os.open(
                    name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                    stat.S_IMODE(source_mode), dir_fd=destination_current,
                )
                try:
                    destination_stream = os.fdopen(destination_file_fd, "wb")
                    destination_file_fd = None
                    with destination_stream, os.fdopen(os.dup(source_file_fd), "rb") as source_stream:
                        while True:
                            remaining = MAX_SNAPSHOT_BYTES - budget["bytes"]
                            block = source_stream.read(min(1024 * 1024, remaining + 1))
                            if not block:
                                break
                            if budget["bytes"] + len(block) > MAX_SNAPSHOT_BYTES:
                                raise PatchApplicationError(
                                    f"package copy exceeds {MAX_SNAPSHOT_BYTES} regular-file bytes"
                                )
                            destination_stream.write(block)
                            budget["bytes"] += len(block)
                        destination_stream.flush()
                        os.fchmod(destination_stream.fileno(), stat.S_IMODE(source_mode))
                finally:
                    if destination_file_fd is not None:
                        os.close(destination_file_fd)
            finally:
                os.close(source_file_fd)
    finally:
        os.close(source_current)
        os.close(destination_current)


def _inventory_fd(root_fd: int) -> dict[str, str]:
    try:
        with PinnedDir.from_fd(root_fd) as pinned:
            return pinned.inventory(
                max_entries=MAX_SNAPSHOT_ENTRIES,
                max_bytes=MAX_SNAPSHOT_BYTES,
                reject_hardlinks=False,
            )
    except CandidateFSError as error:
        raise PatchApplicationError(str(error)) from error


def _remove_tree_at(parent_fd: int, name: str,
                    expected_identity: tuple[int, int] | None = None) -> None:
    """Remove one disposable subtree without following entries or path aliases."""
    try:
        with PinnedDir.from_fd(parent_fd) as parent:
            parent.remove_tree(name, expected_identity=expected_identity,
                               max_entries=MAX_SNAPSHOT_ENTRIES)
    except CandidateFSError as error:
        message = str(error).replace("during removal", "during rollback")
        raise PatchApplicationError(message) from error


def _write_result_new_at(directory_fd: int, result: dict[str, Any]) -> None:
    temporary = f".result.json.{uuid.uuid4().hex}.tmp"
    created = False
    try:
        fd = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                     0o600, dir_fd=directory_fd)
        with os.fdopen(fd, "w", encoding="utf-8") as stream:
            created = True
            json.dump(result, stream, indent=2, sort_keys=True)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        os.link(temporary, "result.json", src_dir_fd=directory_fd,
                dst_dir_fd=directory_fd, follow_symlinks=False)
    finally:
        if created:
            os.unlink(temporary, dir_fd=directory_fd)


def _write_result_new(path: Path, result: dict[str, Any]) -> None:
    temporary = path.with_name(f".{path.name}.{uuid.uuid4().hex}.tmp")
    created = False
    try:
        with temporary.open("x", encoding="utf-8") as stream:
            created = True
            json.dump(result, stream, indent=2, sort_keys=True)
            stream.write("\n")
        os.link(temporary, path)
    finally:
        if created:
            temporary.unlink(missing_ok=True)


def _lf_lines(text: str) -> list[str]:
    """Split physical LF-delimited lines without treating Unicode separators as newlines."""
    parts = text.split("\n")
    return [part + "\n" for part in parts[:-1]] + ([parts[-1]] if parts[-1] else [])


def _strict_apply(source: bytes, patch: bytes, package_path: str = "src/scene.rs") -> bytes:
    try:
        lines = _lf_lines(patch.decode("utf-8"))
        original = _lf_lines(source.decode("utf-8"))
    except UnicodeDecodeError as error:
        raise PatchApplicationError("patch and target must be UTF-8 text") from error
    if any(not line.endswith("\n") for line in lines):
        raise PatchApplicationError("unterminated unified-diff line")
    expected_path = f"vendor/{PACKAGE_DIR}/{package_path}"
    expected_headers = f"--- a/{expected_path}\n+++ b/{expected_path}\n"
    if len(lines) < 3 or "".join(lines[:2]) != expected_headers:
        raise PatchApplicationError(f"patch must target only package file {package_path}")

    output: list[str] = []
    old_cursor = 0
    new_cursor = 0
    index = 2
    saw_hunk = False
    while index < len(lines):
        match = HUNK_RE.fullmatch(lines[index])
        if match is None:
            raise PatchApplicationError(f"unexpected unified-diff line {index + 1}")
        saw_hunk = True
        old_start, old_count, new_start, new_count = match.groups()
        try:
            old_start_i = int(old_start)
            old_count_i = int(old_count) if old_count is not None else 1
            new_start_i = int(new_start)
            new_count_i = int(new_count) if new_count is not None else 1
        except ValueError as error:
            raise PatchApplicationError("unified-diff hunk line number or count is too large") from error
        hunk_old_index = old_start_i if old_count_i == 0 else old_start_i - 1
        hunk_new_index = new_start_i if new_count_i == 0 else new_start_i - 1
        if hunk_old_index < old_cursor or hunk_old_index > len(original):
            raise PatchApplicationError("hunk has an invalid or shifted preimage line")
        untouched = original[old_cursor:hunk_old_index]
        output.extend(untouched)
        new_cursor += len(untouched)
        if hunk_new_index != new_cursor:
            raise PatchApplicationError("hunk new-file position is inconsistent")
        old_cursor = hunk_old_index
        consumed_old = 0
        produced_new = 0
        index += 1
        while index < len(lines) and not lines[index].startswith("@@ "):
            row = lines[index]
            if row.startswith("\\"):
                raise PatchApplicationError("no-newline patch markers are unsupported")
            if not row or row[0] not in " +-":
                raise PatchApplicationError(f"invalid unified-diff row {index + 1}")
            marker, body = row[0], row[1:]
            if marker in " -":
                if old_cursor >= len(original) or original[old_cursor] != body:
                    raise PatchApplicationError(
                        f"exact preimage mismatch at source line {old_cursor + 1}"
                    )
                old_cursor += 1
                consumed_old += 1
            if marker in " +":
                output.append(body)
                produced_new += 1
            index += 1
        if consumed_old != old_count_i or produced_new != new_count_i:
            raise PatchApplicationError("unified-diff hunk line counts do not match its header")
        if consumed_old == 0 and produced_new == 0:
            raise PatchApplicationError("empty unified-diff hunk")
        new_cursor += produced_new
    if not saw_hunk:
        raise PatchApplicationError("patch contains no hunks")
    output.extend(original[old_cursor:])
    applied = "".join(output).encode("utf-8")
    if applied == source:
        raise PatchApplicationError("patch does not change its target")
    return applied


def _resolve_input(path: Path, root: Path, lexical_root: Path) -> Path:
    lexical_root = Path(os.path.abspath(lexical_root))
    absolute = Path(os.path.abspath(path))
    try:
        relative = absolute.relative_to(lexical_root)
    except ValueError as error:
        raise PatchApplicationError("candidate input is outside the repository") from error
    current = lexical_root
    for part in relative.parts:
        current = current / part
        if current.is_symlink():
            raise PatchApplicationError(f"symlink rejected in candidate input path: {current}")
    resolved = absolute.resolve(strict=True)
    try:
        resolved.relative_to(root)
    except ValueError as error:
        raise PatchApplicationError("candidate input resolves outside the repository") from error
    return resolved


def apply_candidate_patch(manifest_path: Path, run_id: str, repository_root: Path) -> dict[str, Any]:
    if RUN_ID_RE.fullmatch(run_id) is None or ".." in run_id:
        raise PatchApplicationError("run_id must be a safe 1-64 character identifier")
    repo_fd_text = os.environ.get("RUSTUI_P3_REPO_ROOT_FD")
    candidate_fd_text = os.environ.get("RUSTUI_P3_CANDIDATE_ROOT_FD")
    candidate_parent_fd_text = os.environ.get("RUSTUI_P3_CANDIDATE_PARENT_FD")
    candidate_id_env = os.environ.get("RUSTUI_P3_CANDIDATE_ID")
    fd_context_present = any((repo_fd_text, candidate_fd_text,
                              candidate_parent_fd_text, candidate_id_env))
    pinned = all((repo_fd_text, candidate_fd_text, candidate_parent_fd_text, candidate_id_env))
    if fd_context_present and not pinned:
        raise PatchApplicationError("inherited candidate descriptor context is incomplete")
    if pinned:
        try:
            repo_fd = int(repo_fd_text)
            candidate_fd = int(candidate_fd_text)
            candidate_parent_fd = int(candidate_parent_fd_text)
        except (TypeError, ValueError) as error:
            raise PatchApplicationError("inherited candidate directory descriptors are invalid") from error
        if RUN_ID_RE.fullmatch(candidate_id_env) is None or ".." in candidate_id_env:
            raise PatchApplicationError("inherited candidate identity is unsafe")
        repo = Path("/dev/fd") / str(repo_fd)
        repo_lexical = repo
        candidate_root = Path("/dev/fd") / str(candidate_fd)
        candidate_id = candidate_id_env
        expected_manifest = PurePosixPath("target/headstart/candidates") / candidate_id / "manifest.json"
        if PurePosixPath(manifest_path.as_posix()) != expected_manifest:
            raise PatchApplicationError("manifest path does not match the pinned candidate identity")
        try:
            parent_info = os.fstat(candidate_parent_fd)
            candidate_info = os.fstat(candidate_fd)
        except OSError as error:
            raise PatchApplicationError("inherited candidate directory descriptor is closed") from error
        if not stat.S_ISDIR(parent_info.st_mode) or not stat.S_ISDIR(candidate_info.st_mode):
            raise PatchApplicationError("inherited candidate descriptors must name directories")
        _verify_child_identity(candidate_parent_fd, candidate_id, candidate_fd,
                               "candidate directory")
        candidate_manifest_fd = os.dup(candidate_fd)
        try:
            manifest_contents = _read_file_at(candidate_manifest_fd, "manifest.json")
        finally:
            os.close(candidate_manifest_fd)
        manifest = candidate_root / "manifest.json"
    else:
        repo = repository_root.resolve(strict=True)
        repo_lexical = Path(os.path.abspath(repository_root))
        candidates_root = repo / "target/headstart/candidates"
        manifest = _resolve_input(manifest_path, repo, repo_lexical)
        candidate_root = manifest.parent
        candidate_root_lexical = repo_lexical / manifest.relative_to(repo)
        candidate_root_lexical = candidate_root_lexical.parent
        manifest_root_fd = _open_directory(candidate_root)
        try:
            manifest_contents = _read_file_at(manifest_root_fd, manifest.name)
        finally:
            os.close(manifest_root_fd)
    record, manifest_hash = _parse_manifest(manifest_contents)
    if pinned:
        if record["candidate_id"] != candidate_id:
            raise PatchApplicationError("manifest path and pinned candidate identity do not match")
    elif candidate_root.parent != candidates_root or candidate_root.name != record["candidate_id"]:
        raise PatchApplicationError("manifest path and candidate identity do not match")
    snapshots = record.get("snapshots")
    if not isinstance(snapshots, dict):
        raise PatchApplicationError("manifest snapshots are missing")
    baseline_record = snapshots.get("baseline")
    candidate_record = snapshots.get("candidate")
    if not isinstance(baseline_record, dict) or not isinstance(candidate_record, dict):
        raise PatchApplicationError("manifest must contain baseline and candidate snapshots")

    if pinned:
        if baseline_record.get("source") != "baseline/source" or candidate_record.get("source") != "candidate/source":
            raise PatchApplicationError("snapshot source paths do not match the pinned candidate layout")
        baseline_root = candidate_root / "baseline/source"
        candidate_source = candidate_root / "candidate/source"
    else:
        baseline_root = _resolve_input(Path(baseline_record["source"]), repo, repo_lexical)
        candidate_source = _resolve_input(Path(candidate_record["source"]), repo, repo_lexical)
    if baseline_root != candidate_root / "baseline/source":
        raise PatchApplicationError("baseline source path does not match candidate layout")
    if candidate_source != candidate_root / "candidate/source":
        raise PatchApplicationError("candidate source path does not match candidate layout")
    baseline_package = baseline_root / "path-sources" / PACKAGE_DIR
    candidate_package = candidate_source / "path-sources" / PACKAGE_DIR
    baseline_package_fd = candidate_package_fd = None
    patch_cache: tempfile.TemporaryDirectory | None = None
    try:
        if pinned:
            baseline_package_fd = _open_directory_at(
                candidate_fd, f"baseline/source/path-sources/{PACKAGE_DIR}"
            )
            candidate_package_fd = _open_directory_at(
                candidate_fd, f"candidate/source/path-sources/{PACKAGE_DIR}"
            )
            baseline_inventory = _inventory_fd(baseline_package_fd)
            candidate_inventory = _inventory_fd(candidate_package_fd)
        else:
            if not baseline_package.is_dir() or not candidate_package.is_dir():
                raise PatchApplicationError("prepared baseline/candidate package copies are missing")
            baseline_package_fd = candidate_package_fd = None
            baseline_inventory = _inventory(baseline_package)
            candidate_inventory = _inventory(candidate_package)
        if _tree_hash(baseline_inventory) != baseline_record.get("path_tree_sha256"):
            raise PatchApplicationError("baseline package tree differs from preparation manifest")
        if _tree_hash(candidate_inventory) != candidate_record.get("path_tree_sha256"):
            raise PatchApplicationError("candidate package tree differs from preparation manifest")

        patches, patch_cache = _patch_series(
            record, candidate_root, candidate_fd if pinned else None
        )
        is_multi_file = record["schema_version"] == MULTI_FILE_PATCH_SCHEMA_VERSION
        file_evidence: list[dict[str, str]] = []
        if is_multi_file:
            if baseline_inventory.keys() != candidate_inventory.keys():
                raise PatchApplicationError("multi-file patch series cannot add or delete package files")
            changed_files = {
                path for path in baseline_inventory
                if baseline_inventory[path] != candidate_inventory[path]
            }
            declared_files = {patch["file"] for patch in patches}
            if not changed_files or declared_files != changed_files:
                raise PatchApplicationError(
                    "patch series file paths must exactly match changed package-relative files"
                )
            current_hashes = dict(baseline_inventory)
            first_hashes: dict[str, str] = {}
            for patch in patches:
                package_file = patch["file"]
                if package_file not in baseline_inventory:
                    raise PatchApplicationError(f"patch target is missing from baseline package: {package_file}")
                current_hash = current_hashes[package_file]
                if current_hash != patch["preimage_sha256"]:
                    raise PatchApplicationError(f"patch preimage hash differs from manifest: {package_file}")
                first_hashes.setdefault(package_file, current_hash)
                current_hashes[package_file] = patch["postimage_sha256"]
            for package_file in sorted(changed_files):
                if current_hashes[package_file] != candidate_inventory[package_file]:
                    raise PatchApplicationError(f"candidate postimage hash differs from manifest: {package_file}")
                file_evidence.append({
                    "path": package_file,
                    "preimage_sha256": first_hashes[package_file],
                    "postimage_sha256": current_hashes[package_file],
                })
        else:
            baseline_scene = baseline_package / "src/scene.rs"
            candidate_scene = candidate_package / "src/scene.rs"
            baseline_scene_hash = (
                _hash_file_at(baseline_package_fd, "src/scene.rs")
                if baseline_package_fd is not None else _sha256(baseline_scene)
            )
            candidate_scene_hash = (
                _hash_file_at(candidate_package_fd, "src/scene.rs")
                if candidate_package_fd is not None else _sha256(candidate_scene)
            )
            if baseline_scene_hash != record["preimage_sha256"]:
                raise PatchApplicationError("baseline scene preimage hash differs from manifest")
            if candidate_scene_hash != record["postimage_sha256"]:
                raise PatchApplicationError("candidate scene postimage hash differs from manifest")

    except BaseException:
        if patch_cache is not None:
            patch_cache.cleanup()
        for descriptor in (candidate_package_fd, baseline_package_fd):
            if descriptor is not None:
                os.close(descriptor)
        raise

    patch_checks_root = candidate_root / "patch-checks"
    output_root = patch_checks_root / run_id
    package_copy = output_root / "package"
    inherited_candidate_fd = candidate_fd if pinned else None
    candidate_fd = None
    patch_checks_fd: int | None = None
    output_fd: int | None = None
    package_fd: int | None = None
    package_identity: tuple[int, int] | None = None
    try:
        candidate_fd = (
            os.dup(inherited_candidate_fd)
            if inherited_candidate_fd is not None
            else _open_directory(candidate_root)
        )
        patch_checks_fd = _open_or_create_directory_at(
            candidate_fd, "patch-checks", "patch-check destination root"
        )
        try:
            os.mkdir(run_id, 0o755, dir_fd=patch_checks_fd)
        except FileExistsError as error:
            raise PatchApplicationError(
                f"patch-check destination already exists: {output_root}"
            ) from error
        try:
            output_fd = os.open(run_id, _directory_flags(), dir_fd=patch_checks_fd)
        except OSError as error:
            raise PatchApplicationError(
                f"patch-check destination cannot be opened without symlinks: {output_root}"
            ) from error
    except BaseException:
        if patch_cache is not None:
            patch_cache.cleanup()
        for descriptor in (output_fd, patch_checks_fd, candidate_fd,
                           candidate_package_fd, baseline_package_fd):
            if descriptor is not None:
                os.close(descriptor)
        raise

    try:
        status = "rejected"
        error_text: str | None = None
        rollback_status = "not_required"
        try:
            os.mkdir("package", 0o755, dir_fd=output_fd)
            package_fd = os.open("package", _directory_flags(), dir_fd=output_fd)
            package_info = os.fstat(package_fd)
            package_identity = package_info.st_dev, package_info.st_ino
            source_fd = (os.dup(baseline_package_fd) if baseline_package_fd is not None
                         else _open_directory(baseline_package))
            try:
                _copy_directory_fd(source_fd, package_fd)
                os.fchmod(package_fd, stat.S_IMODE(os.fstat(source_fd).st_mode))
            finally:
                os.close(source_fd)

            if is_multi_file:
                staged: dict[str, bytes] = {}
                for patch in patches:
                    relative = _safe_package_path(patch["file"])
                    source = staged.get(relative)
                    if source is None:
                        source = _read_file_at(package_fd, relative)
                        if _hash_file_at(package_fd, relative) != patch["preimage_sha256"]:
                            raise PatchApplicationError(
                                f"copied target preimage differs from manifest: {relative}"
                            )
                    if digest_bytes(source) != patch["preimage_sha256"]:
                        raise PatchApplicationError(
                            f"patch preimage hash differs from manifest: {relative}"
                        )
                    applied = _strict_apply(source, _read_cached_patch(patch), relative)
                    if digest_bytes(applied) != patch["postimage_sha256"]:
                        raise PatchApplicationError(
                            f"patch postimage hash differs from manifest: {relative}"
                        )
                    staged[relative] = applied
                expected_inventory = dict(baseline_inventory)
                expected_inventory.update({path: digest_bytes(data) for path, data in staged.items()})
                if expected_inventory != candidate_inventory:
                    raise PatchApplicationError(
                        "staged multi-file result differs from candidate package inventory"
                    )
                for relative, contents in staged.items():
                    _write_file_at(package_fd, relative, contents)
            else:
                target = "src/scene.rs"
                if _hash_file_at(package_fd, target) != record["preimage_sha256"]:
                    raise PatchApplicationError("copied target preimage differs from manifest")
                for patch in patches:
                    if _hash_file_at(package_fd, target) != patch["preimage_sha256"]:
                        raise PatchApplicationError(
                            f"patch preimage differs from manifest: {patch['path'].name}"
                        )
                    applied = _strict_apply(_read_file_at(package_fd, target),
                                             _read_cached_patch(patch))
                    _write_file_at(package_fd, target, applied)
                    if _hash_file_at(package_fd, target) != patch["postimage_sha256"]:
                        raise PatchApplicationError(
                            f"patch postimage hash differs from manifest: {patch['path'].name}"
                        )
                if _hash_file_at(package_fd, target) != record["postimage_sha256"]:
                    raise PatchApplicationError("applied patch postimage hash differs from manifest")

            if _inventory_fd(package_fd) != candidate_inventory:
                raise PatchApplicationError(
                    "copied postimage inventory differs from candidate package inventory"
                )
            status = "passed"
        except (OSError, PatchApplicationError) as error:
            error_text = str(error)
        finally:
            try:
                if patch_checks_fd is not None:
                    _verify_directory_path(
                        patch_checks_root, patch_checks_fd, "patch-check destination"
                    )
                if output_fd is not None:
                    _verify_directory_path(
                        output_root, output_fd, "patch-check output"
                    )
                if package_fd is not None:
                    _verify_directory_path(
                        package_copy, package_fd, "patch-check package output"
                    )
                if digest_bytes(_read_file_at(candidate_fd, "manifest.json")) != manifest_hash:
                    error_text = "candidate manifest changed during patch check"
                    status = "rejected"
                if pinned:
                    _verify_child_identity(candidate_parent_fd, candidate_id, candidate_fd,
                                           "candidate directory")
                current_baseline = (_inventory_fd(baseline_package_fd)
                                    if baseline_package_fd is not None else _inventory(baseline_package))
                if _tree_hash(current_baseline) != baseline_record.get("path_tree_sha256"):
                    error_text = "baseline package changed during patch check"
                    status = "rejected"
                current_candidate = (_inventory_fd(candidate_package_fd)
                                    if candidate_package_fd is not None else _inventory(candidate_package))
                if _tree_hash(current_candidate) != candidate_record.get("path_tree_sha256"):
                    error_text = "candidate package changed during patch check"
                    status = "rejected"
                if any(
                    _hash_file_at(candidate_fd, patch["path"].name) != patch["sha256"]
                    for patch in patches
                ):
                    error_text = "approved patch series changed during patch check"
                    status = "rejected"
            except (OSError, PatchApplicationError) as error:
                error_text = f"post-check integrity verification failed: {error}"
                status = "rejected"

            if is_multi_file and status != "passed":
                try:
                    if package_fd is not None:
                        os.close(package_fd)
                        package_fd = None
                    try:
                        _remove_tree_at(output_fd, "package", package_identity)
                    except FileNotFoundError:
                        pass
                    rollback_status = "completed"
                except (OSError, PatchApplicationError) as error:
                    rollback_status = "failed"
                    error_text = (
                        f"{error_text}; rollback failed: {error}" if error_text else str(error)
                    )

            result = {
                "schema_version": 1,
                "candidate_id": record["candidate_id"],
                "manifest_sha256": manifest_hash,
                "patch_sha256": record["patch_sha256"],
                "patches": [
                    {
                        "path": patch["path"].name,
                        "sha256": patch["sha256"],
                        "preimage_sha256": patch["preimage_sha256"],
                        "postimage_sha256": patch["postimage_sha256"],
                        **({"file": patch["file"]} if is_multi_file else {}),
                    }
                    for patch in patches
                ],
                "preimage_sha256": record.get("preimage_sha256"),
                "postimage_sha256": record.get("postimage_sha256"),
                "files": file_evidence if is_multi_file else None,
                "rollback_on_failure": is_multi_file,
                "rollback_status": rollback_status,
                "status": status,
                "scope": (
                    "exact ordered patch series applied to a fresh copy of the prepared baseline package only"
                    if len(patches) > 1
                    else "exact patch applied to a fresh copy of the prepared baseline package only"
                ),
                "fuzzy_or_offset_application": False,
                "candidate_promotion": "not_run",
                "output_package": (
                    (PurePosixPath("target/headstart/candidates") / candidate_id
                     / "patch-checks" / run_id / "package").as_posix() if pinned
                    else str(package_copy)
                ) if status == "passed" or not is_multi_file or rollback_status == "failed"
                else None,
                "error": error_text,
            }
            _write_result_new_at(output_fd, result)
    finally:
        if package_fd is not None:
            os.close(package_fd)
        if output_fd is not None:
            os.close(output_fd)
        if patch_checks_fd is not None:
            os.close(patch_checks_fd)
        if candidate_fd is not None:
            os.close(candidate_fd)
        if baseline_package_fd is not None:
            os.close(baseline_package_fd)
        if candidate_package_fd is not None:
            os.close(candidate_package_fd)
        if patch_cache is not None:
            patch_cache.cleanup()

    if status != "passed":
        raise PatchApplicationError(
            f"patch check rejected; see {output_root / 'result.json'}: {error_text}"
        )
    return result

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", type=Path, help="prepared candidate manifest")
    parser.add_argument("--run-id", help="unique output ID; defaults to a UTC timestamp and random suffix")
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[1]
    run_id = args.run_id or datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ-") + uuid.uuid4().hex[:8]
    try:
        report = apply_candidate_patch(args.manifest, run_id, repo)
    except (OSError, KeyError, TypeError, PatchApplicationError) as error:
        print(f"patch application rejected: {error}", file=sys.stderr)
        return 1
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
