#!/usr/bin/env python3
"""Install hash-verified GPUI Kit 0.9.1 sources and owned mirrors."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import sys
import tempfile
from pathlib import Path

from prepare_gpui_kit_09_fixture import (
    DEFAULT_MANIFEST,
    FixtureError,
    extract_verified_crate,
    load_manifest,
    verify_archive_file,
)


OWNED = {
    "gpui-pre": "crates/rustui-gpui",
    "gpui-pre-platform": "crates/rustui-gpui-platform",
    "gpui-pre-macos": "crates/rustui-gpui-macos",
    "gpui-pre-apple": "crates/rustui-gpui-apple",
    "gpui-pre-linux": "crates/rustui-gpui-linux",
    "gpui-pre-windows": "crates/rustui-gpui-windows",
    "gpui-pre-web": "crates/rustui-gpui-web",
    "gpui-pre-wgpu": "crates/rustui-gpui-wgpu",
    "gpui-base": "crates/rustui-gpui-base",
    "gpui-component": "crates/rustui-gpui-component",
    "gpui-kit-assets": "crates/rustui-gpui-kit-assets",
}
REPLACED_GROUPS = {"gpui-kit", "gpui-pre"}
ADDED_PACKAGES = {
    "icu_collections",
    "icu_normalizer",
    "icu_normalizer_data",
    "icu_properties",
    "icu_properties_data",
    "icu_segmenter",
    "icu_segmenter_data",
    "objc2-uniform-type-identifiers",
}


def archive_existing(path: Path, root: Path, archive_root: Path) -> None:
    """Preserve the current pre-reset file image before replacing it."""
    try:
        relative = path.relative_to(root)
    except ValueError as error:
        raise FixtureError(f"cannot archive path outside repository: {path}") from error
    destination = archive_root / relative
    if destination.exists():
        return
    destination.parent.mkdir(parents=True, exist_ok=True)
    if path.is_dir():
        shutil.copytree(path, destination)
    elif path.is_file():
        shutil.copy2(path, destination)


def install(fixture: Path, manifest_path: Path, root: Path, archive_root: Path) -> tuple[int, int]:
    manifest = load_manifest(manifest_path)
    selected = [
        source
        for source in manifest["sources"]
        if source.get("group") in REPLACED_GROUPS or source["name"] in ADDED_PACKAGES
    ]
    by_name = {source["name"]: source for source in selected}
    if not ADDED_PACKAGES.issubset(by_name):
        raise FixtureError("source manifest is missing one or more pinned new dependencies")

    vendor = root / "vendor"
    archive_roots = {source["name"] for source in selected if source.get("group") in REPLACED_GROUPS}
    for package in sorted(archive_roots):
        source = by_name[package]
        destination = vendor / f"{package}-{source['version']}"
        expected_archive_hash = source["archive_sha256"]
        for old_path in sorted(vendor.glob(f"{package}-*")):
            checksum_path = old_path / ".cargo-checksum.json"
            if old_path == destination and checksum_path.is_file():
                checksum = json.loads(checksum_path.read_text(encoding="utf-8"))
                if checksum.get("package") == expected_archive_hash:
                    # Preserve any approved patch hunks on an already installed source.
                    continue
            archive_existing(old_path, root, archive_root)

    verified = 0
    with tempfile.TemporaryDirectory(prefix="rustui-kit09-install-verify-") as temporary:
        temporary_root = Path(temporary)
        extracted_sources: dict[str, Path] = {}
        for source in selected:
            package, version = source["name"], source["version"]
            archive = fixture / "archives" / f"{package}-{version}.crate"
            extracted = fixture / "crates" / f"{package}-{version}"
            verify_archive_file(archive, source["archive_sha256"])
            if not extracted.is_dir():
                raise FixtureError(f"verified extraction is missing: {extracted}")
            checksum_file = extracted / ".cargo-checksum.json"
            if not checksum_file.is_file():
                raise FixtureError(f"Cargo checksum metadata is missing: {checksum_file}")
            checksum = json.loads(checksum_file.read_text(encoding="utf-8"))
            if checksum.get("package") != source["archive_sha256"]:
                raise FixtureError(f"extracted archive hash mismatch: {package}-{version}")
            # Independently re-extract to validate every file against the pinned archive.
            check_dir = temporary_root / f"{package}-{version}"
            extract_verified_crate(archive, source, check_dir)
            if tree_hashes(check_dir) != tree_hashes(extracted):
                raise FixtureError(f"verified extraction differs from pinned archive: {package}-{version}")
            extracted_sources[package] = extracted
            verified += 1

    # Replace only packages that belong to the selected upstream snapshots.
    for package in sorted(archive_roots):
        source = by_name[package]
        destination = vendor / f"{package}-{source['version']}"
        checksum_path = destination / ".cargo-checksum.json"
        if destination.is_dir() and checksum_path.is_file():
            checksum = json.loads(checksum_path.read_text(encoding="utf-8"))
            if checksum.get("package") == source["archive_sha256"]:
                continue
        for old_path in vendor.glob(f"{package}-*"):
            if old_path.is_dir():
                shutil.rmtree(old_path)
        shutil.copytree(extracted_sources[package], destination)

    # Add only the three new dependency package names identified by the lock-graph audit.
    for package in sorted(ADDED_PACKAGES):
        source = by_name[package]
        destination = vendor / f"{package}-{source['version']}"
        if not destination.exists():
            shutil.copytree(extracted_sources[package], destination)

    # Refresh source files only; owned manifests are reviewed/aligned separately.
    mirrors = 0
    for package, relative in OWNED.items():
        source = by_name.get(package)
        if source is None:
            continue
        vendor_source = vendor / f"{package}-{source['version']}"
        owned = root / relative
        if not vendor_source.is_dir() or not owned.is_dir():
            raise FixtureError(f"source or owned mirror missing: {vendor_source} -> {owned}")
        archive_existing(owned, root, archive_root)
        for child in list(owned.iterdir()):
            if child.name == "Cargo.toml":
                continue
            if child.is_dir():
                shutil.rmtree(child)
            else:
                child.unlink()
        for child in vendor_source.iterdir():
            if child.name == "Cargo.toml":
                continue
            target = owned / child.name
            if child.is_dir():
                shutil.copytree(child, target)
            else:
                shutil.copy2(child, target)
        mirrors += 1
    return verified, mirrors


def tree_hashes(root: Path) -> dict[str, str]:
    return {
        path.relative_to(root).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture-dir", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--apply", action="store_true", help="replace vendor GPUI sources and refresh owned source files")
    args = parser.parse_args()
    if not args.apply:
        print("pass --apply to install the pinned Kit 0.9.1 sources", file=sys.stderr)
        return 2
    root = Path(__file__).resolve().parents[1]
    try:
        verified, mirrors = install(
            args.fixture_dir,
            args.manifest,
            root,
            root / "archive/pre-reset-2026-11",
        )
    except (FixtureError, OSError, json.JSONDecodeError) as error:
        print(f"GPUI Kit source installation failed: {error}", file=sys.stderr)
        return 1
    print(f"installed hash-verified sources={verified}; owned source mirrors refreshed={mirrors}")
    print("added third-party crates: " + ", ".join(sorted(ADDED_PACKAGES)))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
