#!/usr/bin/env python3
"""Exact GPUI macOS focused-element patch; keeps the pinned crate revision."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import tempfile
import tomllib
from pathlib import Path

MANIFEST = Path("patches/approved/gpui-macos-focused-element-forwarder-v1.json")
MANIFEST_SHA256 = "b3e5bbd2d2e394889e6fa217be7b252f542ff008a128c1bf6b6dd9c436735f34"


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def hash_file(path: Path) -> str:
    return digest(path.read_bytes())


def locked_package(lock: dict, name: str, version: str) -> dict:
    matches = [p for p in lock["package"] if p["name"] == name and p["version"] == version]
    if len(matches) != 1:
        raise ValueError(f"Cargo.lock must contain exactly one {name} {version} package")
    return matches[0]


def verify_provenance(root: Path, artifacts: Path, meta: dict) -> None:
    gpui = meta["gpui"]
    package = root / "vendor/gpui-pre-macos-0.3.9"
    gpui_document = tomllib.loads((package / "Cargo.toml").read_text())
    gpui_meta = gpui_document["package"]
    if (gpui_meta["name"], gpui_meta["version"], gpui_meta["license"]) != (
        gpui["name"], gpui["version"], gpui["license"]
    ):
        raise ValueError("pinned GPUI package identity/license changed")
    if gpui_meta["metadata"]["gpui-pre"]["zed-rev"] != gpui["zed_revision"]:
        raise ValueError("pinned GPUI Zed revision changed")
    dependency = None
    for target in gpui_document.get("target", {}).values():
        dependency = target.get("dependencies", {}).get("accesskit_macos")
        if dependency is not None:
            break
    if dependency is None or dependency.get("version") != gpui["accesskit_dependency_requirement"]:
        raise ValueError("pinned GPUI AccessKit dependency changed")
    checksum = json.loads((package / ".cargo-checksum.json").read_text())
    if checksum["package"] != gpui["cargo_checksum"]:
        raise ValueError("pinned GPUI crate checksum changed")
    if hash_file(package / ".cargo-checksum.json") != gpui["checksum_file_sha256"]:
        raise ValueError("GPUI .cargo-checksum.json changed")
    if checksum["files"].get("src/window.rs") != meta["pre"]:
        raise ValueError("GPUI checksum manifest does not identify the expected PRE source")
    if hash_file(root / gpui["license_path"]) != gpui["license_sha256"]:
        raise ValueError("pinned GPUI Apache license changed")

    root_cargo = tomllib.loads((root / "Cargo.toml").read_text())
    selected_gpui = root_cargo.get("patch", {}).get("crates-io", {}).get(gpui["name"])
    if selected_gpui != {"path": gpui["cargo_path"]}:
        raise ValueError("Cargo.toml must select the pinned local GPUI macOS package")
    lock = tomllib.loads((root / "Cargo.lock").read_text())
    gpui_lock = locked_package(lock, gpui["name"], gpui["version"])
    if "source" in gpui_lock or "checksum" in gpui_lock:
        raise ValueError("Cargo.lock still selects registry GPUI macOS; local path patch is required")
    access = meta["accesskit_macos"]
    access_lock = locked_package(lock, access["name"], access["version"])
    if access_lock["checksum"] != access["cargo_checksum"]:
        raise ValueError("Cargo.lock AccessKit checksum changed")

    source = root / access["vendored_source"]
    checksum_path = source / ".cargo-checksum.json"
    access_checksum = json.loads(checksum_path.read_text())
    if hash_file(checksum_path) != access["checksum_file_sha256"]:
        raise ValueError("vendored AccessKit .cargo-checksum.json changed")
    if access_checksum.get("package") != access["cargo_checksum"]:
        raise ValueError("vendored AccessKit package archive checksum changed")
    if access_lock["checksum"] != access_checksum["package"]:
        raise ValueError("vendored AccessKit checksum does not match Cargo.lock")
    access_meta = tomllib.loads((source / "Cargo.toml").read_text())["package"]
    vcs_path = source / ".cargo_vcs_info.json"
    vcs = json.loads(vcs_path.read_text())
    patch_source = source / "src/patch.rs"
    if (access_meta["name"], access_meta["version"], access_meta["license"], access_meta["repository"]) != (
        access["name"], access["version"], access["license"], access["repository"]
    ):
        raise ValueError("vendored AccessKit package identity/license changed")
    if vcs["git"]["sha1"] != access["revision"]:
        raise ValueError("vendored AccessKit revision changed")
    if hash_file(source / "Cargo.toml") != access["crate_manifest_sha256"]:
        raise ValueError("vendored AccessKit Cargo.toml changed")
    if hash_file(vcs_path) != access["vcs_info_sha256"]:
        raise ValueError("vendored AccessKit VCS metadata changed")
    if hash_file(patch_source) != access["forwarder_source_sha256"]:
        raise ValueError("vendored AccessKit forwarder implementation changed")
    for relative in ("Cargo.toml", ".cargo_vcs_info.json", "src/patch.rs"):
        if access_checksum["files"].get(relative) != hash_file(source / relative):
            raise ValueError(f"vendored AccessKit file checksum mismatch: {relative}")
    patch_text = patch_source.read_text()
    if "pub unsafe fn add_focus_forwarder_to_window_class(class_name: &str)" not in patch_text:
        raise ValueError("AccessKit public focus-forwarder helper is unavailable")
    if "accessibilityFocusedUIElement" not in patch_text or "code for this crate is never unloaded" not in patch_text:
        raise ValueError("AccessKit focus-forwarder behavior or safety contract changed")

    for license_file in access["license_files"]:
        path = artifacts / license_file["path"]
        if hash_file(path) != license_file["sha256"]:
            raise ValueError(f"AccessKit license provenance mismatch: {path.name}")


def apply_in_stage(stage: Path, source_rel: str, patch: Path, reverse: bool) -> bytes:
    staged = stage / source_rel
    staged.parent.mkdir(parents=True, exist_ok=True)
    original = staged.read_bytes()
    command = ["git", "apply", "--whitespace=nowarn"]
    if reverse:
        command.append("--reverse")
    subprocess.run(command + ["--check", str(patch)], cwd=stage, check=True, capture_output=True)
    subprocess.run(command + [str(patch)], cwd=stage, check=True, capture_output=True)
    return staged.read_bytes()


def atomic_write(path: Path, data: bytes, mode: int) -> None:
    fd, name = tempfile.mkstemp(prefix=".gpui-focused-element-", dir=path.parent)
    try:
        os.fchmod(fd, mode)
        with os.fdopen(fd, "wb") as output:
            output.write(data)
            output.flush()
            os.fsync(output.fileno())
        os.replace(name, path)
    finally:
        Path(name).unlink(missing_ok=True)


def install_with_rollback(source: Path, before: bytes, after: bytes, mode: int) -> None:
    try:
        atomic_write(source, after, mode)
        if source.read_bytes() != after:
            raise ValueError("installed source does not match expected image")
    except BaseException as install_error:
        try:
            current = source.read_bytes()
            if current != before:
                atomic_write(source, before, mode)
            if source.read_bytes() != before:
                raise ValueError("rollback verification failed")
        except BaseException as rollback_error:
            raise RuntimeError(f"install failed ({install_error}); rollback also failed ({rollback_error})") from install_error
        raise


def run(root: Path, artifacts: Path, mode: str) -> None:
    root = root.resolve(strict=True)
    artifacts = artifacts.resolve(strict=True)
    manifest_bytes = (artifacts / MANIFEST).read_bytes()
    if digest(manifest_bytes) != MANIFEST_SHA256:
        raise ValueError("metadata hash mismatch")
    meta = json.loads(manifest_bytes)
    patch = artifacts / meta["patch"]
    if hash_file(patch) != meta["patch_sha256"]:
        raise ValueError("patch hash mismatch")
    verify_provenance(root, artifacts, meta)

    source = root / meta["source"]
    if source.is_symlink() or not source.is_file():
        raise ValueError("installed GPUI source is missing or symlinked")
    before = source.read_bytes()
    current = digest(before)
    if current not in (meta["pre"], meta["post"]):
        raise ValueError("unknown or partial GPUI macOS source")
    if mode == "--verify" and current != meta["post"]:
        raise ValueError("exact POST required")

    reverse = current == meta["post"]
    with tempfile.TemporaryDirectory(prefix="gpui-focused-element-check-") as folder:
        stage = Path(folder)
        staged = stage / meta["source"]
        staged.parent.mkdir(parents=True)
        staged.write_bytes(before)
        changed = apply_in_stage(stage, meta["source"], patch, reverse=reverse)
        expected = meta["pre" if reverse else "post"]
        if digest(changed) != expected:
            raise ValueError("exact patch replay hash mismatch")
        roundtrip = apply_in_stage(stage, meta["source"], patch, reverse=not reverse)
        if roundtrip != before:
            raise ValueError("reverse replay mismatch")

    if mode in ("--apply", "--reverse"):
        if (mode == "--apply") == reverse:
            raise ValueError("source already at requested image")
        if source.read_bytes() != before:
            raise ValueError("source changed during staging")
        after = changed
        install_with_rollback(source, before, after, source.stat().st_mode & 0o777)
    print("GPUI macOS focused-element exact source, provenance, and replay checks passed")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--artifacts", type=Path)
    modes = parser.add_mutually_exclusive_group(required=True)
    for mode in ("--check", "--apply", "--verify", "--reverse"):
        modes.add_argument(mode, action="store_true")
    args = parser.parse_args()
    mode = next(m for m in ("--check", "--apply", "--verify", "--reverse") if getattr(args, m[2:]))
    try:
        run(args.root, args.artifacts or args.root, mode)
    except (ValueError, OSError, subprocess.CalledProcessError, RuntimeError) as error:
        parser.exit(1, f"error: {error}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
