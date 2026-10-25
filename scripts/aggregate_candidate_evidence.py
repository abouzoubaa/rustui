#!/usr/bin/env python3
"""Reconcile local P3 pipeline events and gate evidence without accepting a candidate."""

from __future__ import annotations

import argparse
import errno
import hashlib
import json
import math
import os
import re
import stat
import struct
import sys
import zlib
from pathlib import Path, PurePosixPath, PureWindowsPath
from typing import Any

from p3_candidate_fs import CandidateFSError, PinnedDir, relative_parts
from validate_candidate_evidence import validate as validate_p3f


SCHEMA_VERSION = 1
MAX_RECORD_BYTES = 1024 * 1024
MAX_COMMAND_LOG_BYTES = 64 * 1024 * 1024
MAX_GUI_ARTIFACT_BYTES = 64 * 1024 * 1024
# P3F bounds the command log and all supporting evidence to 4 GiB combined.
MAX_P3F_ARTIFACT_BYTES = 4 * 1024 * 1024 * 1024
# P3G bounds snapshot manifests at 32 MiB.
MAX_SNAPSHOT_MANIFEST_BYTES = 32 * 1024 * 1024
PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"
HASH_RE = re.compile(r"^[0-9a-f]{64}$")
CANDIDATE_ID_RE = re.compile(r"[A-Za-z0-9][A-Za-z0-9._-]{0,63}\Z")
GATES = ("build", "correctness", "gui", "performance")
PIPELINE_GATES = (*GATES, "promotion")
GATE_STATES = {"not_run", "passed", "failed", "inconclusive"}
COMMAND_VALIDATORS = {
    "P3E-patch-plan": "validate_patch_plan.py",
    "P3F-evidence-record": "validate_candidate_evidence.py",
    "P3G-snapshot": "validate_candidate_snapshot.py",
    "P3H-baseline": "report_candidate_sources.py",
    "P3H-candidate": "report_candidate_sources.py",
    "P3I-J-source-and-graph-comparison": "compare_candidate_sources.py",
    "P3K-exact-patch": "apply_candidate_patch.py",
}
SCOPE = "local evidence integrity and gate-state consistency only; no candidate acceptance or promotion"


class AggregateError(ValueError):
    """Untrusted or inconsistent candidate evidence."""


def _unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise AggregateError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def _reject_constant(value: str) -> Any:
    raise AggregateError(f"nonstandard JSON constant: {value}")


def _finite_float(value: str) -> float:
    parsed = float(value)
    if not math.isfinite(parsed):
        raise AggregateError("JSON number is outside the finite float range")
    return parsed


def _safe_relative(value: Any, label: str) -> Path:
    if not isinstance(value, str) or not value or "\x00" in value or "\\" in value:
        raise AggregateError(f"{label} must be a slash-separated relative path")
    posix, windows = PurePosixPath(value), PureWindowsPath(value)
    if posix.is_absolute() or windows.is_absolute() or windows.drive or any(
        part in ("", ".", "..") for part in value.split("/")
    ):
        raise AggregateError(f"{label} must be a safe relative path")
    return Path(*value.split("/"))


def _open_child(parent_fd: int, name: str, flags: int, label: str,
                relative: Path) -> int:
    try:
        return os.open(name, flags, dir_fd=parent_fd)
    except OSError as error:
        try:
            mode = os.stat(name, dir_fd=parent_fd, follow_symlinks=False).st_mode
        except (OSError, NotImplementedError):
            raise error
        if stat.S_ISLNK(mode):
            raise AggregateError(f"symlink rejected in {label}: {relative.as_posix()}") from error
        raise


def _read_file(
    root: Path | PinnedDir,
    relative: Path,
    label: str,
    *,
    with_identity: bool = False,
    max_bytes: int | None = None,
) -> bytes | tuple[bytes, tuple[int, int]]:
    if (not relative.parts or relative.is_absolute()
            or any(part in ("", ".", "..") for part in relative.parts)):
        raise AggregateError(f"{label} must be a safe relative path")
    if (not hasattr(os, "O_NOFOLLOW") or os.open not in os.supports_dir_fd
            or os.stat not in os.supports_dir_fd
            or os.stat not in os.supports_follow_symlinks):
        raise AggregateError(
            "platform lacks descriptor-relative O_NOFOLLOW support required for safe evidence reads"
        )

    directory_flags = (os.O_RDONLY | getattr(os, "O_NONBLOCK", 0)
                       | getattr(os, "O_DIRECTORY", 0) | os.O_NOFOLLOW)
    file_flags = os.O_RDONLY | getattr(os, "O_NONBLOCK", 0) | os.O_NOFOLLOW
    current_fd: int | None = None
    file_fd: int | None = None
    try:
        # Pin the root and walk each path component through directory descriptors.
        # O_NOFOLLOW prevents an entry swap from redirecting any individual open.
        if isinstance(root, PinnedDir):
            current_fd = os.dup(root.fd)
        else:
            current_fd = os.open(root.anchor, directory_flags)
            if not stat.S_ISDIR(os.fstat(current_fd).st_mode):
                raise AggregateError("evidence root must be a directory")
            for part in root.parts[1:]:
                child_fd = _open_child(current_fd, part, directory_flags, label, relative)
                os.close(current_fd)
                current_fd = child_fd
                if not stat.S_ISDIR(os.fstat(current_fd).st_mode):
                    raise AggregateError("evidence root path contains a non-directory component")
        for part in relative.parts[:-1]:
            child_fd = _open_child(current_fd, part, directory_flags, label, relative)
            os.close(current_fd)
            current_fd = child_fd
            if not stat.S_ISDIR(os.fstat(current_fd).st_mode):
                raise AggregateError(f"{label} path contains a non-directory component")
        file_fd = _open_child(current_fd, relative.parts[-1], file_flags, label, relative)
        if not stat.S_ISREG(os.fstat(file_fd).st_mode):
            raise AggregateError(f"{label} must be a regular file: {relative.as_posix()}")
        with os.fdopen(file_fd, "rb") as stream:
            file_fd = None
            before_read = os.fstat(stream.fileno())
            if before_read.st_nlink != 1:
                raise AggregateError(f"{label} must not have multiple hard links")
            identity = (before_read.st_dev, before_read.st_ino)
            version = (before_read.st_size, before_read.st_mtime_ns, before_read.st_ctime_ns)
            raw = stream.read() if max_bytes is None else stream.read(max_bytes + 1)
            if max_bytes is not None and len(raw) > max_bytes:
                raise AggregateError(f"{label} exceeds the {max_bytes}-byte record limit")
            after_read = os.fstat(stream.fileno())
            if after_read.st_nlink != 1:
                raise AggregateError(f"{label} gained a hard link while being read")
            if (identity != (after_read.st_dev, after_read.st_ino)
                    or version != (after_read.st_size, after_read.st_mtime_ns,
                                   after_read.st_ctime_ns)):
                raise AggregateError(f"{label} changed while it was being read")
        current_file_fd = _open_child(current_fd, relative.parts[-1], file_flags, label, relative)
        try:
            current = os.fstat(current_file_fd)
        finally:
            os.close(current_file_fd)
        if current.st_nlink != 1:
            raise AggregateError(f"{label} gained a hard link while being read")
        if (identity != (current.st_dev, current.st_ino)
                or version != (current.st_size, current.st_mtime_ns, current.st_ctime_ns)):
            raise AggregateError(f"{label} changed while it was being read")
        if with_identity:
            return raw, identity
        return raw
    except AggregateError:
        raise
    except (OSError, RuntimeError) as error:
        if error.errno == errno.ELOOP:
            raise AggregateError(f"symlink rejected in {label}: {relative.as_posix()}") from error
        raise AggregateError(f"{label} cannot be read under root: {relative.as_posix()}") from error
    finally:
        if file_fd is not None:
            os.close(file_fd)
        if current_fd is not None:
            os.close(current_fd)


def _load_json(raw: bytes, label: str) -> Any:
    try:
        return json.loads(raw.decode("utf-8"), object_pairs_hook=_unique_object,
                          parse_constant=_reject_constant, parse_float=_finite_float)
    except (UnicodeError, json.JSONDecodeError, AggregateError) as error:
        raise AggregateError(f"invalid {label}: {error}") from error


def _sha256(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def _same_json_value(left: Any, right: Any) -> bool:
    """Compare parsed JSON without Python's bool/int equality equivalence."""
    if type(left) is not type(right):
        return False
    if isinstance(left, dict):
        return (left.keys() == right.keys()
                and all(_same_json_value(left[key], right[key]) for key in left))
    if isinstance(left, list):
        return len(left) == len(right) and all(
            _same_json_value(a, b) for a, b in zip(left, right)
        )
    return left == right


def _snapshot_file_maps_match(source: Any, candidate: Any) -> bool:
    return (isinstance(candidate, dict) and isinstance(source, dict)
            and _same_json_value(source, candidate))


def _artifact(root: Path | PinnedDir, base: Path, value: Any, label: str,
              *, max_bytes: int | None = None) -> bytes:
    if not isinstance(value, dict) or set(value) != {"path", "sha256"}:
        raise AggregateError(f"{label} must contain exactly path and sha256")
    relative = _safe_relative(value.get("path"), f"{label}.path")
    expected = value.get("sha256")
    if not isinstance(expected, str) or HASH_RE.fullmatch(expected) is None:
        raise AggregateError(f"{label}.sha256 must be a lowercase SHA-256 digest")
    raw = _read_file(root, base / relative, label, max_bytes=max_bytes)
    if _sha256(raw) != expected:
        raise AggregateError(f"{label}.sha256 does not match the file SHA-256")
    return raw


def _summary_artifact(
    root: Path | PinnedDir,
    value: Any,
    label: str,
    *,
    with_identity: bool = False,
    max_bytes: int | None = None,
) -> tuple[Path, bytes] | tuple[Path, bytes, tuple[int, int]]:
    if not isinstance(value, dict) or set(value) != {"path", "sha256"}:
        raise AggregateError(f"{label} must contain exactly path and sha256")
    relative = _safe_relative(value.get("path"), f"{label}.path")
    expected = value.get("sha256")
    if not isinstance(expected, str) or HASH_RE.fullmatch(expected) is None:
        raise AggregateError(f"{label}.sha256 must be a lowercase SHA-256 digest")
    read_result = _read_file(root, relative, label, with_identity=with_identity,
                             max_bytes=max_bytes)
    if with_identity:
        raw, identity = read_result
    else:
        raw = read_result
    if _sha256(raw) != expected:
        raise AggregateError(f"{label}.sha256 does not match the file SHA-256")
    if with_identity:
        return relative, raw, identity
    return relative, raw


def _has_png_structure(raw: bytes) -> bool:
    """Check PNG chunk framing and required structure, not decoded pixels."""
    if not raw.startswith(PNG_SIGNATURE):
        return False
    offset = len(PNG_SIGNATURE)
    seen_ihdr = seen_idat = seen_idat_data = seen_iend = False
    seen_plte = False
    idat_ended = False
    while offset < len(raw):
        if len(raw) - offset < 12:
            return False
        length = int.from_bytes(raw[offset:offset + 4], "big")
        chunk_type = raw[offset + 4:offset + 8]
        chunk_end = offset + 12 + length
        if chunk_end > len(raw) or not all(
            65 <= byte <= 90 or 97 <= byte <= 122 for byte in chunk_type
        ) or 97 <= chunk_type[2] <= 122:
            return False
        if 65 <= chunk_type[0] <= 90 and chunk_type not in {
            b"IHDR", b"PLTE", b"IDAT", b"IEND",
        }:
            return False
        data_end = offset + 8 + length
        data = raw[offset + 8:data_end]
        expected_crc = int.from_bytes(raw[data_end:chunk_end], "big")
        if zlib.crc32(chunk_type + data) & 0xffffffff != expected_crc:
            return False

        if not seen_ihdr:
            if chunk_type != b"IHDR" or length != 13:
                return False
            width, height, depth, color, compression, filtering, interlace = struct.unpack(
                ">IIBBBBB", data
            )
            valid_depths = {
                0: {1, 2, 4, 8, 16}, 2: {8, 16}, 3: {1, 2, 4, 8},
                4: {8, 16}, 6: {8, 16},
            }
            if (width == 0 or height == 0 or depth not in valid_depths.get(color, set())
                    or compression != 0 or filtering != 0 or interlace not in (0, 1)):
                return False
            seen_ihdr = True
        elif chunk_type == b"IHDR":
            return False

        if chunk_type == b"PLTE":
            if (seen_plte or seen_idat or color in (0, 4)
                    or length < 3 or length > 768 or length % 3 != 0):
                return False
            seen_plte = True

        if chunk_type == b"IDAT":
            if idat_ended or seen_iend or (color == 3 and not seen_plte):
                return False
            seen_idat = True
            seen_idat_data = seen_idat_data or length > 0
        elif seen_idat and chunk_type != b"IEND":
            idat_ended = True

        if chunk_type == b"IEND":
            if length != 0 or not seen_idat or not seen_idat_data:
                return False
            seen_iend = True
            return chunk_end == len(raw)
        offset = chunk_end
    return False


def _check_summary_identity(summary: Any, *, milestone: str, candidate_id: str,
                            manifest_hash: str) -> None:
    if not isinstance(summary, dict) or type(summary.get("schema_version")) is not int \
            or summary["schema_version"] != 1:
        raise AggregateError(f"{milestone} follow-up summary schema is invalid")
    if (summary.get("milestone") != milestone
            or summary.get("candidate_id") != candidate_id
            or summary.get("manifest_sha256") != manifest_hash):
        raise AggregateError(f"{milestone} candidate or manifest identity conflicts with pipeline result")


def _check_followup_event(root: Path | PinnedDir, *, event_path: Any, event_hash: Any,
                          log_path: Any, log_hash: Any, manifest_hash: str,
                          candidate_id: str, expected_step: str,
                          required_identity_field: str,
                          summary_exit_status: Any, label: str) -> None:
    event_relative, event_raw = _summary_artifact(
        root, {"path": event_path, "sha256": event_hash}, f"{label} event",
        max_bytes=MAX_RECORD_BYTES,
    )
    event = _load_json(event_raw, f"{label} event")
    if (not isinstance(event, dict) or type(event.get("schema_version")) is not int
            or event["schema_version"] != 1):
        raise AggregateError(f"{label} event schema_version must be 1")
    if (event.get("step") != expected_step
            or event.get("status") != "passed"
            or type(event.get("exit_status")) is not int or event["exit_status"] != 0):
        raise AggregateError(f"{label} event has the wrong step or is not successful for this manifest")
    action = "check" if expected_step.endswith("check") else "build"
    command = event.get("command")
    if (not isinstance(command, list) or not command
            or not all(isinstance(part, str) for part in command)
            or PurePosixPath(command[0].replace("\\", "/")).name != "cargo-headstart"
            or command[1:] != [action, "--workspace", "--all-targets", "--locked", "--offline"]):
        raise AggregateError(f"{label} event command does not match its locked all-targets {action}")
    if (type(summary_exit_status) is not int
            or summary_exit_status != event["exit_status"]):
        raise AggregateError(f"{label} summary exit_status conflicts with its successful event")
    expected_identity = (candidate_id if required_identity_field == "candidate_id"
                         else manifest_hash)
    if event.get(required_identity_field) != expected_identity:
        raise AggregateError(
            f"{label} event {required_identity_field} identity is missing or conflicts with summary"
        )
    if event.get("candidate_id") is not None and event["candidate_id"] != candidate_id:
        raise AggregateError(f"{label} event candidate identity conflicts with summary")
    if event.get("manifest_sha256") is not None and event["manifest_sha256"] != manifest_hash:
        raise AggregateError(f"{label} event manifest identity conflicts with summary")
    event_log = event.get("log")
    if not isinstance(event_log, dict) or set(event_log) != {"path", "sha256"}:
        raise AggregateError(f"{label} event has no hash-bound log")
    event_log_relative = _safe_relative(event_log["path"], f"{label} event log path")
    summary_log_relative, _ = _summary_artifact(
        root, {"path": log_path, "sha256": log_hash}, f"{label} log",
    )
    if (event_log_relative != summary_log_relative
            and event_relative.parent / event_log_relative != summary_log_relative):
        raise AggregateError(f"{label} event log path conflicts with follow-up summary")
    if event_log.get("sha256") != log_hash:
        raise AggregateError(f"{label} event log digest conflicts with follow-up summary")


def _check_m276_success(root: Path | PinnedDir, summary: Any, label: str) -> None:
    _, event_raw = _summary_artifact(
        root, {"path": summary.get("success_event_path"),
               "sha256": summary.get("success_event_sha256")}, f"{label} event",
        max_bytes=MAX_RECORD_BYTES,
    )
    event = _load_json(event_raw, f"{label} event")
    if (not isinstance(event, dict) or event.get("milestone") != "M276"
            or event.get("status") != "passed"
            or type(event.get("exit_status")) is not int or event["exit_status"] != 0):
        raise AggregateError(f"{label} event is not a successful M276 event")
    for field in ("candidate_id", "manifest_sha256"):
        if event.get(field) != summary.get(field):
            raise AggregateError(f"{label} event {field} identity is missing or conflicts with summary")
    _, log_raw = _summary_artifact(
        root, {"path": summary.get("success_log_path"),
               "sha256": summary.get("success_log_sha256")}, f"{label} log",
    )
    event_hashes = event.get("sha256")
    if not isinstance(event_hashes, dict) or event_hashes.get("raw_log") != _sha256(log_raw):
        raise AggregateError(f"{label} event does not bind the referenced success log")


def _check_m276_source_identity(root: Path | PinnedDir, summary: Any, manifest: Any,
                                root_lexical: Path | None = None) -> None:
    identity = summary.get("source_identity")
    expected_fields = {
        "baseline_scene_path", "baseline_scene_sha256",
        "candidate_scene_path", "candidate_scene_sha256",
    }
    if not isinstance(identity, dict) or set(identity) != expected_fields:
        raise AggregateError("M276 summary must bind baseline and candidate scene source identities")

    snapshots = manifest.get("snapshots") if isinstance(manifest, dict) else None
    if not isinstance(snapshots, dict):
        raise AggregateError("candidate manifest has no paired snapshots for M276 source identity")

    for side in ("baseline", "candidate"):
        scene_path = _safe_relative(identity[f"{side}_scene_path"], f"M276 {side} scene path")
        scene_hash = identity[f"{side}_scene_sha256"]
        if not isinstance(scene_hash, str) or HASH_RE.fullmatch(scene_hash) is None:
            raise AggregateError(f"M276 {side} scene SHA-256 must be lowercase")

        snapshot = snapshots.get(side)
        if not isinstance(snapshot, dict):
            raise AggregateError(f"candidate manifest has no {side} snapshot")
        source_root = snapshot.get("source")
        if not isinstance(source_root, str):
            raise AggregateError(f"candidate manifest has no {side} source root")
        if isinstance(root, PinnedDir):
            if root_lexical is not None and Path(source_root).is_absolute():
                try:
                    expected_source = Path(os.path.abspath(source_root)).relative_to(root_lexical)
                except ValueError as error:
                    raise AggregateError(
                        f"candidate manifest {side} source root escapes candidate root"
                    ) from error
            else:
                try:
                    expected_source = Path(*relative_parts(source_root))
                except CandidateFSError as error:
                    raise AggregateError(
                        f"candidate manifest {side} source root escapes candidate root"
                    ) from error
        else:
            try:
                expected_source = Path(source_root).resolve(strict=True).relative_to(root)
            except (OSError, ValueError) as error:
                raise AggregateError(f"candidate manifest {side} source root escapes candidate root") from error
        if expected_source != Path(side) / "source" or not scene_path.is_relative_to(expected_source):
            raise AggregateError(f"M276 {side} scene path does not belong to its manifest source root")

        source_relative = scene_path.relative_to(expected_source).as_posix()
        files = snapshot.get("files")
        manifest_hash = files.get(source_relative) if isinstance(files, dict) else None
        if not isinstance(manifest_hash, str) or HASH_RE.fullmatch(manifest_hash) is None:
            raise AggregateError(f"candidate manifest does not bind the {side} scene source")
        if scene_hash != manifest_hash:
            raise AggregateError(f"M276 {side} scene SHA-256 conflicts with candidate manifest")
        scene_raw = _read_file(root, scene_path, f"M276 {side} scene source")
        if _sha256(scene_raw) != scene_hash:
            raise AggregateError(f"M276 {side} scene source SHA-256 does not match file")


def _check_m287_gui(root: Path | PinnedDir, summary: Any, *, candidate_id: str,
                    manifest_hash: str, source_manifest_hash: str,
                    source_manifest: Any,
                    build_summary: tuple[Path, bytes, Any]) -> dict[str, Any]:
    expected_fields = {
        "schema_version", "milestone", "candidate_id", "manifest_sha256",
        "source_hash_manifest_sha256", "build_summary_path", "build_summary_sha256",
        "gate_states", "snapshots",
    }
    if not isinstance(summary, dict) or set(summary) != expected_fields:
        raise AggregateError("M287 GUI summary has missing or unknown fields")
    if type(summary.get("schema_version")) is not int or summary["schema_version"] != 1:
        raise AggregateError("M287 GUI summary schema_version must be 1")
    if (summary.get("milestone") != "M287"
            or summary.get("candidate_id") != candidate_id
            or summary.get("manifest_sha256") != manifest_hash):
        raise AggregateError("M287 candidate or manifest identity conflicts with pipeline result")
    if summary.get("source_hash_manifest_sha256") != source_manifest_hash:
        raise AggregateError("M287 source hash manifest digest conflicts with pipeline result")

    build_path, build_raw, _ = build_summary
    linked_build_path = _safe_relative(summary.get("build_summary_path"),
                                       "M287 build_summary_path")
    if (linked_build_path != build_path
            or summary.get("build_summary_sha256") != _sha256(build_raw)):
        raise AggregateError("M287 build summary path or digest does not match supplied M274 summary")

    states = summary.get("gate_states")
    expected_state_fields = {"visual", "accessibility", "gui"}
    if not isinstance(states, dict) or set(states) != expected_state_fields:
        raise AggregateError("M287 gate_states must contain exactly visual, accessibility, and gui")
    if any(value not in GATE_STATES for value in states.values()):
        raise AggregateError("M287 gate_states contains an invalid state")

    snapshots = summary.get("snapshots")
    expected_snapshot_fields = {
        "source_tree_sha256", "screenshot_state", "screenshot",
        "accessibility_state", "accessibility_tree",
    }
    source_snapshots = (source_manifest.get("snapshots")
                        if isinstance(source_manifest, dict) else None)
    if not isinstance(snapshots, dict) or set(snapshots) != {"baseline", "candidate"}:
        raise AggregateError("M287 snapshots must contain exactly baseline and candidate")
    if not isinstance(source_snapshots, dict):
        raise AggregateError("source hash manifest has no paired snapshots for M287")

    retained_snapshots: dict[str, Any] = {}
    artifact_identities: dict[str, dict[str, tuple[int, int] | None]] = {}
    for side, snapshot in snapshots.items():
        if not isinstance(snapshot, dict) or set(snapshot) != expected_snapshot_fields:
            raise AggregateError(f"M287 {side} snapshot has missing or unknown fields")
        source_tree_hash = snapshot.get("source_tree_sha256")
        expected_snapshot = source_snapshots.get(side)
        expected_tree_hash = (expected_snapshot.get("tree_sha256")
                              if isinstance(expected_snapshot, dict) else None)
        if (not isinstance(source_tree_hash, str) or HASH_RE.fullmatch(source_tree_hash) is None
                or source_tree_hash != expected_tree_hash):
            raise AggregateError(f"M287 {side} source tree digest conflicts with source hash manifest")

        artifacts: dict[str, Any] = {}
        artifact_identities[side] = {}
        for state_key, artifact_key in (("screenshot_state", "screenshot"),
                                        ("accessibility_state", "accessibility_tree")):
            state = snapshot.get(state_key)
            if state not in GATE_STATES:
                raise AggregateError(f"M287 {side} {state_key} is invalid")
            artifact = snapshot.get(artifact_key)
            if artifact is None:
                if state == "passed":
                    raise AggregateError(f"M287 {side} {artifact_key} is required for a passed state")
                artifacts[artifact_key] = None
                artifact_identities[side][artifact_key] = None
                continue
            relative, raw, identity = _summary_artifact(
                root, artifact, f"M287 {side} {artifact_key}", with_identity=True,
                max_bytes=MAX_GUI_ARTIFACT_BYTES,
            )
            if artifact_key == "screenshot" and not _has_png_structure(raw):
                raise AggregateError(f"M287 {side} screenshot is not a PNG image")
            artifacts[artifact_key] = {"path": relative.as_posix(), "sha256": _sha256(raw)}
            artifact_identities[side][artifact_key] = identity
        retained_snapshots[side] = {
            "source_tree_sha256": source_tree_hash,
            "screenshot_state": snapshot["screenshot_state"],
            "screenshot": artifacts["screenshot"],
            "accessibility_state": snapshot["accessibility_state"],
            "accessibility_tree": artifacts["accessibility_tree"],
        }

    for artifact_key, label in (
        ("screenshot", "screenshot"),
        ("accessibility_tree", "accessibility tree"),
    ):
        baseline_artifact = retained_snapshots["baseline"][artifact_key]
        candidate_artifact = retained_snapshots["candidate"][artifact_key]
        if (
            baseline_artifact is not None
            and candidate_artifact is not None
            and baseline_artifact["path"] == candidate_artifact["path"]
        ):
            raise AggregateError(
                f"M287 baseline and candidate {label} paths must differ"
            )
        baseline_identity = artifact_identities["baseline"][artifact_key]
        candidate_identity = artifact_identities["candidate"][artifact_key]
        if (
            baseline_identity is not None
            and candidate_identity is not None
            and baseline_identity == candidate_identity
        ):
            raise AggregateError(
                f"M287 baseline candidate {label} must identify distinct files"
            )

    seen_artifact_identities: dict[tuple[int, int], tuple[str, str]] = {}
    for side in ("baseline", "candidate"):
        for artifact_key, label in (
            ("screenshot", "screenshot"),
            ("accessibility_tree", "accessibility tree"),
        ):
            identity = artifact_identities[side][artifact_key]
            if identity is None:
                continue
            previous = seen_artifact_identities.get(identity)
            if previous is not None:
                previous_side, previous_label = previous
                raise AggregateError(
                    f"M287 {side} {label} and {previous_side} {previous_label} "
                    "must identify distinct files"
                )
            seen_artifact_identities[identity] = (side, label)

    if (states["visual"] == "passed"
            and any(snapshots[side]["screenshot_state"] != "passed"
                    for side in ("baseline", "candidate"))):
        raise AggregateError("M287 visual pass requires both screenshot captures to pass")
    if (states["accessibility"] == "passed"
            and any(snapshots[side]["accessibility_state"] != "passed"
                    for side in ("baseline", "candidate"))):
        raise AggregateError("M287 accessibility pass requires both AX captures to pass")
    comparison_states = (states["visual"], states["accessibility"])
    if all(state == "passed" for state in comparison_states):
        expected_gui_state = "passed"
    elif "failed" in comparison_states:
        expected_gui_state = "failed"
    elif "inconclusive" in comparison_states:
        expected_gui_state = "inconclusive"
    else:
        expected_gui_state = "not_run"
    if states["gui"] != expected_gui_state:
        raise AggregateError("M287 gui state conflicts with visual/accessibility states")

    return {
        "candidate_id": candidate_id,
        "manifest_sha256": manifest_hash,
        "source_hash_manifest_sha256": source_manifest_hash,
        "build_summary_path": build_path.as_posix(),
        "build_summary_sha256": _sha256(build_raw),
        "gate_states": dict(states),
        "snapshots": retained_snapshots,
    }


def _aggregate_followups(root: Path | PinnedDir, root_lexical: Path | None, candidate_id: str, manifest_hash: str,
                         source_manifest_hash: str, manifest: Any,
                         source_manifest: Any, summaries: list[Path]
                         ) -> tuple[dict[str, str], list[dict[str, Any]]]:
    states: dict[str, str] = {}
    records: dict[str, tuple[Path, bytes, Any]] = {}
    gui_evidence: dict[str, Any] | None = None
    for path in summaries:
        relative, raw = _record_path(root, root_lexical, path, "follow-up summary",
                                     relative_to_root=True)
        summary = _load_json(raw, "follow-up summary")
        milestone = summary.get("milestone") if isinstance(summary, dict) else None
        if milestone not in ("M274", "M276", "M287"):
            raise AggregateError("follow-up summary milestone must be M274, M276, or M287")
        if milestone in records:
            raise AggregateError(f"duplicate {milestone} follow-up summary")
        _check_summary_identity(summary, milestone=milestone, candidate_id=candidate_id,
                                manifest_hash=manifest_hash)
        records[milestone] = (relative, raw, summary)

    if any(milestone in records for milestone in ("M276", "M287")) and "M274" not in records:
        raise AggregateError("M276/M287 follow-up requires its linked M274 build summary")

    if "M274" in records:
        relative, _, summary = records["M274"]
        expected_gates = {"build": "passed", "correctness": "not_run", "gui": "not_run",
                          "performance": "not_run", "promotion": "not_run"}
        if (summary.get("gate") != "build" or summary.get("status") != "passed"
                or summary.get("gates") != expected_gates):
            raise AggregateError("M274 summary does not record only the passed build gate")
        events = summary.get("events")
        expected_events = {"baseline_check", "baseline_build", "candidate_check", "candidate_build"}
        expected_steps = {
            "baseline_check": "P3 baseline workspace all-targets check",
            "baseline_build": "P3 baseline workspace all-targets build",
            "candidate_check": "candidate-workspace-all-targets-check",
            "candidate_build": "candidate-workspace-all-targets-build",
        }
        if not isinstance(events, dict) or set(events) != expected_events:
            raise AggregateError("M274 summary must contain the four paired build/check events")
        for name, event in events.items():
            if not isinstance(event, dict):
                raise AggregateError(f"M274 {name} record is invalid")
            _check_followup_event(
                root, event_path=event.get("event_path"), event_hash=event.get("event_sha256"),
                log_path=event.get("log_path"), log_hash=event.get("log_sha256"),
                manifest_hash=manifest_hash, candidate_id=candidate_id,
                required_identity_field=("candidate_id" if name.startswith("baseline_")
                                         else "manifest_sha256"),
                summary_exit_status=event.get("exit_status"),
                expected_step=expected_steps[name], label=f"M274 {name}",
            )
        states["build"] = "passed"

    if "M276" in records:
        _, _, summary = records["M276"]
        build_relative, build_raw, _ = records["M274"]
        linked_build_path = _safe_relative(summary.get("build_summary_path"),
                                           "M276 build_summary_path")
        if (linked_build_path != build_relative
                or summary.get("build_summary_sha256") != _sha256(build_raw)):
            raise AggregateError("M276 build summary path or digest does not match supplied M274 summary")
        expected_gates = {"build": "passed", "correctness": "passed", "gui": "not_run",
                          "performance": "not_run", "promotion": "not_run"}
        if (summary.get("gate") != "correctness" or summary.get("status") != "passed"
                or summary.get("gates") != expected_gates):
            raise AggregateError("M276 summary does not record passed build/correctness only")
        _check_m276_source_identity(root, summary, manifest, root_lexical)
        _check_m276_success(root, summary, "M276 correctness")
        states["build"] = "passed"
        states["correctness"] = "passed"

    if "M287" in records:
        m274_record = records["M274"]
        _, _, summary = records["M287"]
        gui_evidence = _check_m287_gui(
            root, summary, candidate_id=candidate_id, manifest_hash=manifest_hash,
            source_manifest_hash=source_manifest_hash, source_manifest=source_manifest,
            build_summary=m274_record,
        )
        states["gui"] = gui_evidence["gate_states"]["gui"]

    followup_records: list[dict[str, Any]] = []
    for milestone, (relative, raw, _) in sorted(records.items()):
        record: dict[str, Any] = {
            "milestone": milestone, "path": relative.as_posix(), "sha256": _sha256(raw),
        }
        if milestone == "M287" and gui_evidence is not None:
            record.update(gui_evidence)
        followup_records.append(record)
    return states, followup_records


def _record_path(root: Path | PinnedDir, root_lexical: Path | None, value: Path, label: str,
                 *, relative_to_root: bool = False) -> tuple[Path, bytes]:
    if isinstance(root, PinnedDir) and root_lexical is None:
        try:
            relative = Path(*relative_parts(value.as_posix()))
            raw = _read_file(root, relative, label, max_bytes=MAX_RECORD_BYTES)
            return relative, raw
        except CandidateFSError as error:
            raise AggregateError(f"{label} must be a safe candidate-relative path") from error
    assert root_lexical is not None
    try:
        path = value if value.is_absolute() else (
            root_lexical / value if relative_to_root else Path.cwd() / value
        )
        lexical_path = Path(os.path.abspath(path))
        try:
            relative = lexical_path.relative_to(root_lexical)
        except ValueError:
            # Reject aliases that resolve into the root instead of trusting their target.
            common = Path(os.path.commonpath((root_lexical, lexical_path)))
            current = common
            for part in lexical_path.relative_to(common).parts:
                current /= part
                try:
                    if stat.S_ISLNK(current.lstat().st_mode):
                        raise AggregateError(f"symlink rejected in {label}: {current}")
                except FileNotFoundError:
                    raise
            resolved_root = root_lexical if isinstance(root, PinnedDir) else root
            relative = lexical_path.resolve(strict=True).relative_to(resolved_root)
    except AggregateError:
        raise
    except (OSError, RuntimeError, ValueError) as error:
        raise AggregateError(f"{label} must be an existing file inside root") from error
    if not relative.parts:
        raise AggregateError(f"{label} must be a file inside root")
    raw = _read_file(root, relative, label, max_bytes=MAX_RECORD_BYTES)
    return Path(*relative.parts), raw


def aggregate(result_path: Path, root: Path | PinnedDir, p3f_path: Path | None = None,
              followup_paths: list[Path] | None = None, *,
              path_root: Path | None = None) -> dict[str, Any]:
    """Reconcile recorded evidence; never execute commands or decide acceptance."""
    report: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "status": "invalid",
        "scope": SCOPE,
        "candidate_id": None,
        "manifest_sha256": None,
        "pipeline_status": None,
        "gate_states": {name: "not_run" for name in PIPELINE_GATES},
        "event_count": 0,
        "followup_summaries": [],
        "errors": [],
    }
    errors: list[str] = report["errors"]
    try:
        if isinstance(root, PinnedDir):
            root_lexical = path_root
        else:
            root_lexical = root.absolute()
            root = root.resolve(strict=True)
            if not root.is_dir():
                raise AggregateError("root must be an existing directory")
        _, result_raw = _record_path(root, root_lexical, result_path, "pipeline result")
        result = _load_json(result_raw, "pipeline result")
        if not isinstance(result, dict) or set(result) != {
            "schema_version", "candidate_id", "manifest_sha256", "status", "scope",
            "events", "gates", "source_hash_manifest_sha256",
        }:
            raise AggregateError("pipeline result has missing or unknown fields")
        if type(result["schema_version"]) is not int or result["schema_version"] != 1:
            raise AggregateError("pipeline result schema_version must be 1")
        candidate_id = result["candidate_id"]
        if (not isinstance(candidate_id, str) or CANDIDATE_ID_RE.fullmatch(candidate_id) is None
                or ".." in candidate_id):
            raise AggregateError("pipeline result candidate_id is invalid")
        manifest_hash = result["manifest_sha256"]
        source_hash_manifest_hash = result["source_hash_manifest_sha256"]
        for value, label in ((manifest_hash, "manifest_sha256"),
                             (source_hash_manifest_hash, "source_hash_manifest_sha256")):
            if not isinstance(value, str) or HASH_RE.fullmatch(value) is None:
                raise AggregateError(f"pipeline result {label} must be a lowercase SHA-256 digest")
        if not isinstance(result["scope"], str):
            raise AggregateError("pipeline result scope must be a string")
        if result["status"] not in ("partial", "failed"):
            raise AggregateError("pipeline result status must be partial or failed")
        report.update(candidate_id=candidate_id, manifest_sha256=manifest_hash,
                      pipeline_status=result["status"])

        manifest_raw = _read_file(
            root, Path("manifest.json"), "candidate manifest",
            max_bytes=MAX_SNAPSHOT_MANIFEST_BYTES,
        )
        if _sha256(manifest_raw) != manifest_hash:
            raise AggregateError("pipeline result manifest_sha256 does not match manifest.json")
        manifest = _load_json(manifest_raw, "candidate manifest")
        if not isinstance(manifest, dict) or manifest.get("candidate_id") != candidate_id:
            raise AggregateError("candidate manifest identity does not match pipeline result")

        source_manifest_raw = _read_file(
            root, Path("source-hash-manifest.json"), "source hash manifest",
            max_bytes=MAX_SNAPSHOT_MANIFEST_BYTES,
        )
        if _sha256(source_manifest_raw) != source_hash_manifest_hash:
            raise AggregateError("source_hash_manifest_sha256 does not match source-hash-manifest.json")
        source_manifest = _load_json(source_manifest_raw, "source hash manifest")
        if (not isinstance(source_manifest, dict)
                or type(source_manifest.get("schema_version")) is not int
                or source_manifest["schema_version"] != 1):
            raise AggregateError("source hash manifest schema_version must be 1")
        if (source_manifest.get("candidate_id") != candidate_id
                or source_manifest.get("manifest_sha256") != manifest_hash):
            raise AggregateError("source hash manifest identity does not match pipeline result")
        candidate_snapshots = manifest.get("snapshots")
        source_snapshots = source_manifest.get("snapshots")
        if not isinstance(candidate_snapshots, dict) or not isinstance(source_snapshots, dict):
            raise AggregateError("source hash manifest snapshots are missing")
        for side in ("baseline", "candidate"):
            candidate_snapshot = candidate_snapshots.get(side)
            source_snapshot = source_snapshots.get(side)
            if not isinstance(candidate_snapshot, dict) or not isinstance(source_snapshot, dict):
                raise AggregateError(f"source hash manifest {side} snapshot is missing")
            candidate_files = candidate_snapshot.get("files")
            source_files = source_snapshot.get("files")
            if not _snapshot_file_maps_match(source_files, candidate_files):
                raise AggregateError(
                    f"source hash manifest {side} snapshot files conflict with candidate manifest"
                )
            tree_hash = source_snapshot.get("tree_sha256")
            if (not isinstance(tree_hash, str) or HASH_RE.fullmatch(tree_hash) is None
                    or tree_hash != _sha256(json.dumps(source_files, sort_keys=True).encode())):
                raise AggregateError(f"source hash manifest {side} snapshot tree hash is invalid")
        source_input_hash = source_manifest.get("source_snapshot_manifest_sha256")
        if not isinstance(source_input_hash, str) or HASH_RE.fullmatch(source_input_hash) is None:
            raise AggregateError("source hash manifest has no valid source snapshot manifest digest")
        if source_manifest.get("source_manifest_sha256", source_input_hash) != source_input_hash:
            raise AggregateError("source hash manifest source digests conflict")

        if p3f_path is None:
            p3f_path = Path("evidence/P3F-record.json")
        _, p3f_raw = _record_path(root, root_lexical, p3f_path, "P3F evidence record",
                                  relative_to_root=True)
        p3f_record = _load_json(p3f_raw, "P3F evidence record")
        p3f_report = validate_p3f(p3f_record, root)
        if p3f_report.get("status") != "valid":
            raise AggregateError("P3F evidence record is invalid: " + "; ".join(p3f_report.get("errors", [])))
        if p3f_record.get("manifest_sha256") != manifest_hash:
            raise AggregateError("P3F manifest_sha256 conflicts with pipeline result")

        # P3F's verifier checks content hashes; also reject symlinked referenced artifacts.
        p3f_artifacts = [p3f_record.get("log"), *p3f_record.get("evidence", [])]
        p3f_artifact_bytes = 0
        intake_raw: bytes | None = None
        for index, item in enumerate(p3f_artifacts):
            raw = _artifact(
                root, Path(), item, f"P3F artifact[{index}]",
                max_bytes=MAX_P3F_ARTIFACT_BYTES - p3f_artifact_bytes,
            )
            p3f_artifact_bytes += len(raw)
            if isinstance(item, dict) and item.get("path") == "approved-intake.json":
                intake_raw = raw
        intake_artifacts = [item for item in p3f_record["evidence"]
                            if item.get("path") == "approved-intake.json"]
        if len(intake_artifacts) != 1:
            raise AggregateError("P3F record must bind exactly one approved-intake.json artifact")
        if p3f_record.get("provenance", {}).get("approved_intake_sha256") != intake_artifacts[0]["sha256"]:
            raise AggregateError("P3F approved intake provenance conflicts with its artifact hash")
        if intake_raw is None:
            raise AggregateError("P3F approved-intake.json artifact was not read")
        intake = _load_json(intake_raw, "approved intake")
        if (not isinstance(intake, dict) or intake.get("approved") is not True
                or intake.get("candidate_id") != candidate_id):
            raise AggregateError("approved intake candidate identity does not match pipeline result")
        source_ref = intake.get("source_manifest")
        if not isinstance(source_ref, dict) or source_ref.get("sha256") != source_input_hash:
            raise AggregateError("approved intake source manifest hash conflicts with source hash manifest")
        if p3f_record.get("provenance", {}).get("source_candidate_id") != intake.get("source_candidate_id"):
            raise AggregateError("P3F source candidate provenance conflicts with approved intake")

        gates = result["gates"]
        if not isinstance(gates, dict) or set(gates) != set(PIPELINE_GATES):
            raise AggregateError("pipeline result gates must contain exactly build, correctness, gui, performance, and promotion")
        p3f_gates = p3f_record.get("gates")
        if not isinstance(p3f_gates, dict) or set(p3f_gates) != set(GATES):
            raise AggregateError("P3F gates must contain exactly build, correctness, gui, and performance")
        for name in GATES:
            if gates[name] not in GATE_STATES or p3f_gates[name] not in GATE_STATES:
                raise AggregateError(f"gate {name} has an invalid state")
            if gates[name] != p3f_gates[name]:
                raise AggregateError(f"gate {name} conflicts between pipeline result and P3F record")
        if gates["promotion"] != "not_run":
            raise AggregateError("promotion must remain not_run; aggregation cannot promote candidates")

        events = result["events"]
        if not isinstance(events, list) or not events:
            raise AggregateError("pipeline result events must be a nonempty array")
        seen_event_paths: set[str] = set()
        seen_steps: set[str] = set()
        failed_event = False
        p3f_event_log: bytes | None = None
        for index, event in enumerate(events):
            label = f"events[{index}]"
            if not isinstance(event, dict):
                raise AggregateError(f"{label} must be an object")
            step = event.get("step")
            if not isinstance(step, str) or not step or step in seen_steps:
                raise AggregateError(f"{label}.step must be unique and nonempty")
            seen_steps.add(step)
            if type(event.get("schema_version")) is not int or event["schema_version"] != 1:
                raise AggregateError(f"{label}.schema_version must be 1")
            if event.get("status") not in ("passed", "failed", "not_run"):
                raise AggregateError(f"{label}.status is invalid")
            exit_status = event.get("exit_status")
            if exit_status is not None and type(exit_status) is not int:
                raise AggregateError(f"{label}.exit_status must be an integer or null")
            if event["status"] == "passed" and exit_status != 0:
                raise AggregateError(f"{label} passed without exit_status 0")
            if event["status"] == "not_run" and exit_status is not None:
                raise AggregateError(f"{label} not_run must have a null exit_status")
            if event["status"] == "failed":
                if "command" in event and (type(exit_status) is not int or exit_status == 0):
                    raise AggregateError(
                        f"{label} failed command must have a nonzero integer exit_status"
                    )
                if exit_status == 0:
                    raise AggregateError(f"{label} failed with exit_status 0")
                failed_event = True
            allowed_manifest_hash = (source_input_hash if step in
                                     ("P3E-patch-plan", "P3E-patch-binding") else manifest_hash)
            if event.get("manifest_sha256") != allowed_manifest_hash:
                raise AggregateError(f"{label}.manifest_sha256 conflicts with its intake stage")
            event_ref = event.get("event_record")
            if not isinstance(event_ref, dict) or set(event_ref) != {"path", "sha256"}:
                raise AggregateError(f"{label}.event_record must contain exactly path and sha256")
            event_path = event_ref.get("path")
            event_relative = _safe_relative(event_path, f"{label}.event_record.path")
            normalized_path = event_relative.as_posix()
            if normalized_path in seen_event_paths:
                raise AggregateError(f"{label}.event_record path is duplicated")
            seen_event_paths.add(normalized_path)
            embedded = {key: value for key, value in event.items() if key != "event_record"}
            event_raw = _artifact(
                root, Path("evidence"), event_ref, f"{label}.event_record",
                max_bytes=MAX_RECORD_BYTES,
            )
            if not _same_json_value(_load_json(event_raw, f"{label} event"), embedded):
                raise AggregateError(f"{label} event file conflicts with pipeline result")
            if isinstance(event.get("log"), dict):
                log_raw = _artifact(
                    root, Path("evidence"), event["log"], f"{label}.log",
                    max_bytes=(MAX_RECORD_BYTES if step == "P3F-evidence-record"
                               else MAX_COMMAND_LOG_BYTES if "command" in event else None),
                )
                if step == "P3F-evidence-record":
                    p3f_event_log = log_raw
            elif "command" in event:
                raise AggregateError(f"{label} command event has no hash-bound log")
            elif event["status"] == "passed" and not isinstance(event.get("reason"), str):
                raise AggregateError(f"{label} internal pass event has no recorded reason")
            expected_validator = COMMAND_VALIDATORS.get(step)
            command = event.get("command")
            if expected_validator is not None:
                expected_path = Path(__file__).resolve().parent / expected_validator
                if (not isinstance(command, list) or len(command) < 2
                        or not all(isinstance(argument, str) for argument in command)
                        or (Path(command[1]) != expected_path
                            and command[1] != expected_validator)):
                    raise AggregateError(
                        f"{label}.command does not identify the validator for {step}"
                    )
            elif command is not None:
                raise AggregateError(f"{label}.command is unsupported for step {step}")
        if (result["status"] == "partial" and failed_event) or (result["status"] == "failed" and not failed_event):
            raise AggregateError("pipeline result status conflicts with recorded validator events")
        expected_p3f_exit_status = 1 if result["status"] == "failed" else 0
        if p3f_record.get("exit_status") != expected_p3f_exit_status:
            raise AggregateError("P3F evidence record exit_status conflicts with pipeline status")
        p3f_event = next((event for event in events if event["step"] == "P3F-evidence-record"), None)
        if p3f_event is None or p3f_event_log is None:
            raise AggregateError("pipeline result lacks the P3F validator event and log")
        if p3f_event.get("status", "passed") != "passed" or p3f_event.get("exit_status") != 0:
            raise AggregateError("P3F validator event is not a recorded pass")
        p3f_event_report = _load_json(p3f_event_log, "P3F validator output")
        if (not isinstance(p3f_event_report, dict)
                or type(p3f_event_report.get("schema_version")) is not int
                or p3f_event_report["schema_version"] != 1
                or p3f_event_report.get("status") != "valid"
                or p3f_event_report.get("manifest_sha256") != manifest_hash
                or p3f_event_report.get("gate_states") != p3f_gates
                or p3f_event_report.get("errors") != []):
            raise AggregateError("P3F validator event does not record a valid matching evidence record")

        followup_states, followup_records = _aggregate_followups(
            root, root_lexical, candidate_id, manifest_hash, source_hash_manifest_hash,
            manifest, source_manifest, followup_paths or [],
        )
        reconciled_gates = {name: gates[name] for name in GATES} | {"promotion": "not_run"}
        reconciled_gates.update(followup_states)
        report.update(status="consistent", gate_states=reconciled_gates,
                      event_count=len(events), followup_summaries=followup_records)
    except (AggregateError, OSError, RuntimeError, ValueError, TypeError, KeyError) as error:
        errors.append(str(error))
    return report


def _candidate_relative_from_repo(value: Path, candidate_prefix: Path, label: str) -> Path:
    """Translate a runner CLI path to the candidate descriptor's relative namespace."""
    try:
        parts = relative_parts(value.as_posix())
    except CandidateFSError as error:
        raise AggregateError(f"{label} must be a repository-relative path") from error
    prefix = candidate_prefix.parts
    if parts[:len(prefix)] != prefix or len(parts) == len(prefix):
        raise AggregateError(f"{label} must be beneath the pinned candidate root")
    return Path(*parts[len(prefix):])


def _invalid_report(error: Exception) -> dict[str, Any]:
    return {
        "schema_version": SCHEMA_VERSION,
        "status": "invalid",
        "scope": SCOPE,
        "candidate_id": None,
        "manifest_sha256": None,
        "pipeline_status": None,
        "gate_states": {name: "not_run" for name in PIPELINE_GATES},
        "event_count": 0,
        "followup_summaries": [],
        "errors": [str(error)],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pipeline_result", type=Path, help="pipeline-result.json")
    parser.add_argument("--root", type=Path, required=True, help="candidate root for referenced artifacts")
    parser.add_argument("--p3f-record", type=Path, help="P3F record (defaults to evidence/P3F-record.json)")
    parser.add_argument("--followup-summary", type=Path, action="append", default=[],
                        help="hash-linked M274/M276/M287 gate summary under --root (repeatable)")
    parser.add_argument("--output", type=Path,
                        help="optional report path beneath --root (written atomically)")
    args = parser.parse_args()
    inherited = (
        os.environ.get("RUSTUI_P3_REPO_ROOT_FD"),
        os.environ.get("RUSTUI_P3_CANDIDATE_PARENT_FD"),
        os.environ.get("RUSTUI_P3_CANDIDATE_ROOT_FD"),
        os.environ.get("RUSTUI_P3_CANDIDATE_ID"),
    )
    if any(value is not None for value in inherited):
        repo: PinnedDir | None = None
        parent: PinnedDir | None = None
        candidate: PinnedDir | None = None
        try:
            if not all(value is not None for value in inherited):
                raise AggregateError("incomplete inherited P3 root identity")
            repo_fd, parent_fd, candidate_fd = (int(inherited[index]) for index in range(3))
            candidate_id = inherited[3]
            assert candidate_id is not None
            if (CANDIDATE_ID_RE.fullmatch(candidate_id) is None or ".." in candidate_id):
                raise AggregateError("inherited candidate ID is invalid")
            repo = PinnedDir.from_fd(repo_fd)
            parent = PinnedDir.from_fd(parent_fd)
            candidate = PinnedDir.from_fd(candidate_fd)
            expected_parent = PinnedDir.open_child(repo, "target/headstart/candidates")
            try:
                if expected_parent.identity() != parent.identity():
                    raise AggregateError("inherited candidate parent does not match repository root")
            finally:
                expected_parent.close()
            expected_candidate = PinnedDir.open_child(parent, candidate_id)
            try:
                if expected_candidate.identity() != candidate.identity():
                    raise AggregateError("inherited candidate root does not match candidate ID")
            finally:
                expected_candidate.close()

            candidate_prefix = Path("target/headstart/candidates") / candidate_id
            if args.root != candidate_prefix:
                raise AggregateError("--root does not match the inherited candidate identity")
            result_relative = _candidate_relative_from_repo(
                args.pipeline_result, candidate_prefix, "pipeline result",
            )
            p3f_relative = (Path("evidence/P3F-record.json") if args.p3f_record is None else
                            _candidate_relative_from_repo(
                                args.p3f_record, candidate_prefix, "P3F record",
                            ))
            followup_relatives = [
                _candidate_relative_from_repo(path, candidate_prefix, "follow-up summary")
                for path in args.followup_summary
            ]
            output_relative = (None if args.output is None else
                               _candidate_relative_from_repo(args.output, candidate_prefix, "report output"))
            report = aggregate(result_relative, candidate, p3f_relative, followup_relatives)
            if report.get("candidate_id") not in (None, candidate_id):
                report["status"] = "invalid"
                report["errors"].append("pipeline result identity does not match inherited candidate ID")
                report["gate_states"] = {name: "not_run" for name in PIPELINE_GATES}
            if output_relative is not None:
                candidate.atomic_write(
                    output_relative,
                    (json.dumps(report, sort_keys=True) + "\n").encode(),
                )
        except (AggregateError, CandidateFSError, OSError, ValueError) as error:
            report = _invalid_report(error)
        finally:
            for pinned in (candidate, parent, repo):
                if pinned is not None:
                    pinned.close()
    else:
        try:
            resolved_root = args.root.resolve(strict=True)
            with PinnedDir.open_root(resolved_root) as output_root:
                report = aggregate(
                    args.pipeline_result, output_root, args.p3f_record,
                    args.followup_summary, path_root=args.root.absolute(),
                )
                if args.output is not None:
                    if args.output.is_absolute():
                        output_relative = args.output.absolute().relative_to(resolved_root)
                    else:
                        output_relative = Path(*relative_parts(args.output.as_posix()))
                    output_root.atomic_write(
                        output_relative, (json.dumps(report, sort_keys=True) + "\n").encode(),
                    )
        except (AggregateError, CandidateFSError, OSError, ValueError) as error:
            report = _invalid_report(error)
    print(json.dumps(report, sort_keys=True))
    return 0 if report["status"] == "consistent" else 1


if __name__ == "__main__":
    raise SystemExit(main())
