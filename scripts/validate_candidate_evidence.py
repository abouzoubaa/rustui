#!/usr/bin/env python3
"""Validate one local P3 candidate evidence record without running commands."""

from __future__ import annotations

import argparse
import errno
import hashlib
import json
import math
import os
import re
import stat
import sys
from pathlib import Path, PurePosixPath, PureWindowsPath
from typing import Any

try:
    from p3_candidate_fs import CandidateFSError, PinnedDir, relative_parts
except ModuleNotFoundError:  # importlib-based tests load this file outside scripts/ on sys.path
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    from p3_candidate_fs import CandidateFSError, PinnedDir, relative_parts


SCHEMA_VERSION = 1
MAX_RECORD_BYTES = 1024 * 1024
MAX_ARTIFACT_BYTES = 4 * 1024 * 1024 * 1024
ARTIFACT_READ_BYTES = 1024 * 1024
GATE_NAMES = ("build", "correctness", "gui", "performance")
GATE_STATES = {"not_run", "passed", "failed", "inconclusive"}
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")


class EvidenceError(ValueError):
    """Invalid evidence JSON or schema."""


def _open_absolute_root(path: Path) -> PinnedDir:
    absolute = Path(os.path.abspath(path))
    current = PinnedDir.open_root(absolute.anchor)
    try:
        for part in absolute.parts[1:]:
            child = PinnedDir.open_child(current, part)
            current.close()
            current = child
        result = current
        current = None
        return result
    finally:
        if current is not None:
            current.close()


def _unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise EvidenceError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def _reject_constant(value: str) -> Any:
    raise EvidenceError(f"nonstandard JSON constant: {value}")


def _finite_float(value: str) -> float:
    parsed = float(value)
    if not math.isfinite(parsed):
        raise EvidenceError("JSON number is outside the finite float range")
    return parsed


def _digest(value: Any, field: str, errors: list[str]) -> bool:
    if not isinstance(value, str) or SHA256_RE.fullmatch(value) is None:
        errors.append(f"{field} must be a lowercase SHA-256 digest")
        return False
    return True


def _open_child(parent_fd: int, name: str, flags: int, field: str, relative: Path) -> int:
    try:
        return os.open(name, flags, dir_fd=parent_fd)
    except OSError as error:
        try:
            mode = os.stat(name, dir_fd=parent_fd, follow_symlinks=False).st_mode
        except (OSError, NotImplementedError):
            raise error
        if stat.S_ISLNK(mode):
            raise EvidenceError(f"{field} contains a symlink: {relative.as_posix()}") from error
        raise


def _open_child(parent_fd: int, name: str, flags: int, field: str, relative: Path) -> int:
    """Open one child through the common pinned-directory helper."""
    parent = PinnedDir.from_fd(parent_fd)
    try:
        if flags & getattr(os, "O_DIRECTORY", 0):
            child = PinnedDir.open_child(parent, name)
            try:
                return os.dup(child.fd)
            finally:
                child.close()
        return parent.open_file(name, flags)
    except (CandidateFSError, OSError) as error:
        cause = error
        while cause.__cause__ is not None:
            cause = cause.__cause__
        if isinstance(cause, OSError) and cause.errno == errno.ELOOP:
            raise EvidenceError(f"{field} contains a symlink: {relative.as_posix()}") from error
        raise EvidenceError(f"{field} cannot open {relative.as_posix()}") from error
    finally:
        parent.close()


def _open_regular_file(root: PinnedDir | Path, relative: Path, field: str):
    try:
        relative_parts(relative.as_posix())
    except CandidateFSError as error:
        raise EvidenceError(f"{field} must be a safe relative path") from error
    owned = not isinstance(root, PinnedDir)
    pinned_root = _open_absolute_root(root) if owned else root
    current = PinnedDir.from_fd(pinned_root.fd)
    try:
        directory_flags = (os.O_RDONLY | getattr(os, "O_NONBLOCK", 0)
                           | getattr(os, "O_DIRECTORY", 0) | getattr(os, "O_NOFOLLOW", 0))
        for part in relative.parts[:-1]:
            child_fd = _open_child(current.fd, part, directory_flags, field, relative)
            current.close()
            current = PinnedDir(child_fd)
        descriptor = _open_child(
            current.fd, relative.parts[-1],
            os.O_RDONLY | getattr(os, "O_NONBLOCK", 0) | getattr(os, "O_NOFOLLOW", 0),
            field, relative,
        )
        return os.fdopen(descriptor, "rb")
    except (CandidateFSError, OSError) as error:
        cause = error.__cause__
        if isinstance(cause, OSError) and cause.errno == errno.ELOOP:
            raise EvidenceError(f"{field} contains a symlink: {relative.as_posix()}") from error
        raise EvidenceError(f"{field} cannot be read beneath pinned root: {relative.as_posix()}") from error
    finally:
        current.close()
        if owned:
            pinned_root.close()


def _relative_path(value: Any, field: str, errors: list[str]) -> Path | None:
    if not isinstance(value, str) or not value:
        errors.append(f"{field} must be a nonempty relative path")
        return None
    if "\x00" in value or "\\" in value:
        errors.append(f"{field} must be a NUL-free slash-separated relative path")
        return None
    posix = PurePosixPath(value)
    windows = PureWindowsPath(value)
    if posix.is_absolute() or windows.is_absolute() or windows.drive:
        errors.append(f"{field} must be relative")
        return None
    if any(part in ("", ".", "..") for part in value.split("/")):
        errors.append(f"{field} contains an empty, dot, or traversal path segment")
        return None
    return Path(*value.split("/"))


def _artifact(root: PinnedDir | Path, value: Any, field: str, errors: list[str],
              seen_files: set[tuple[int, int]], total_artifact_bytes: list[int]) -> Path | None:
    if not isinstance(value, dict) or set(value) != {"path", "sha256"}:
        errors.append(f"{field} must contain exactly path and sha256")
        return None
    relative = _relative_path(value.get("path"), f"{field}.path", errors)
    digest_ok = _digest(value.get("sha256"), f"{field}.sha256", errors)
    if relative is None:
        return None
    if digest_ok:
        try:
            digest = hashlib.sha256()
            with _open_regular_file(root, relative, f"{field}.path") as stream:
                file_stat = os.fstat(stream.fileno())
                file_identity = (file_stat.st_dev, file_stat.st_ino)
                file_version = (file_stat.st_size, file_stat.st_mtime_ns, file_stat.st_ctime_ns)
                if file_stat.st_nlink != 1:
                    errors.append(f"{field}.path must not have multiple hard links")
                    return None
                remaining = MAX_ARTIFACT_BYTES - total_artifact_bytes[0]
                if file_stat.st_size > remaining:
                    errors.append(
                        f"{field} exceeds the aggregate artifact byte limit of {MAX_ARTIFACT_BYTES} bytes"
                    )
                    return None
                while True:
                    remaining = MAX_ARTIFACT_BYTES - total_artifact_bytes[0]
                    chunk = stream.read(min(ARTIFACT_READ_BYTES, remaining + 1))
                    if not chunk:
                        break
                    if len(chunk) > remaining:
                        total_artifact_bytes[0] += len(chunk)
                        errors.append(
                            f"{field} exceeds the aggregate artifact byte limit of {MAX_ARTIFACT_BYTES} bytes"
                        )
                        return None
                    digest.update(chunk)
                    total_artifact_bytes[0] += len(chunk)
                after_read = os.fstat(stream.fileno())
                if after_read.st_nlink > file_stat.st_nlink:
                    errors.append(f"{field}.path gained a hard link while its digest was being checked")
                    return None
                if (after_read.st_nlink != file_stat.st_nlink
                        or file_identity != (after_read.st_dev, after_read.st_ino)
                        or file_version != (after_read.st_size, after_read.st_mtime_ns,
                                            after_read.st_ctime_ns)):
                    errors.append(f"{field}.path changed while its digest was being checked")
                    return None
            # A stable descriptor can still refer to an inode that was replaced at
            # its path while hashing. Reopen through the same no-symlink walk and
            # require the current path to identify the bytes we just checked.
            with _open_regular_file(root, relative, f"{field}.path") as current_stream:
                current_stat = os.fstat(current_stream.fileno())
            if current_stat.st_nlink > 1 and file_identity == (current_stat.st_dev, current_stat.st_ino):
                errors.append(f"{field}.path gained a hard link while its digest was being checked")
                return None
            if (current_stat.st_nlink != 1
                    or file_identity != (current_stat.st_dev, current_stat.st_ino)
                    or file_version != (current_stat.st_size, current_stat.st_mtime_ns,
                                        current_stat.st_ctime_ns)):
                errors.append(f"{field}.path changed while its digest was being checked")
                return None
            actual = digest.hexdigest()
        except (EvidenceError, OSError) as error:
            errors.append(f"{field}.path cannot be read: {error}")
            return None
        else:
            if actual != value["sha256"]:
                errors.append(f"{field}.sha256 does not match the file SHA-256")
                return None
            identity = file_identity
            if identity in seen_files:
                errors.append(f"{field}.path must reference a distinct regular file")
                return None
            seen_files.add(identity)
    else:
        return None
    return relative


def validate(record: Any, root: Path | PinnedDir, pinned_root: PinnedDir | None = None) -> dict[str, Any]:
    """Validate record structure and local artifact integrity; never writes."""
    errors: list[str] = []
    report: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "status": "invalid",
        "scope": "candidate evidence record and local artifact integrity only; no acceptance or approval",
        "manifest_sha256": None,
        "gate_states": {name: "not_run" for name in GATE_NAMES},
        "errors": errors,
    }
    if pinned_root is None and not isinstance(root, PinnedDir):
        try:
            with _open_absolute_root(root.resolve(strict=True)) as opened_root:
                return validate(record, opened_root, opened_root)
        except (OSError, RuntimeError, CandidateFSError):
            errors.append("root must resolve to an existing directory")
            report["errors"] = errors
            return report
    root = pinned_root or root
    if not isinstance(record, dict):
        errors.append("evidence record must be a JSON object")
        report["errors"] = errors
        return report

    required = {"manifest_sha256", "command", "exit_status", "log", "evidence", "provenance", "gates"}
    if set(record) != required:
        errors.append("record must contain exactly manifest_sha256, command, exit_status, log, evidence, provenance, and gates")
    if _digest(record.get("manifest_sha256"), "manifest_sha256", errors):
        report["manifest_sha256"] = record["manifest_sha256"]

    command = record.get("command")
    if not isinstance(command, list) or not command or any(not isinstance(part, str) or not part or "\x00" in part for part in command):
        errors.append("command must be a nonempty array of nonempty strings")
    exit_status = record.get("exit_status")
    if type(exit_status) is not int or not 0 <= exit_status <= 255:
        errors.append("exit_status must be an integer from 0 to 255")

    seen_artifact_files: set[tuple[int, int]] = set()
    total_artifact_bytes = [0]
    log_path = _artifact(root, record.get("log"), "log", errors, seen_artifact_files,
                         total_artifact_bytes)
    if not isinstance(record.get("evidence"), list):
        errors.append("evidence must be an array of path and sha256 records")
        evidence_paths: list[Path] = []
    else:
        evidence_paths = []
        for index, item in enumerate(record["evidence"]):
            path = _artifact(root, item, f"evidence[{index}]", errors, seen_artifact_files,
                             total_artifact_bytes)
            if path is not None:
                evidence_paths.append(path)

    if not isinstance(record.get("provenance"), dict):
        errors.append("provenance must be an object; unknown provenance may be recorded as unknown")

    gates = record.get("gates")
    if not isinstance(gates, dict) or set(gates) != set(GATE_NAMES):
        errors.append("gates must contain exactly build, correctness, gui, and performance")
    else:
        for name, state in gates.items():
            if not isinstance(state, str) or state not in GATE_STATES:
                errors.append(f"gates.{name} must be one of {', '.join(sorted(GATE_STATES))}")
            elif state == "passed":
                if type(exit_status) is not int or exit_status != 0:
                    errors.append(f"gates.{name} cannot pass unless exit_status is 0")
                if log_path is None:
                    errors.append(f"gates.{name} cannot pass without a valid log artifact")
                if not evidence_paths:
                    errors.append(f"gates.{name} cannot pass without at least one valid evidence artifact")
        report["gate_states"] = gates

    report["status"] = "valid" if not errors else "invalid"
    return report


def _load(root: Path | PinnedDir, relative: Path) -> Any:
    try:
        with _open_regular_file(root, relative, "evidence record") as stream:
            before = os.fstat(stream.fileno())
            if before.st_nlink != 1:
                raise EvidenceError("evidence record must not have multiple hard links")
            identity = (before.st_dev, before.st_ino)
            version = (before.st_size, before.st_mtime_ns, before.st_ctime_ns)
            raw = stream.read(MAX_RECORD_BYTES + 1)
            if len(raw) > MAX_RECORD_BYTES:
                raise EvidenceError(f"evidence record exceeds {MAX_RECORD_BYTES} bytes")
            after = os.fstat(stream.fileno())
            if after.st_nlink != 1:
                raise EvidenceError("evidence record gained a hard link while being read")
            if (identity != (after.st_dev, after.st_ino)
                    or version != (after.st_size, after.st_mtime_ns, after.st_ctime_ns)):
                raise EvidenceError("evidence record changed while being read")
        with _open_regular_file(root, relative, "evidence record") as current_stream:
            current = os.fstat(current_stream.fileno())
        if current.st_nlink != 1:
            raise EvidenceError("evidence record gained a hard link while being read")
        if (identity != (current.st_dev, current.st_ino)
                or version != (current.st_size, current.st_mtime_ns, current.st_ctime_ns)):
            raise EvidenceError("evidence record changed while being read")
        return json.loads(raw.decode("utf-8"), object_pairs_hook=_unique_object,
                          parse_constant=_reject_constant, parse_float=_finite_float)
    except (OSError, UnicodeError, json.JSONDecodeError, EvidenceError) as error:
        raise EvidenceError(str(error)) from error


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("record", type=Path, help="candidate evidence JSON record")
    parser.add_argument("--root", type=Path, required=True, help="root for referenced log and evidence files")
    args = parser.parse_args()
    try:
        root_lexical = Path(os.path.abspath(args.root))
        record_arg = args.record
        record_lexical = Path(os.path.abspath(
            record_arg if record_arg.is_absolute() else root_lexical / record_arg
        ))
        record_relative = record_lexical.relative_to(root_lexical)
        if not record_relative.parts:
            raise EvidenceError("record path must identify a regular file beneath root")
        inherited_repo = os.environ.get("RUSTUI_P3_REPO_ROOT_FD")
        inherited_parent = os.environ.get("RUSTUI_P3_CANDIDATE_PARENT_FD")
        inherited_candidate = os.environ.get("RUSTUI_P3_CANDIDATE_ROOT_FD")
        candidate_id = os.environ.get("RUSTUI_P3_CANDIDATE_ID")
        if any(value is not None for value in
               (inherited_repo, inherited_parent, inherited_candidate, candidate_id)):
            if not all((inherited_repo, inherited_parent, inherited_candidate, candidate_id)):
                raise EvidenceError("incomplete inherited candidate root identity")
            if (re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]{0,63}", candidate_id) is None
                    or ".." in candidate_id):
                raise EvidenceError("inherited candidate ID must be a safe candidate identifier")
            expected = Path("target/headstart/candidates") / candidate_id
            try:
                supplied_root = root_lexical.relative_to(Path.cwd())
            except ValueError as error:
                raise EvidenceError("inherited root must be repo-relative") from error
            if supplied_root != expected:
                raise EvidenceError("root argument does not match candidate identity")
            repo = PinnedDir.from_fd(int(inherited_repo))
            parent = PinnedDir.from_fd(int(inherited_parent))
            try:
                expected_parent = PinnedDir.open_child(
                    repo, "target/headstart/candidates"
                )
                try:
                    if expected_parent.identity() != parent.identity():
                        raise EvidenceError(
                            "inherited candidate parent does not match repository root"
                        )
                finally:
                    expected_parent.close()
                identity_root = PinnedDir.open_child(parent, candidate_id)
            finally:
                parent.close()
                repo.close()
            pinned_root = PinnedDir.from_fd(int(inherited_candidate))
            if pinned_root.identity() != identity_root.identity():
                identity_root.close()
                pinned_root.close()
                raise EvidenceError("inherited candidate directory does not match candidate parent and ID")
            identity_root.close()
        else:
            pinned_root = _open_absolute_root(root_lexical.resolve(strict=True))
        with pinned_root:
            record = _load(pinned_root, record_relative)
            report = validate(record, pinned_root, pinned_root)
    except (EvidenceError, OSError, RuntimeError, ValueError) as error:
        report = {
            "schema_version": SCHEMA_VERSION,
            "status": "invalid",
            "scope": "candidate evidence record and local artifact integrity only; no acceptance or approval",
            "manifest_sha256": None,
            "gate_states": {name: "not_run" for name in GATE_NAMES},
            "errors": [f"cannot read evidence record: {error}"],
        }
    print(json.dumps(report, sort_keys=True))
    return 0 if report["status"] == "valid" else 1


if __name__ == "__main__":
    raise SystemExit(main())
