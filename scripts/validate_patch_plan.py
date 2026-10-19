#!/usr/bin/env python3
"""Validate a local, immutable upstream patch-plan manifest without applying it."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path, PurePosixPath, PureWindowsPath
from typing import Any


SCHEMA_VERSION = 1
GATE_NAMES = ("build", "correctness", "gui", "performance")
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
REVISION_RE = re.compile(r"^[0-9a-f]{40}$")
IDENTIFIER_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]*$")


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


def _contained_file(root: Path, relative: Path, field: str, errors: list[str]) -> Path | None:
    candidate = root / relative
    try:
        resolved = candidate.resolve(strict=True)
        resolved.relative_to(root)
    except FileNotFoundError:
        errors.append(f"{field} does not exist under root")
        return None
    except (OSError, RuntimeError, ValueError):
        errors.append(f"{field} escapes root or cannot be resolved")
        return None
    if not resolved.is_file():
        errors.append(f"{field} is not a regular file")
        return None
    return resolved


def _hash_matches(path: Path, expected: Any, field: str, errors: list[str]) -> None:
    if not _sha256_is_valid(expected):
        errors.append(f"{field} must be a lowercase SHA-256 digest")
        return
    try:
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError as error:
        errors.append(f"{field} cannot be read: {error}")
        return
    if actual != expected:
        errors.append(f"{field} does not match the file SHA-256")


def _nonempty_string(value: Any, field: str, errors: list[str]) -> None:
    if not isinstance(value, str) or not value.strip() or "\x00" in value:
        errors.append(f"{field} must be a nonempty string")


def validate(manifest: Any, root: Path) -> dict[str, Any]:
    """Return a structured validation report; never modifies files or runs commands."""
    errors: list[str] = []
    root_resolved = root.resolve()
    report: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "status": "invalid",
        "scope": "manifest integrity only; validity does not imply approval or compatibility",
        "candidate_id": None,
        "source_revision": None,
        "patch_count": 0,
        "gates": {name: "not_run" for name in GATE_NAMES},
        "errors": errors,
    }

    if not root_resolved.is_dir():
        errors.append("root must be an existing directory")
        report["errors"] = errors
        return report
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
        if not isinstance(candidate_id, str) or IDENTIFIER_RE.fullmatch(candidate_id) is None or ".." in candidate_id:
            errors.append("candidate.id must be a safe nonempty identifier")
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
                    checksum_path = _contained_file(root_resolved, checksum_rel, f"{field}.checksum_file", errors)
                    if checksum_path is not None:
                        _hash_matches(checksum_path, package.get("checksum_sha256"), f"{field}.checksum_sha256", errors)
                elif not _sha256_is_valid(package.get("checksum_sha256")):
                    errors.append(f"{field}.checksum_sha256 must be a lowercase SHA-256 digest")

                license_files = package.get("license_files")
                if not isinstance(license_files, list) or not license_files:
                    errors.append(f"{field}.license_files must be a nonempty array")
                    continue
                seen_license_paths: set[str] = set()
                for license_index, license_file in enumerate(license_files):
                    license_field = f"{field}.license_files[{license_index}]"
                    if isinstance(license_file, str) and license_file in seen_license_paths:
                        errors.append(f"{license_field} duplicates a license path")
                        continue
                    if isinstance(license_file, str):
                        seen_license_paths.add(license_file)
                    license_rel = _relative_path(license_file, license_field, errors)
                    if license_rel is not None:
                        _contained_file(root_resolved, license_rel, license_field, errors)

    patches = manifest.get("patches")
    if not isinstance(patches, list):
        errors.append("patches must be an ordered array")
    else:
        report["patch_count"] = len(patches)
        seen_patch_paths: set[str] = set()
        seen_patch_files: set[Path] = set()
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
                resolved_patch = _contained_file(root_resolved, patch_rel, f"{field}.path", errors)
                if resolved_patch is not None:
                    if resolved_patch in seen_patch_files and not duplicate_path:
                        errors.append(f"duplicate patch path: {patch_path_value}")
                    seen_patch_files.add(resolved_patch)
                    _hash_matches(resolved_patch, patch.get("sha256"), f"{field}.sha256", errors)

    gates = manifest.get("gates", {})
    if not isinstance(gates, dict):
        errors.append("gates must be an object of gate names to not_run")
    else:
        for name, state in gates.items():
            if name not in GATE_NAMES:
                errors.append(f"unknown gate name: {name}")
            elif state != "not_run":
                errors.append(f"gate {name} must remain not_run in the input manifest")

    report["status"] = "valid" if not errors else "invalid"
    return report


def _load_json(path: Path) -> Any:
    try:
        return json.loads(
            path.read_text(encoding="utf-8"),
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

    try:
        root = args.root.resolve(strict=True)
        manifest_path = args.manifest.resolve(strict=True)
        try:
            manifest_path.relative_to(root)
        except ValueError as error:
            raise ManifestError("manifest file must resolve inside root") from error
        if not root.is_dir() or not manifest_path.is_file():
            raise ManifestError("root must be a directory containing a regular manifest file")
        manifest = _load_json(manifest_path)
        report = validate(manifest, root)
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
    print(json.dumps(report, sort_keys=True))
    return 0 if report["status"] == "valid" else 1


if __name__ == "__main__":
    raise SystemExit(main())
