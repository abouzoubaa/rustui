#!/usr/bin/env python3
"""Validate paired P3 candidate source snapshots without running commands."""

from __future__ import annotations

import argparse
import errno
import hashlib
import json
import os
import re
import sys
import stat
from pathlib import Path
from typing import Any

try:
    from p3_candidate_fs import CandidateFSError, PinnedDir, relative_parts
except ModuleNotFoundError:  # importlib-based tests load this file outside scripts/ on sys.path
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    from p3_candidate_fs import CandidateFSError, PinnedDir, relative_parts


SCHEMA_VERSION = 1
MAX_MANIFEST_BYTES = 32 * 1024 * 1024
# The largest checked-in prepared tree has about 43k files and is roughly 2 GiB.
# Keep substantial headroom while bounding metadata and I/O per snapshot.
MAX_SNAPSHOT_ENTRIES = 100_000
MAX_SNAPSHOT_BYTES = 4 * 1024 * 1024 * 1024
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
CANDIDATE_ID_RE = re.compile(r"[A-Za-z0-9][A-Za-z0-9._-]{0,63}\Z")


class SnapshotError(ValueError):
    """Invalid candidate snapshot metadata or on-disk contents."""


def _open_absolute_root(path: Path) -> PinnedDir:
    absolute = Path(os.path.abspath(path))
    current = PinnedDir.open_root(absolute.anchor)
    try:
        for part in absolute.parts[1:]:
            try:
                child = PinnedDir.open_child(current, part)
            except CandidateFSError as error:
                cause = error
                while cause.__cause__ is not None:
                    cause = cause.__cause__
                if isinstance(cause, OSError):
                    raise OSError(cause.errno, f"unsafe root path component: {part}") from error
                raise OSError(errno.EIO, f"unsafe root path component: {part}") from error
            current.close()
            current = child
        result = current
        current = None
        return result
    finally:
        if current is not None:
            current.close()


def _inherited_candidate_root(root_arg: Path) -> PinnedDir | None:
    repo_fd = os.environ.get("RUSTUI_P3_REPO_ROOT_FD")
    parent_fd = os.environ.get("RUSTUI_P3_CANDIDATE_PARENT_FD")
    candidate_fd = os.environ.get("RUSTUI_P3_CANDIDATE_ROOT_FD")
    candidate_id = os.environ.get("RUSTUI_P3_CANDIDATE_ID")
    if not any((repo_fd, parent_fd, candidate_fd, candidate_id)):
        return None
    if not all((repo_fd, parent_fd, candidate_fd, candidate_id)):
        raise SnapshotError("incomplete inherited candidate root identity")
    expected = Path("target/headstart/candidates") / candidate_id
    supplied = root_arg if root_arg.is_absolute() else Path.cwd() / root_arg
    try:
        if supplied.relative_to(Path.cwd()) != expected:
            raise SnapshotError("candidate root argument does not match candidate identity")
    except ValueError as error:
        raise SnapshotError("candidate root argument must be repo-relative") from error
    repository = PinnedDir.from_fd(int(repo_fd))
    try:
        parent = PinnedDir.from_fd(int(parent_fd))
        try:
            repository_candidates = PinnedDir.open_child(
                repository, "target/headstart/candidates"
            )
            try:
                if repository_candidates.identity() != parent.identity():
                    raise SnapshotError(
                        "inherited candidate parent does not match repository candidates directory"
                    )
            finally:
                repository_candidates.close()
            identity_root = PinnedDir.open_child(parent, candidate_id)
        finally:
            parent.close()
    finally:
        repository.close()
    try:
        candidate = PinnedDir.from_fd(int(candidate_fd))
    except BaseException:
        identity_root.close()
        raise
    if identity_root.identity() != candidate.identity():
        identity_root.close()
        candidate.close()
        raise SnapshotError("inherited candidate directory does not match candidate parent and ID")
    identity_root.close()
    return candidate


def _unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    value: dict[str, Any] = {}
    for key, item in pairs:
        if key in value:
            raise SnapshotError(f"duplicate JSON key: {key}")
        value[key] = item
    return value


def _reject_constant(value: str) -> Any:
    raise SnapshotError(f"nonstandard JSON constant: {value}")


def _inventory(root: Path, identities: set[tuple[int, int]] | None = None) -> dict[str, str]:
    if (not hasattr(os, "O_NOFOLLOW") or os.open not in os.supports_dir_fd
            or os.stat not in os.supports_dir_fd
            or os.stat not in os.supports_follow_symlinks):
        raise SnapshotError("descriptor-relative no-follow snapshot access is unavailable")

    directory_flags = (os.O_RDONLY | getattr(os, "O_NONBLOCK", 0)
                       | getattr(os, "O_DIRECTORY", 0) | os.O_NOFOLLOW)
    file_flags = os.O_RDONLY | getattr(os, "O_NONBLOCK", 0) | os.O_NOFOLLOW
    absolute = Path(os.path.abspath(root))
    directory_fd = os.open(absolute.anchor, directory_flags)
    try:
        for part in absolute.parts[1:]:
            child_fd = _open_inventory_child(directory_fd, part, directory_flags)
            os.close(directory_fd)
            directory_fd = child_fd
        if not stat.S_ISDIR(os.fstat(directory_fd).st_mode):
            raise SnapshotError(f"source snapshot root is not a directory: {root}")
        return _inventory_fd(directory_fd, identities=identities)
    finally:
        os.close(directory_fd)


def _open_inventory_child(directory_fd: int, name: str, flags: int) -> int:
    parent = PinnedDir.from_fd(directory_fd)
    try:
        if flags & getattr(os, "O_DIRECTORY", 0):
            child = PinnedDir.open_child(parent, name)
            return child.fd
        return parent.open_file(name, flags)
    except CandidateFSError as error:
        cause = error
        while cause.__cause__ is not None:
            cause = cause.__cause__
        if isinstance(cause, OSError) and cause.errno == errno.ELOOP:
            raise SnapshotError(f"symlink rejected in source snapshot: {name}") from error
        if flags & getattr(os, "O_DIRECTORY", 0):
            raise
        raise SnapshotError(f"unsafe entry rejected in source snapshot: {name}") from error
    finally:
        parent.close()


def _inventory_fd(directory_fd: int, prefix: str = "",
                  identities: set[tuple[int, int]] | None = None,
                  budget: dict[str, int] | None = None) -> dict[str, str]:
    if budget is None:
        budget = {"entries": 0, "bytes": 0}
    result: dict[str, str] = {}
    directory_flags = (os.O_RDONLY | getattr(os, "O_NONBLOCK", 0)
                       | getattr(os, "O_DIRECTORY", 0) | os.O_NOFOLLOW)
    file_flags = os.O_RDONLY | getattr(os, "O_NONBLOCK", 0) | os.O_NOFOLLOW

    def relative_path(name: str | None = None) -> str:
        parts = [prefix] if prefix else []
        parts.extend(frame["name"] for frame in stack[1:])
        if name is not None:
            parts.append(name)
        return "/".join(parts)

    def begin_scan(fd: int, name: str | None) -> dict[str, Any]:
        info = os.fstat(fd)
        frame: dict[str, Any] = {
            "name": name,
            "identity": (info.st_dev, info.st_ino),
            "before": (info.st_dev, info.st_ino, info.st_mtime_ns, info.st_ctime_ns),
            "names": [],
            "index": 0,
        }
        with os.scandir(fd) as entries:
            for entry in entries:
                budget["entries"] += 1
                if budget["entries"] > MAX_SNAPSHOT_ENTRIES:
                    raise SnapshotError(
                        f"source snapshot exceeds {MAX_SNAPSHOT_ENTRIES} entries"
                    )
                frame["names"].append(entry.name)
        return frame

    # Keep the pinned root plus one active directory open. Descend by replacing
    # the active descriptor; ascend through `..` and verify parent/child identity.
    stack: list[dict[str, Any]] = []

    active_fd = os.dup(directory_fd)
    try:
        stack.append(begin_scan(active_fd, None))
        while stack:
            frame = stack[-1]
            if frame["index"] == len(frame["names"]):
                info = os.fstat(active_fd)
                after = (info.st_dev, info.st_ino, info.st_mtime_ns, info.st_ctime_ns)
                if after != frame["before"]:
                    raise SnapshotError(
                        f"directory changed while inventorying source snapshot: "
                        f"{relative_path() or '.'}"
                    )
                if len(stack) == 1:
                    break

                child_frame = frame
                parent_frame = stack[-2]
                parent_fd = os.open("..", directory_flags, dir_fd=active_fd)
                try:
                    parent_info = os.fstat(parent_fd)
                    if (not stat.S_ISDIR(parent_info.st_mode)
                            or (parent_info.st_dev, parent_info.st_ino) != parent_frame["identity"]):
                        raise SnapshotError(
                            f"directory parent changed while inventorying source snapshot: "
                            f"{relative_path() or '.'}"
                        )
                    named_child_fd = _open_inventory_child(
                        parent_fd, child_frame["name"], directory_flags
                    )
                    try:
                        child_info = os.fstat(named_child_fd)
                        if (not stat.S_ISDIR(child_info.st_mode)
                                or (child_info.st_dev, child_info.st_ino) != child_frame["identity"]):
                            raise SnapshotError(
                                f"directory identity changed while inventorying source snapshot: "
                                f"{relative_path() or '.'}"
                            )
                    finally:
                        os.close(named_child_fd)
                except BaseException:
                    os.close(parent_fd)
                    raise

                os.close(active_fd)
                active_fd = parent_fd
                stack.pop()
                continue

            name = frame["names"][frame["index"]]
            frame["index"] += 1
            try:
                child_fd = _open_inventory_child(active_fd, name, directory_flags)
            except (OSError, CandidateFSError):
                file_fd = _open_inventory_child(active_fd, name, file_flags)
                try:
                    if not stat.S_ISREG(os.fstat(file_fd).st_mode):
                        raise SnapshotError(
                            f"non-regular file rejected in source snapshot: {relative_path(name)}"
                        )
                    if identities is not None:
                        info = os.fstat(file_fd)
                        identity = (info.st_dev, info.st_ino)
                        if identity in identities:
                            raise SnapshotError(
                                "regular file is shared between baseline and candidate "
                                f"snapshots: {relative_path(name)}"
                            )
                        identities.add(identity)
                    info = os.fstat(file_fd)
                    if info.st_nlink != 1:
                        raise SnapshotError(
                            "regular file has multiple hard links in source snapshot: "
                            f"{relative_path(name)}"
                        )
                    before_hash = (info.st_dev, info.st_ino, info.st_size,
                                   info.st_mtime_ns, info.st_ctime_ns)
                    if budget["bytes"] + info.st_size > MAX_SNAPSHOT_BYTES:
                        raise SnapshotError(
                            f"source snapshot exceeds {MAX_SNAPSHOT_BYTES} regular-file bytes"
                        )
                    digest = hashlib.sha256()
                    with os.fdopen(file_fd, "rb") as source:
                        file_fd = -1
                        for block in iter(lambda: source.read(1024 * 1024), b""):
                            if budget["bytes"] + len(block) > MAX_SNAPSHOT_BYTES:
                                raise SnapshotError(
                                    f"source snapshot exceeds {MAX_SNAPSHOT_BYTES} regular-file bytes"
                                )
                            budget["bytes"] += len(block)
                            digest.update(block)
                        after_info = os.fstat(source.fileno())
                        after_hash = (after_info.st_dev, after_info.st_ino, after_info.st_size,
                                      after_info.st_mtime_ns, after_info.st_ctime_ns)
                        if after_hash != before_hash:
                            raise SnapshotError(
                                "regular file changed while hashing source snapshot: "
                                f"{relative_path(name)}"
                            )
                        if after_info.st_nlink != 1:
                            raise SnapshotError(
                                "regular file multiple hard links in source snapshot: "
                                f"{relative_path(name)}"
                            )
                    result[relative_path(name)] = digest.hexdigest()
                finally:
                    if file_fd >= 0:
                        os.close(file_fd)
            else:
                try:
                    child_frame = begin_scan(child_fd, name)
                except BaseException:
                    os.close(child_fd)
                    raise
                os.close(active_fd)
                active_fd = child_fd
                stack.append(child_frame)
        return result
    finally:
        os.close(active_fd)


def _inventory_snapshot(root: Path, identities: set[tuple[int, int]] | None = None,
                        *, pinned_root: PinnedDir | None = None) -> dict[str, str]:
    """Inventory a snapshot and optionally reject regular files sharing baseline inodes."""
    owned = pinned_root is None
    pinned_root = pinned_root or _open_absolute_root(root)
    try:
        return _inventory_fd(pinned_root.fd, identities=identities,
                             budget={"entries": 0, "bytes": 0})
    finally:
        if owned:
            pinned_root.close()


def _tree_hash(files: dict[str, str]) -> str:
    return hashlib.sha256(json.dumps(files, sort_keys=True).encode()).hexdigest()


def _load_manifest(path: Path, root_path: Path, root: Path,
                   pinned_root: PinnedDir | None = None) -> Any:
    """Open the manifest beneath a pinned root without following path symlinks."""
    relative = path.relative_to(root_path)
    try:
        relative_parts(relative.as_posix())
    except CandidateFSError as error:
        raise SnapshotError("manifest path must be a safe path beneath root") from error
    owned = pinned_root is None
    pinned_root = pinned_root or _open_absolute_root(root)
    try:
        try:
            descriptor = pinned_root.open_file(
                relative.as_posix(), os.O_RDONLY | getattr(os, "O_NONBLOCK", 0)
            )
        except CandidateFSError as error:
            cause = error
            while cause.__cause__ is not None:
                cause = cause.__cause__
            if isinstance(cause, OSError):
                if cause.errno == errno.ELOOP:
                    raise OSError(errno.ELOOP, f"symlink rejected in manifest path: {relative}") from error
                raise OSError(cause.errno, f"manifest cannot be opened safely: {relative}") from error
            raise OSError(errno.EIO, f"manifest cannot be opened safely: {relative}") from error
        with os.fdopen(descriptor, "rb") as source:
            before = os.fstat(source.fileno())
            if before.st_nlink != 1:
                raise SnapshotError("manifest has multiple hard links")
            version = (before.st_dev, before.st_ino, before.st_size,
                       before.st_mtime_ns, before.st_ctime_ns)
            raw = source.read(MAX_MANIFEST_BYTES + 1)
            after = os.fstat(source.fileno())
            if after.st_nlink != 1:
                raise SnapshotError("manifest has multiple hard links")
            if version != (after.st_dev, after.st_ino, after.st_size,
                           after.st_mtime_ns, after.st_ctime_ns):
                raise SnapshotError("manifest changed while reading")
    finally:
        if owned:
            pinned_root.close()
    if len(raw) > MAX_MANIFEST_BYTES:
        raise SnapshotError(f"manifest exceeds {MAX_MANIFEST_BYTES} bytes")
    return json.loads(raw.decode("utf-8"), object_pairs_hook=_unique_object,
                      parse_constant=_reject_constant)


def _resolve_source_path(source: Path, root: Path) -> Path:
    if ".." in source.parts:
        raise SnapshotError("source path is unsafe")
    absolute = Path(os.path.abspath(source if source.is_absolute() else root / source))
    try:
        relative = absolute.relative_to(root)
    except ValueError:
        # macOS exposes /var as an alias for /private/var. Map only a real
        # directory boundary; a symlink alias to the candidate root is rejected.
        boundary = absolute.parent
        while True:
            try:
                if not boundary.is_symlink() and os.path.samefile(boundary, root):
                    relative = absolute.relative_to(boundary)
                    break
            except OSError:
                pass
            if boundary == boundary.parent:
                raise SnapshotError("source path is outside candidate root")
            boundary = boundary.parent
    try:
        relative_parts(relative.as_posix())
    except CandidateFSError as error:
        raise SnapshotError("source path is unsafe") from error
    if not relative.parts:
        raise SnapshotError("source path must name a directory beneath candidate root")
    current = root
    for part in relative.parts:
        current /= part
        if current.is_symlink():
            raise SnapshotError("symlink rejected in source path and source root")
    return relative


def _validate(manifest: Any, root: Path, pinned_root: PinnedDir,
              *, require_candidate_relative_sources: bool = False) -> dict[str, Any]:
    errors: list[str] = []
    report: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "status": "invalid",
        "scope": "paired source snapshot integrity and source parity only; no provenance, approval, build, or acceptance",
        "candidate_id": None,
        "snapshot_files": {},
        "source_delta_files": [],
        "errors": errors,
    }
    if not isinstance(manifest, dict):
        errors.append("manifest must be a JSON object")
        return report
    if type(manifest.get("schema_version")) is not int or manifest["schema_version"] != SCHEMA_VERSION:
        errors.append(f"schema_version must be integer {SCHEMA_VERSION}")
    candidate_id = manifest.get("candidate_id")
    if (not isinstance(candidate_id, str) or CANDIDATE_ID_RE.fullmatch(candidate_id) is None
            or ".." in candidate_id):
        errors.append("candidate_id must be a safe 1-64 character identifier")
    else:
        report["candidate_id"] = candidate_id

    snapshots = manifest.get("snapshots")
    if not isinstance(snapshots, dict) or not {"baseline", "candidate"}.issubset(snapshots):
        errors.append("snapshots must contain baseline and candidate records")
        return report

    inventories: dict[str, dict[str, str]] = {}
    path_tree_hashes: dict[str, str] = {}
    baseline_file_identities: set[tuple[int, int]] = set()
    source_roots: set[Path] = set()
    for variant in ("baseline", "candidate"):
        record = snapshots[variant]
        if not isinstance(record, dict) or not isinstance(record.get("source"), str):
            errors.append(f"snapshots.{variant}.source must be a path")
            continue
        try:
            source = Path(record["source"])
            if require_candidate_relative_sources:
                expected_source = f"{variant}/source"
                if source.is_absolute() or source.as_posix() != expected_source:
                    raise SnapshotError(
                        f"snapshots.{variant}.source must be candidate-relative {expected_source}"
                    )
            if not source.is_absolute():
                source = root / source
            relative = _resolve_source_path(source, root)
            if any(relative == previous or relative in previous.parents or previous in relative.parents
                   for previous in source_roots):
                raise SnapshotError("baseline and candidate source paths must not overlap")
            source_roots.add(relative)
            try:
                source_dir = PinnedDir.open_child(pinned_root, relative.as_posix())
            except CandidateFSError as error:
                cause = error
                while cause.__cause__ is not None:
                    cause = cause.__cause__
                if isinstance(cause, OSError) and cause.errno == errno.ELOOP:
                    raise SnapshotError("symlink rejected in source path and source root") from error
                raise
            try:
                actual = _inventory_fd(source_dir.fd, identities=baseline_file_identities,
                                       budget={"entries": 0, "bytes": 0})
                try:
                    current_source = PinnedDir.open_child(pinned_root, relative.as_posix())
                except CandidateFSError as error:
                    raise SnapshotError(
                        "source directory identity changed while inventorying source snapshot"
                    ) from error
                try:
                    if current_source.identity() != source_dir.identity():
                        raise SnapshotError(
                            "source directory identity changed while inventorying source snapshot"
                        )
                finally:
                    current_source.close()
            finally:
                source_dir.close()
        except (OSError, RuntimeError, ValueError) as error:
            if isinstance(error, CandidateFSError):
                cause = error
                while cause.__cause__ is not None:
                    cause = cause.__cause__
                if isinstance(cause, OSError) and cause.errno == errno.ELOOP:
                    error = SnapshotError("symlink rejected in source path and source root")
            errors.append(f"snapshots.{variant}.source cannot be validated: {error}")
            continue

        expected = record.get("files")
        if not isinstance(expected, dict) or any(
            not isinstance(path, str) or not isinstance(digest, str) or SHA256_RE.fullmatch(digest) is None
            for path, digest in expected.items()
        ):
            errors.append(f"snapshots.{variant}.files must map paths to lowercase SHA-256 digests")
            continue
        if actual != expected:
            errors.append(f"snapshots.{variant}.files does not match its source directory")
        inventories[variant] = actual
        report["snapshot_files"][variant] = len(actual)

        for field in ("path_tree_sha256", "pristine_path_tree_sha256"):
            declared = record.get(field)
            if declared is not None and (not isinstance(declared, str) or SHA256_RE.fullmatch(declared) is None):
                errors.append(f"snapshots.{variant}.{field} must be a lowercase SHA-256 digest")
        if "path_tree_sha256" not in record:
            errors.append(f"snapshots.{variant}.path_tree_sha256 is required")
        path_files = {name for name in actual if name.startswith("path-sources/")}
        if any(name.count("/") < 2 for name in path_files):
            errors.append(
                f"snapshots.{variant}.path-sources files must be beneath a package directory"
            )
        path_packages = {name.split("/", 2)[1] for name in path_files if name.count("/") >= 2}
        if len(path_packages) == 1:
            prefix = f"path-sources/{next(iter(path_packages))}/"
            path_tree = {name.removeprefix(prefix): actual[name]
                         for name in sorted(path_files) if name.startswith(prefix)}
            actual_path_tree_hash = _tree_hash(path_tree)
            path_tree_hashes[variant] = actual_path_tree_hash
            if ("path_tree_sha256" in record
                    and record["path_tree_sha256"] != actual_path_tree_hash):
                errors.append(f"snapshots.{variant}.path_tree_sha256 does not match its path-source package")
        elif "path_tree_sha256" in record:
            errors.append(f"snapshots.{variant}.path_tree_sha256 requires exactly one path-source package")
        elif "pristine_path_tree_sha256" in record:
            errors.append(
                f"snapshots.{variant}.pristine_path_tree_sha256 requires exactly one path-source package"
            )

    if set(inventories) == {"baseline", "candidate"}:
        baseline, candidate = inventories["baseline"], inventories["candidate"]
        baseline_pristine_hash = path_tree_hashes.get("baseline")
        if baseline_pristine_hash is not None:
            for variant in ("baseline", "candidate"):
                record = snapshots.get(variant)
                if not isinstance(record, dict):
                    continue
                declared_pristine_hash = record.get("pristine_path_tree_sha256")
                if (isinstance(declared_pristine_hash, str)
                        and SHA256_RE.fullmatch(declared_pristine_hash) is not None
                        and declared_pristine_hash != baseline_pristine_hash):
                    errors.append(
                        f"snapshots.{variant}.pristine_path_tree_sha256 does not match the baseline path-source package"
                    )
        delta = sorted(path for path in set(baseline) | set(candidate)
                       if baseline.get(path) != candidate.get(path))
        report["source_delta_files"] = delta
        if not delta:
            errors.append("baseline and candidate snapshots have no source delta")
        if any(not path.startswith("path-sources/") for path in delta):
            errors.append("baseline and candidate may differ only beneath path-sources/")
    report["status"] = "valid" if not errors else "invalid"
    return report


def validate(manifest: Any, root: Path, pinned_root: PinnedDir | None = None,
             *, require_candidate_relative_sources: bool = False) -> dict[str, Any]:
    """Validate using one pinned candidate root for every source traversal."""
    if pinned_root is not None:
        return _validate(manifest, root, pinned_root,
                         require_candidate_relative_sources=require_candidate_relative_sources)
    with _open_absolute_root(root) as opened_root:
        return _validate(manifest, root, opened_root,
                         require_candidate_relative_sources=require_candidate_relative_sources)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", type=Path, help="candidate manifest containing paired snapshots")
    parser.add_argument("--root", type=Path, required=True, help="candidate directory containing both snapshots")
    args = parser.parse_args()
    try:
        root_path = Path(os.path.abspath(args.root))
        manifest_arg = args.manifest
        manifest_lexical = Path(os.path.abspath(manifest_arg if manifest_arg.is_absolute()
                                                 else root_path / manifest_arg))
        pinned_root = _inherited_candidate_root(args.root)
        inherited = pinned_root is not None
        if inherited:
            candidate_id = os.environ["RUSTUI_P3_CANDIDATE_ID"]
            expected = Path("target/headstart/candidates") / candidate_id
            root_path = Path.cwd() / expected
            if manifest_arg.is_absolute():
                manifest_relative = manifest_arg.relative_to(root_path)
            else:
                arg_path = Path(manifest_arg)
                try:
                    manifest_relative = arg_path.relative_to(expected)
                except ValueError:
                    manifest_relative = arg_path
            relative_parts(manifest_relative.as_posix())
            manifest_lexical = root_path / manifest_relative
        else:
            manifest_relative = manifest_lexical.relative_to(root_path)
            # Resolve parent aliases such as macOS /var, but keep the candidate
            # directory itself as the final no-follow component.
            root_path = root_path.parent.resolve(strict=True) / root_path.name
            manifest_lexical = root_path / manifest_relative
            pinned_root = _open_absolute_root(root_path)
        with pinned_root:
            root = root_path
            manifest = _load_manifest(manifest_lexical, root_path, root, pinned_root)
            report = validate(manifest, root, pinned_root,
                              require_candidate_relative_sources=inherited)
            if inherited and report["candidate_id"] != candidate_id:
                report["status"] = "invalid"
                report["errors"].append(
                    "manifest candidate_id does not match inherited candidate identity"
                )
    except (SnapshotError, OSError, UnicodeError, json.JSONDecodeError, RuntimeError, ValueError) as error:
        report = {
            "schema_version": SCHEMA_VERSION,
            "status": "invalid",
            "scope": "paired source snapshot integrity and source parity only; no provenance, approval, build, or acceptance",
            "candidate_id": None,
            "snapshot_files": {},
            "source_delta_files": [],
            "errors": [f"cannot read candidate manifest: {error}"],
        }
    print(json.dumps(report, sort_keys=True))
    return 0 if report["status"] == "valid" else 1


if __name__ == "__main__":
    raise SystemExit(main())
