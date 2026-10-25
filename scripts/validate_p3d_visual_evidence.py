#!/usr/bin/env python3
"""Check P3D four-state screenshot evidence references and review fields.

This checks record completeness and file hashes only. It does not inspect pixels,
decide visual equivalence, or assign acceptance thresholds.
"""

import argparse
import hashlib
import json
import math
import os
from pathlib import Path, PurePosixPath, PureWindowsPath
import re
import stat
import sys

try:
    from p3_candidate_fs import CandidateFSError, PinnedDir, relative_parts
except ModuleNotFoundError:  # importlib-based tests load this file outside scripts/ on sys.path
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    from p3_candidate_fs import CandidateFSError, PinnedDir, relative_parts


ROOT = Path(__file__).parent.parent
STATES = (
    "cached_initial",
    "cached_after_mutation",
    "uncached_initial",
    "uncached_after_mutation",
)
HEX = re.compile(r"^[0-9a-f]{64}$")
READ_CHUNK_BYTES = 1024 * 1024
MAX_RECORD_BYTES = 1024 * 1024


class InvalidEvidence(ValueError):
    pass


def _unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise InvalidEvidence(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def _reject_constant(value):
    raise InvalidEvidence(f"non-standard JSON numeric constant: {value}")


def _finite_float(value):
    parsed = float(value)
    if not math.isfinite(parsed):
        raise InvalidEvidence("JSON number is outside the finite float range")
    return parsed


def require_sha(value, label):
    if not isinstance(value, str) or not HEX.fullmatch(value):
        raise InvalidEvidence(f"{label} must be a lowercase SHA-256")


def require_scale_factor(value):
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise InvalidEvidence("provenance.scale_factor must be a positive finite number")
    try:
        scale_factor = float(value)
    except (OverflowError, TypeError, ValueError):
        raise InvalidEvidence("provenance.scale_factor must be a positive finite number") from None
    if not math.isfinite(scale_factor) or scale_factor <= 0:
        raise InvalidEvidence("provenance.scale_factor must be a positive finite number")


def matches_exact_json_value(value, expected):
    if type(value) is not type(expected):
        return False
    if isinstance(expected, dict):
        return (value.keys() == expected.keys()
                and all(matches_exact_json_value(value[key], expected[key]) for key in expected))
    if isinstance(expected, list):
        return (len(value) == len(expected)
                and all(matches_exact_json_value(actual, wanted)
                        for actual, wanted in zip(value, expected)))
    return value == expected


def _read_file(root: PinnedDir, relative: str, label: str, *,
               retain_contents=True, max_bytes=None) -> tuple[bytes | str, tuple[int, int]]:
    try:
        fd = root.open_file(relative, os.O_RDONLY | getattr(os, "O_NONBLOCK", 0))
        with os.fdopen(fd, "rb") as stream:
            before = os.fstat(stream.fileno())
            digest = hashlib.sha256()
            chunks = []
            total_bytes = 0
            while block := stream.read(READ_CHUNK_BYTES):
                total_bytes += len(block)
                if max_bytes is not None and total_bytes > max_bytes:
                    raise InvalidEvidence(f"{label} exceeds the {max_bytes}-byte record limit")
                digest.update(block)
                if retain_contents:
                    chunks.append(block)
            after = os.fstat(stream.fileno())
    except (OSError, CandidateFSError) as error:
        raise InvalidEvidence(f"{label} path cannot be read safely") from error
    if before.st_nlink > 1 or after.st_nlink > 1:
        raise InvalidEvidence(f"{label} path has hard-link aliases")
    identity = (before.st_dev, before.st_ino)
    version = (before.st_size, before.st_mtime_ns, before.st_ctime_ns)
    if (not stat.S_ISREG(before.st_mode) or identity != (after.st_dev, after.st_ino)
            or version != (after.st_size, after.st_mtime_ns, after.st_ctime_ns)):
        raise InvalidEvidence(f"{label} path changed while its digest was being checked")
    try:
        current_fd = root.open_file(relative, os.O_RDONLY | getattr(os, "O_NONBLOCK", 0))
        with os.fdopen(current_fd, "rb") as current:
            current_stat = os.fstat(current.fileno())
    except (OSError, CandidateFSError) as error:
        raise InvalidEvidence(f"{label} path changed while its digest was being checked") from error
    if current_stat.st_nlink > 1:
        raise InvalidEvidence(f"{label} path has hard-link aliases")
    if (identity != (current_stat.st_dev, current_stat.st_ino)
            or version != (current_stat.st_size, current_stat.st_mtime_ns,
                           current_stat.st_ctime_ns)):
        raise InvalidEvidence(f"{label} path changed while its digest was being checked")
    content = b"".join(chunks) if retain_contents else digest.hexdigest()
    return content, identity


def _artifact_relative(value, label, candidate_id=None):
    if not isinstance(value, dict) or set(value) != {"path", "sha256"}:
        raise InvalidEvidence(f"{label} must contain path and sha256")
    relative = value["path"]
    if isinstance(relative, str) and "\x00" in relative:
        raise InvalidEvidence(f"{label} path cannot be resolved")
    if (not isinstance(relative, str) or not relative or "\\" in relative):
        raise InvalidEvidence(f"{label} path must be relative")
    posix_path, windows_path = PurePosixPath(relative), PureWindowsPath(relative)
    if (posix_path.is_absolute() or windows_path.is_absolute() or windows_path.drive
            or any(part in ("", ".", "..") for part in relative.split("/"))):
        raise InvalidEvidence(f"{label} path must be a safe relative path")
    relative = "/".join(relative.split("/"))
    if candidate_id is not None:
        prefix = f"target/headstart/candidates/{candidate_id}/"
        if relative.startswith(prefix):
            relative = relative[len(prefix):]
        elif relative.startswith("target/headstart/candidates/"):
            raise InvalidEvidence(f"{label} path does not match the pinned candidate id")
    return relative


def artifact(root: PinnedDir, value, label, candidate_id=None):
    if not isinstance(root, PinnedDir):
        with PinnedDir.open_root(root) as pinned:
            return artifact(pinned, value, label, candidate_id)
    relative = _artifact_relative(value, label, candidate_id)
    require_sha(value["sha256"], f"{label}.sha256")
    digest, identity = _read_file(root, relative, label, retain_contents=False)
    if digest != value["sha256"]:
        raise InvalidEvidence(f"{label} hash mismatch: {relative}")
    return identity


def artifact_json(root, value, label, candidate_id=None, *, with_identity=False):
    if not isinstance(root, PinnedDir):
        with PinnedDir.open_root(root) as pinned:
            return artifact_json(pinned, value, label, candidate_id,
                                 with_identity=with_identity)
    relative = _artifact_relative(value, label, candidate_id)
    require_sha(value["sha256"], f"{label}.sha256")
    raw, identity = _read_file(root, relative, label, max_bytes=MAX_RECORD_BYTES)
    if hashlib.sha256(raw).hexdigest() != value["sha256"]:
        raise InvalidEvidence(f"{label} hash mismatch while reading")
    try:
        document = json.loads(raw, object_pairs_hook=_unique_object,
                              parse_constant=_reject_constant, parse_float=_finite_float)
    except UnicodeDecodeError as error:
        raise InvalidEvidence(f"{label} must contain valid UTF-8") from error
    except json.JSONDecodeError as error:
        raise InvalidEvidence(f"{label} must contain valid JSON") from error
    if not isinstance(document, dict):
        raise InvalidEvidence(f"{label} must contain a JSON object")
    return (document, identity) if with_identity else document


def validate_observation(state, name):
    observation = state.get("observation")
    if not isinstance(observation, dict):
        raise InvalidEvidence(f"missing observation for {name}")
    if type(observation.get("all_32_values_visible")) is not bool:
        raise InvalidEvidence(f"{name}: all_32_values_visible must be reviewed as true/false")
    if type(observation.get("expected_state_visible")) is not bool:
        raise InvalidEvidence(f"{name}: expected_state_visible must be reviewed as true/false")
    for key in ("ax_active", "native_drag_active"):
        if type(observation.get(key)) is not bool:
            raise InvalidEvidence(f"{name}: {key} must be recorded as true/false")
    for key in ("reviewer", "notes"):
        if not isinstance(observation.get(key), str) or not observation[key].strip():
            raise InvalidEvidence(f"{name}: observation.{key} must be recorded")


def _relative_manifest(path: Path, root_path: Path) -> str:
    lexical_root = Path(os.path.abspath(root_path))
    lexical_path = Path(os.path.abspath(path if path.is_absolute() else Path.cwd() / path))
    try:
        relative = lexical_path.relative_to(lexical_root)
    except ValueError as error:
        raise InvalidEvidence("P3D visual evidence path must be beneath the repository root") from error
    try:
        return "/".join(relative_parts(relative))
    except CandidateFSError as error:
        raise InvalidEvidence("P3D visual evidence path must be a safe relative path") from error


def _candidate_context():
    names = ("RUSTUI_P3_REPO_ROOT_FD", "RUSTUI_P3_CANDIDATE_PARENT_FD",
             "RUSTUI_P3_CANDIDATE_ROOT_FD", "RUSTUI_P3_CANDIDATE_ID")
    values = [os.environ.get(name) for name in names]
    if not any(value is not None for value in values):
        return None
    if any(value is None for value in values):
        raise InvalidEvidence("inherited candidate directory descriptors are incomplete")
    try:
        repo_fd, parent_fd, candidate_fd = (int(values[index]) for index in range(3))
        candidate_id = values[3]
        if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]{0,63}", candidate_id) or ".." in candidate_id:
            raise InvalidEvidence("inherited candidate id is invalid")
        repo = PinnedDir.from_fd(repo_fd)
        parent = PinnedDir.from_fd(parent_fd)
        candidate = PinnedDir.from_fd(candidate_fd)
    except (ValueError, CandidateFSError, OSError) as error:
        raise InvalidEvidence("inherited candidate directory descriptors are invalid") from error
    try:
        expected_parent = PinnedDir.open_child(repo, "target/headstart/candidates")
        expected_candidate = PinnedDir.open_child(parent, candidate_id)
        try:
            parent_stat, expected_parent_stat = os.fstat(parent.fd), os.fstat(expected_parent.fd)
            if (parent_stat.st_dev, parent_stat.st_ino) != (expected_parent_stat.st_dev,
                                                            expected_parent_stat.st_ino):
                raise InvalidEvidence("candidate parent does not match the repository candidate directory")
            candidate_stat = os.fstat(candidate.fd)
            expected_candidate_stat = os.fstat(expected_candidate.fd)
            if (candidate_stat.st_dev, candidate_stat.st_ino) != (
                    expected_candidate_stat.st_dev, expected_candidate_stat.st_ino):
                raise InvalidEvidence("candidate id does not match the pinned candidate directory")
        finally:
            expected_parent.close()
            expected_candidate.close()
        return repo, parent, candidate, candidate_id
    except Exception:
        repo.close()
        parent.close()
        candidate.close()
        raise


def validate(path, *, root_path=None, pinned_root=None, candidate_id=None):
    owned_root = pinned_root is None
    if owned_root:
        try:
            pinned_root = PinnedDir.open_root(root_path or ROOT)
        except (OSError, CandidateFSError) as error:
            raise InvalidEvidence("repository root cannot be opened") from error
    root_path = Path(root_path or ROOT)
    manifest_root = None
    manifest_root_owned = False
    try:
        if candidate_id is not None:
            prefix = f"target/headstart/candidates/{candidate_id}/"
            path_text = os.fspath(path)
            if Path(path_text).is_absolute():
                relative_repo = _relative_manifest(Path(path_text), root_path)
            else:
                relative_repo = "/".join(relative_parts(path_text))
            if not relative_repo.startswith(prefix):
                raise InvalidEvidence("P3D visual evidence path does not match the pinned candidate id")
            manifest_relative = relative_repo[len(prefix):]
            artifacts_root = pinned_root
        else:
            try:
                manifest_relative = _relative_manifest(Path(path), root_path)
                manifest_root = pinned_root
            except InvalidEvidence:
                manifest_path = Path(os.path.abspath(path))
                manifest_root = PinnedDir.open_root(manifest_path.parent)
                manifest_root_owned = True
                manifest_relative = "/".join(relative_parts(manifest_path.name))
            artifacts_root = pinned_root
        raw, _ = _read_file(manifest_root or artifacts_root, manifest_relative,
                            "P3D visual evidence", max_bytes=MAX_RECORD_BYTES)
        try:
            doc = json.loads(raw, object_pairs_hook=_unique_object,
                             parse_constant=_reject_constant, parse_float=_finite_float)
        except UnicodeDecodeError as error:
            raise InvalidEvidence("P3D visual evidence must be valid UTF-8") from error
        if not isinstance(doc, dict):
            raise InvalidEvidence("P3D visual evidence must be a JSON object")
        return _validate_doc(doc, artifacts_root, candidate_id)
    finally:
        if manifest_root_owned:
            manifest_root.close()
        if owned_root:
            pinned_root.close()


def _validate_doc(doc, root, candidate_id=None):
    if (type(doc.get("schema_version")) is not int or doc["schema_version"] != 1
            or doc.get("kind") != "rustui_p3d_visual_evidence"):
        raise InvalidEvidence("unsupported P3D visual evidence schema")
    if not matches_exact_json_value(doc.get("workload"), {"name": "32-leaf", "leaves": 32}):
        raise InvalidEvidence("P3D native visual comparison must use the 32-leaf workload")

    provenance = doc.get("provenance")
    if not isinstance(provenance, dict):
        raise InvalidEvidence("missing provenance")
    build_ref = provenance.get("build_record")
    plan_ref = provenance.get("plan")
    build, build_file_id = artifact_json(
        root, build_ref, "provenance.build_record", candidate_id, with_identity=True
    )
    plan, plan_file_id = artifact_json(
        root, plan_ref, "provenance.plan", candidate_id, with_identity=True
    )
    if plan.get("build_record") != build_ref:
        raise InvalidEvidence("provenance.plan does not reference provenance.build_record")
    if not matches_exact_json_value(
        plan.get("workload"),
        {"name": "32-leaf", "leaves": 32, "text_lines_per_leaf": 12},
    ):
        raise InvalidEvidence("provenance.plan workload does not match the 32-leaf visual evidence")
    require_sha(provenance.get("binary_sha256"), "provenance.binary_sha256")
    binary_hash = provenance["binary_sha256"]
    build_binary = build.get("binary")
    plan_binary = plan.get("binary")
    if (not isinstance(build_binary, dict) or not isinstance(plan_binary, dict)
            or build_binary.get("sha256") != binary_hash
            or plan_binary.get("sha256") != binary_hash
            or build_binary.get("path") != plan_binary.get("path")):
        raise InvalidEvidence("build record, plan, and visual evidence binary provenance do not match")
    binary_file_id = artifact(root, {"path": build_binary.get("path"), "sha256": binary_hash},
                              "provenance.binary", candidate_id)
    provenance_files = {build_file_id, plan_file_id, binary_file_id}
    for key in ("platform", "appearance", "display"):
        value = provenance.get(key)
        if not isinstance(value, str) or not value.strip():
            raise InvalidEvidence(f"provenance.{key} must be a non-empty string")
    require_scale_factor(provenance.get("scale_factor"))
    if not matches_exact_json_value(provenance.get("capture_layout"), {
        "mode": "native_wide_8x4",
        "columns": 8,
        "window_logical_points": [1700, 1000],
        "leaf_logical_points": [210, 220],
        "lines_per_leaf": 12,
    }):
        raise InvalidEvidence("provenance.capture_layout does not match the P3D visual-capture layout")

    states = doc.get("states")
    if not isinstance(states, dict) or set(states) != set(STATES):
        raise InvalidEvidence("states must contain exactly the four cached/uncached initial/after-mutation states")
    incomplete = []
    screenshot_files = set()
    for name in STATES:
        state = states[name]
        status = state.get("status") if isinstance(state, dict) else None
        if not isinstance(status, str) or status not in {"not_run", "captured", "failed", "inconclusive"}:
            raise InvalidEvidence(f"invalid status for {name}")
        if status != "captured":
            incomplete.append(name)
            if status == "not_run" and state.get("screenshot") is not None:
                raise InvalidEvidence(f"{name}: not_run state cannot reference a screenshot")
            if status in {"failed", "inconclusive"}:
                validate_observation(state, name)
            if state.get("screenshot") is not None:
                file_id = artifact(root, state["screenshot"], f"states.{name}.screenshot",
                                   candidate_id)
                if file_id in provenance_files:
                    raise InvalidEvidence(f"{name}: screenshot must be distinct from provenance artifacts")
                if file_id in screenshot_files:
                    raise InvalidEvidence("each visual state must reference a distinct screenshot file")
                screenshot_files.add(file_id)
            continue
        file_id = artifact(root, state.get("screenshot"), f"states.{name}.screenshot",
                           candidate_id)
        if file_id in provenance_files:
            raise InvalidEvidence(f"{name}: screenshot must be distinct from provenance artifacts")
        if file_id in screenshot_files:
            raise InvalidEvidence("each visual state must reference a distinct screenshot file")
        screenshot_files.add(file_id)
        validate_observation(state, name)
    return incomplete


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", type=Path)
    args = parser.parse_args()
    context = None
    try:
        context = _candidate_context()
        if context is None:
            incomplete = validate(args.manifest)
        else:
            repo, parent, candidate, candidate_id = context
            incomplete = validate(args.manifest, pinned_root=candidate,
                                  candidate_id=candidate_id)
        if incomplete:
            print("incomplete: " + ", ".join(incomplete))
        else:
            print("record complete; visual acceptance remains a human review decision")
    except (OSError, json.JSONDecodeError, CandidateFSError, InvalidEvidence) as error:
        print(f"invalid P3D visual evidence: {error}", file=sys.stderr)
        return 2
    finally:
        if context is not None:
            for directory in context[:3]:
                directory.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
