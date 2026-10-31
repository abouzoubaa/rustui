#!/usr/bin/env python3
"""Validate a local, immutable upstream patch-plan manifest without applying it."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import sys
from pathlib import Path, PurePosixPath, PureWindowsPath
from typing import Any, BinaryIO

from p3_candidate_fs import CandidateFSError, PinnedDir, relative_parts


SCHEMA_VERSION = 1
GATE_NAMES = ("build", "correctness", "gui", "performance")
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
REVISION_RE = re.compile(r"^[0-9a-f]{40}$")
IDENTIFIER_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$")


class ManifestError(ValueError):
    """Invalid manifest JSON or schema."""


def _object_without_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ManifestError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def _reject_nonstandard_json_constant(value: str) -> Any:
    raise ManifestError(f"nonstandard JSON constant: {value}")


def _sha256_is_valid(value: Any) -> bool:
    return isinstance(value, str) and SHA256_RE.fullmatch(value) is not None


def _relative_path(value: Any, field: str, errors: list[str]) -> Path | None:
    if not isinstance(value, str) or not value:
        errors.append(f"{field} must be a nonempty relative path")
        return None
    if "\x00" in value:
        errors.append(f"{field} contains a NUL byte")
        return None
    if "\\" in value:
        errors.append(f"{field} must use slash-separated relative paths")
        return None

    posix_path = PurePosixPath(value)
    windows_path = PureWindowsPath(value)
    if posix_path.is_absolute() or windows_path.is_absolute() or windows_path.drive:
        errors.append(f"{field} must be relative")
        return None
    if any(part in ("", ".", "..") for part in value.split("/")):
        errors.append(f"{field} contains an empty, dot, or traversal path segment")
        return None
    return Path(*value.split("/"))


def _contained_file(
    root: PinnedDir | Path, relative: Path, field: str, errors: list[str]
) -> Path | None:
    """Confirm a relative regular file exists beneath the pinned root."""
    try:
        with _open_contained_file(root, relative):
            pass
    except (OSError, CandidateFSError) as error:
        cause: BaseException = error
        while cause.__cause__ is not None:
            cause = cause.__cause__
        if isinstance(cause, FileNotFoundError):
            errors.append(f"{field} does not exist under root: {relative.as_posix()}")
        elif isinstance(error, CandidateFSError) and "not regular file" in str(error):
            errors.append(f"{field} is not a regular file: {relative.as_posix()}")
        else:
            errors.append(f"{field} escapes root or cannot be resolved: {relative.as_posix()}")
        return None
    else:
        # Keep only the manifest-relative label. Actual reads always reopen it
        # through this same descriptor anchor.
        return relative


_HASH_READ_CHUNK_SIZE = 64 * 1024


def _open_contained_file(root: PinnedDir | Path, relative: Path) -> BinaryIO:
    """Open one regular file relative to a pinned root without following links."""
    owned = not isinstance(root, PinnedDir)
    pinned_root = PinnedDir.open_root(root) if owned else root
    try:
        descriptor = pinned_root.open_file(
            relative.as_posix(), os.O_RDONLY | getattr(os, "O_NONBLOCK", 0)
        )
        try:
            return os.fdopen(descriptor, "rb")
        except BaseException:
            os.close(descriptor)
            raise
    finally:
        if owned:
            pinned_root.close()


def _read_contained_file(root: PinnedDir | Path, relative: Path) -> bytes:
    """Read one small metadata file without following path component symlinks."""
    with _open_contained_file(root, relative) as stream:
        return stream.read()


def _hash_contained_file(root: PinnedDir | Path, relative: Path) -> str:
    """Hash a contained file in fixed-size chunks."""
    return _hash_contained_file_with_identity(root, relative)[0]


def _hash_contained_file_with_identity(
    root: PinnedDir | Path, relative: Path
) -> tuple[str, tuple[int, int]]:
    """Hash a contained file and reject changes during the descriptor-relative read."""
    digest = hashlib.sha256()
    with _open_contained_file(root, relative) as stream:
        before = os.fstat(stream.fileno())
        for chunk in iter(lambda: stream.read(_HASH_READ_CHUNK_SIZE), b""):
            digest.update(chunk)
        after = os.fstat(stream.fileno())
    identity = (before.st_dev, before.st_ino)
    version = (before.st_size, before.st_mtime_ns, before.st_ctime_ns)
    if (identity != (after.st_dev, after.st_ino)
            or version != (after.st_size, after.st_mtime_ns, after.st_ctime_ns)):
        raise OSError("file changed while hashing")
    # The descriptor may still refer to an inode that was replaced at its path
    # while hashing; require the current path to name the bytes we checked.
    with _open_contained_file(root, relative) as current:
        current_stat = os.fstat(current.fileno())
    if (identity != (current_stat.st_dev, current_stat.st_ino)
            or version != (current_stat.st_size, current_stat.st_mtime_ns,
                           current_stat.st_ctime_ns)):
        raise OSError("file changed while hashing")
    return digest.hexdigest(), identity


def _hash_matches(
    root: PinnedDir | Path, relative: Path, expected: Any, field: str, errors: list[str]
) -> tuple[int, int] | None:
    if not _sha256_is_valid(expected):
        errors.append(f"{field} must be a lowercase SHA-256 digest")
        return None
    try:
        actual, identity = _hash_contained_file_with_identity(root, relative)
    except (OSError, CandidateFSError) as error:
        errors.append(f"{field} cannot be read: {error}")
        return None
    if actual != expected:
        errors.append(f"{field} does not match the file SHA-256")
    return identity


def _nonempty_string(value: Any, field: str, errors: list[str]) -> None:
    if not isinstance(value, str) or not value.strip() or "\x00" in value:
        errors.append(f"{field} must be a nonempty string")


def _validate_pinned(manifest: Any, root: PinnedDir) -> dict[str, Any]:
    """Validate using only paths opened relative to ``root``."""
    errors: list[str] = []
    report: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "status": "invalid",
        "scope": "manifest integrity only; validity does not imply approval or compatibility",
        "candidate_id": None,
        "source_revision": None,
        "patch_count": 0,
        "gates": {name: "not_run" for name in GATE_NAMES},
    }
    if not isinstance(manifest, dict):
        errors.append("manifest must be a JSON object")
        report["errors"] = errors
        return report

    required_fields = {"schema_version", "candidate", "source", "patches"}
    allowed_fields = required_fields | {"gates"}
    if not required_fields.issubset(manifest) or not set(manifest).issubset(allowed_fields):
        errors.append("manifest requires schema_version, candidate, source, and patches; only gates is optional")
    if type(manifest.get("schema_version")) is not int or manifest.get("schema_version") != SCHEMA_VERSION:
        errors.append(f"schema_version must be {SCHEMA_VERSION}")

    candidate = manifest.get("candidate")
    if not isinstance(candidate, dict) or set(candidate) != {
        "id", "snapshot_sha256", "parent_manifest_sha256"
    }:
        errors.append("candidate must contain exactly id, snapshot_sha256, and parent_manifest_sha256")
    else:
        candidate_id = candidate.get("id")
        if (
            not isinstance(candidate_id, str)
            or IDENTIFIER_RE.fullmatch(candidate_id) is None
            or ".." in candidate_id
        ):
            errors.append("candidate.id must be a safe 1-64 character identifier without ..")
        else:
            report["candidate_id"] = candidate_id
        for key in ("snapshot_sha256", "parent_manifest_sha256"):
            if not _sha256_is_valid(candidate.get(key)):
                errors.append(f"candidate.{key} must be a lowercase SHA-256 digest")

    source = manifest.get("source")
    if not isinstance(source, dict) or set(source) != {"repository", "revision", "packages"}:
        errors.append("source must contain exactly repository, revision, and packages")
    else:
        _nonempty_string(source.get("repository"), "source.repository", errors)
        revision = source.get("revision")
        if not isinstance(revision, str) or REVISION_RE.fullmatch(revision) is None:
            errors.append("source.revision must be a full lowercase 40-character commit SHA")
        else:
            report["source_revision"] = revision
        packages = source.get("packages")
        if not isinstance(packages, list) or not packages:
            errors.append("source.packages must be a nonempty array")
        else:
            package_names: set[str] = set()
            for index, package in enumerate(packages):
                field = f"source.packages[{index}]"
                if not isinstance(package, dict) or set(package) != {
                    "name", "version", "checksum_file", "checksum_sha256",
                    "license_expression", "license_files",
                }:
                    errors.append(f"{field} has missing or unexpected fields")
                    continue
                name = package.get("name")
                _nonempty_string(name, f"{field}.name", errors)
                if isinstance(name, str):
                    if name in package_names:
                        errors.append(f"duplicate package name: {name}")
                    package_names.add(name)
                _nonempty_string(package.get("version"), f"{field}.version", errors)
                _nonempty_string(package.get("license_expression"), f"{field}.license_expression", errors)
                checksum_rel = _relative_path(package.get("checksum_file"), f"{field}.checksum_file", errors)
                if checksum_rel is not None:
                    checksum_file = _contained_file(root, checksum_rel, f"{field}.checksum_file", errors)
                    if checksum_file is not None:
                        _hash_matches(root, checksum_rel, package.get("checksum_sha256"), f"{field}.checksum_sha256", errors)
                elif not _sha256_is_valid(package.get("checksum_sha256")):
                    errors.append(f"{field}.checksum_sha256 must be a lowercase SHA-256 digest")
                license_files = package.get("license_files")
                if not isinstance(license_files, list) or not license_files:
                    errors.append(f"{field}.license_files must be a nonempty array")
                else:
                    seen_license_paths: set[str] = set()
                    for license_index, value in enumerate(license_files):
                        license_field = f"{field}.license_files[{license_index}]"
                        license_rel = _relative_path(value, license_field, errors)
                        if isinstance(value, str) and value in seen_license_paths:
                            errors.append(f"{license_field} duplicates a license path")
                            continue
                        if isinstance(value, str):
                            seen_license_paths.add(value)
                        if license_rel is not None:
                            _contained_file(root, license_rel, license_field, errors)

    patches = manifest.get("patches")
    if not isinstance(patches, list):
        errors.append("patches must be an ordered array")
    else:
        report["patch_count"] = len(patches)
        seen_patch_paths: set[str] = set()
        seen_patch_files: set[Path] = set()
        seen_patch_identities: set[tuple[int, int]] = set()
        for index, patch in enumerate(patches):
            field = f"patches[{index}]"
            if not isinstance(patch, dict) or set(patch) != {"path", "sha256"}:
                errors.append(f"{field} must contain exactly path and sha256")
                continue
            patch_path_value = patch.get("path")
            duplicate_path = False
            if isinstance(patch_path_value, str):
                if patch_path_value in seen_patch_paths:
                    errors.append(f"duplicate patch path: {patch_path_value}")
                    duplicate_path = True
                seen_patch_paths.add(patch_path_value)
            patch_rel = _relative_path(patch_path_value, f"{field}.path", errors)
            if patch_rel is not None:
                contained_patch = _contained_file(root, patch_rel, f"{field}.path", errors)
                if contained_patch is not None:
                    if contained_patch in seen_patch_files and not duplicate_path:
                        errors.append(f"duplicate patch path: {patch_path_value}")
                    seen_patch_files.add(contained_patch)
                    identity = _hash_matches(
                        root, patch_rel, patch.get("sha256"), f"{field}.sha256", errors
                    )
                    if identity is not None and identity in seen_patch_identities:
                        errors.append(f"duplicate patch file identity: {patch_path_value}")
                    if identity is not None:
                        seen_patch_identities.add(identity)

    gates = manifest.get("gates", {})
    if not isinstance(gates, dict):
        errors.append("gates must be an object")
    else:
        for name, state in gates.items():
            if name not in GATE_NAMES:
                errors.append(f"unknown gate name: {name}")
            elif state != "not_run":
                errors.append(f"gate {name} must remain not_run in input manifest")

    report["status"] = "valid" if not errors else "invalid"
    report["errors"] = errors
    return report


def validate(manifest: Any, root: PinnedDir | Path) -> dict[str, Any]:
    """Validate a manifest against one pinned source root."""
    owned = not isinstance(root, PinnedDir)
    try:
        pinned_root = PinnedDir.open_root(root) if owned else root
    except (CandidateFSError, OSError) as error:
        return {
            "schema_version": SCHEMA_VERSION,
            "status": "invalid",
            "scope": "manifest integrity only; validity does not imply approval or compatibility",
            "candidate_id": None,
            "source_revision": None,
            "patch_count": 0,
            "gates": {name: "not_run" for name in GATE_NAMES},
            "errors": [f"root must be an accessible directory: {error}"],
        }
    try:
        return _validate_pinned(manifest, pinned_root)
    finally:
        if owned:
            pinned_root.close()


def _load_json(root: PinnedDir | Path, relative: Path) -> Any:
    try:
        return json.loads(
            _read_contained_file(root, relative).decode("utf-8"),
            object_pairs_hook=_object_without_duplicate_keys,
            parse_constant=_reject_nonstandard_json_constant,
        )
    except (OSError, UnicodeError, json.JSONDecodeError, ManifestError) as error:
        raise ManifestError(str(error)) from error


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", type=Path, help="candidate manifest JSON file")
    parser.add_argument("--root", type=Path, required=True, help="root for referenced patch and metadata files")
    args = parser.parse_args()

    pinned_root: PinnedDir | None = None
    try:
        source_root_fd = os.environ.get("RUSTUI_P3_SOURCE_ROOT_FD")
        if source_root_fd is not None:
            pinned_root = PinnedDir.from_fd(int(source_root_fd))
            root_parts = Path(*relative_parts(args.root.as_posix()))
            manifest_parts = Path(*relative_parts(args.manifest.as_posix()))
            try:
                manifest_relative = manifest_parts.relative_to(root_parts)
            except ValueError as error:
                raise ManifestError("manifest path must be beneath --root") from error
            relative_parts(manifest_relative.as_posix())

            # These repo-relative CLI paths identify the source candidate. The
            # inherited descriptor remains authoritative for every file read.
            repo_root_fd = os.environ.get("RUSTUI_P3_REPO_ROOT_FD")
            if repo_root_fd is not None:
                with PinnedDir.from_fd(int(repo_root_fd)) as repo_root:
                    try:
                        named_root = PinnedDir.open_child(repo_root, args.root.as_posix())
                    except CandidateFSError as error:
                        raise ManifestError("--root does not resolve beneath the pinned repository") from error
                    try:
                        if named_root.identity() != pinned_root.identity():
                            raise ManifestError("--root does not identify the inherited source root")
                    finally:
                        named_root.close()

            manifest = _load_json(pinned_root, manifest_relative)
            report = validate(manifest, pinned_root)
        else:
            # Standalone invocations preserve absolute/relative path CLI
            # behavior while opening the root once and anchoring all descendants.
            root_lexical = Path(os.path.abspath(args.root))
            manifest_lexical = Path(os.path.abspath(args.manifest))
            try:
                manifest_relative = manifest_lexical.relative_to(root_lexical)
            except ValueError as error:
                raise ManifestError("manifest file must resolve inside root") from error
            relative_parts(manifest_relative.as_posix())
            pinned_root = PinnedDir.open_root(args.root)
            manifest = _load_json(pinned_root, manifest_relative)
            report = validate(manifest, pinned_root)
    except (ManifestError, OSError, RuntimeError, ValueError) as error:
        report = {
            "schema_version": SCHEMA_VERSION,
            "status": "invalid",
            "scope": "manifest integrity only; validity does not imply approval or compatibility",
            "candidate_id": None,
            "source_revision": None,
            "patch_count": 0,
            "gates": {name: "not_run" for name in GATE_NAMES},
            "errors": [f"cannot read manifest: {error}"],
        }
    finally:
        if pinned_root is not None:
            pinned_root.close()
    print(json.dumps(report, sort_keys=True))
    return 0 if report["status"] == "valid" else 1


if __name__ == "__main__":
    raise SystemExit(main())
