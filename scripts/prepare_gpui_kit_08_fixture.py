#!/usr/bin/env python3
"""Download and inspect the pinned GPUI Kit 0.8.1 source fixture."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
import tarfile
import tempfile
import urllib.request
from collections import defaultdict
from pathlib import Path, PurePosixPath
from typing import Any


ROOT = Path(__file__).resolve().parent.parent
DEFAULT_MANIFEST = ROOT / "scripts/data/gpui-kit-0.8.1-sources.json"
DEFAULT_PATCH_DIR = ROOT / "patches/approved"

OWNED_MIRRORS = {
    "rustui-gpui-platform": "gpui-pre-platform",
    "rustui-gpui-component": "gpui-component",
    "rustui-gpui-kit-assets": "gpui-kit-assets",
    "rustui-gpui-macos": "gpui-pre-macos",
    "rustui-gpui-apple": "gpui-pre-apple",
    "rustui-gpui-linux": "gpui-pre-linux",
    "rustui-gpui-windows": "gpui-pre-windows",
    "rustui-gpui-web": "gpui-pre-web",
    "rustui-gpui-wgpu": "gpui-pre-wgpu",
    "rustui-gpui-base": "gpui-base",
    "rustui-gpui": "gpui-pre",
}


class FixtureError(RuntimeError):
    """A pinned archive or source-file verification failed."""


class MissingArchive(FixtureError):
    """A required downloaded archive does not exist."""


class HashMismatch(FixtureError):
    """A downloaded archive differs from its pinned SHA-256."""


class MissingSourceFile(FixtureError):
    """A required source file is absent from its pinned crate archive."""


def verify_archive_file(path: Path, expected_sha256: str) -> str:
    if not path.is_file():
        raise MissingArchive(f"required crate archive is missing: {path}")
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    if digest != expected_sha256:
        raise HashMismatch(
            f"SHA-256 mismatch for {path.name}: expected {expected_sha256}, got {digest}"
        )
    return digest


def download_archive(source: dict[str, str], destination: Path) -> str:
    destination.parent.mkdir(parents=True, exist_ok=True)
    request = urllib.request.Request(
        source["archive_url"], headers={"User-Agent": "rustui-a2-1-source-harness"}
    )
    try:
        with urllib.request.urlopen(request, timeout=60) as response:
            data = response.read()
    except Exception as error:  # surfaced with crate identity by the caller
        raise FixtureError(f"download failed: {source['name']}-{source['version']}: {error}") from error
    destination.write_bytes(data)
    return verify_archive_file(destination, source["archive_sha256"])


def read_crate_file(archive_path: Path, package: str, version: str, relative_path: str) -> bytes:
    relative = PurePosixPath(relative_path)
    if relative.is_absolute() or ".." in relative.parts:
        raise FixtureError(f"unsafe crate source path: {relative_path}")
    member_name = f"{package}-{version}/{relative.as_posix()}"
    if not archive_path.is_file():
        raise MissingArchive(f"required crate archive is missing: {archive_path}")
    try:
        with tarfile.open(archive_path, "r:gz") as archive:
            try:
                member = archive.getmember(member_name)
            except KeyError as error:
                raise MissingSourceFile(
                    f"required source file is missing from {archive_path.name}: {relative_path}"
                ) from error
            if not member.isfile():
                raise MissingSourceFile(
                    f"required source path is not a regular file in {archive_path.name}: {relative_path}"
                )
            source = archive.extractfile(member)
            if source is None:
                raise MissingSourceFile(
                    f"cannot read source file from {archive_path.name}: {relative_path}"
                )
            return source.read()
    except tarfile.TarError as error:
        raise FixtureError(f"invalid crate archive {archive_path.name}: {error}") from error


def extract_verified_crate(
    archive_path: Path, source: dict[str, str], destination: Path
) -> str:
    """Extract a complete hash-verified crate and write Cargo vendor metadata."""
    verify_archive_file(archive_path, source["archive_sha256"])
    package, version = source["name"], source["version"]
    expected_root = f"{package}-{version}"
    destination.mkdir(parents=True, exist_ok=False)
    try:
        with tarfile.open(archive_path, "r:gz") as archive:
            for member in archive.getmembers():
                relative = PurePosixPath(member.name)
                if (
                    relative.is_absolute()
                    or ".." in relative.parts
                    or not relative.parts
                    or relative.parts[0] != expected_root
                    or member.issym()
                    or member.islnk()
                    or not (member.isdir() or member.isfile())
                ):
                    raise FixtureError(f"unsafe or unexpected archive entry: {member.name}")
                parts = relative.parts[1:]
                if not parts:
                    continue
                target = destination.joinpath(*parts)
                if member.isdir():
                    target.mkdir(parents=True, exist_ok=True)
                    continue
                target.parent.mkdir(parents=True, exist_ok=True)
                contents = archive.extractfile(member)
                if contents is None:
                    raise FixtureError(f"cannot read archive entry: {member.name}")
                with target.open("xb") as output:
                    output.write(contents.read())
                os.chmod(target, member.mode & 0o777)
    except (tarfile.TarError, OSError) as error:
        raise FixtureError(f"cannot extract {package}-{version}: {error}") from error

    files = {
        path.relative_to(destination).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
        for path in sorted(destination.rglob("*"))
        if path.is_file()
    }
    checksum = {"files": files, "package": source["archive_sha256"]}
    checksum_path = destination / ".cargo-checksum.json"
    checksum_path.write_text(
        json.dumps(checksum, sort_keys=True, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )
    return hashlib.sha256(checksum_path.read_bytes()).hexdigest()


def target_source_path(target: str, package_names: list[str]) -> tuple[str, str] | None:
    parts = PurePosixPath(target).parts
    if len(parts) < 3:
        return None
    if parts[0] == "vendor":
        directory = parts[1]
        matches = [name for name in package_names if directory.startswith(name + "-")]
        if not matches:
            if directory.startswith("gpui-") or directory.startswith("block-") or directory.startswith("cosmic-text-"):
                raise FixtureError(f"approved patch target is not pinned in the manifest: {target}")
            return None
        package = max(matches, key=len)
        return package, PurePosixPath(*parts[2:]).as_posix()
    if parts[0] == "crates":
        owned = OWNED_MIRRORS.get(parts[1]) if len(parts) > 2 else None
        if owned:
            return owned, PurePosixPath(*parts[2:]).as_posix()
    return None


def collect_patch_targets(patch_dir: Path, package_names: list[str]) -> dict[tuple[str, str], set[str]]:
    targets: dict[tuple[str, str], set[str]] = defaultdict(set)
    for patch_path in sorted(patch_dir.glob("*.patch")):
        for line in patch_path.read_text(encoding="utf-8").splitlines():
            if not line.startswith("+++ b/"):
                continue
            resolved = target_source_path(line[6:], package_names)
            if resolved:
                targets[resolved].add(patch_path.name)
    return targets


def known_unavailable(manifest: dict[str, Any]) -> dict[tuple[str, str], dict[str, Any]]:
    return {
        (entry["package"], entry["path"]): entry
        for entry in manifest.get("known_unavailable_targets", [])
    }


def build_fixture(manifest: dict[str, Any], fixture_dir: Path, patch_dir: Path) -> tuple[int, int, int]:
    sources = manifest["sources"]
    source_by_name = {source["name"]: source for source in sources}
    package_names = sorted(source_by_name, key=len, reverse=True)
    archives: dict[str, Path] = {}
    for source in sources:
        archive_path = fixture_dir / "archives" / f"{source['name']}-{source['version']}.crate"
        download_archive(source, archive_path)
        archives[source["name"]] = archive_path

    targets = collect_patch_targets(patch_dir, package_names)
    unavailable = known_unavailable(manifest)
    hashes = 0
    unresolved = 0
    for (package, relative_path), patch_names in sorted(targets.items()):
        source = source_by_name[package]
        excluded = unavailable.get((package, relative_path))
        if excluded:
            unresolved += 1
            print(
                f"UNAVAILABLE {package}/{relative_path} "
                f"patches={','.join(sorted(patch_names))} reason={excluded['reason']}"
            )
            continue
        content = read_crate_file(
            archives[package], package, source["version"], relative_path
        )
        digest = hashlib.sha256(content).hexdigest()
        target_path = fixture_dir / "sources" / f"{package}-{source['version']}" / relative_path
        target_path.parent.mkdir(parents=True, exist_ok=True)
        target_path.write_bytes(content)
        hashes += 1
        old_targets = ",".join(sorted(patch_names))
        print(f"{digest}  {package}/{relative_path}  patches={old_targets}")

    return len(sources), hashes, unresolved


def extract_crate_set(manifest: dict[str, Any], fixture_dir: Path) -> int:
    count = 0
    for source in manifest["sources"]:
        archive_path = fixture_dir / "archives" / f"{source['name']}-{source['version']}.crate"
        destination = fixture_dir / "crates" / f"{source['name']}-{source['version']}"
        checksum_hash = extract_verified_crate(archive_path, source, destination)
        print(
            f"EXTRACTED {source['name']}-{source['version']} "
            f"checksum-file-sha256={checksum_hash}"
        )
        count += 1
    return count


def load_manifest(path: Path) -> dict[str, Any]:
    try:
        manifest = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise FixtureError(f"cannot read source manifest {path}: {error}") from error
    if manifest.get("schema_version") != 1:
        raise FixtureError(f"unsupported source manifest schema: {manifest.get('schema_version')}")
    return manifest


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--patch-dir", type=Path, default=DEFAULT_PATCH_DIR)
    parser.add_argument(
        "--fixture-dir",
        type=Path,
        help="retain fixture files in a new directory; omitted means a disposable temporary directory",
    )
    parser.add_argument(
        "--extract-crates",
        action="store_true",
        help="also extract every verified archive to fixture/crates with Cargo checksum metadata",
    )
    args = parser.parse_args(argv)
    try:
        manifest = load_manifest(args.manifest)
        if args.fixture_dir:
            if args.fixture_dir.exists():
                raise FixtureError(f"fixture directory already exists: {args.fixture_dir}")
            args.fixture_dir.mkdir(parents=True, exist_ok=False)
            archive_count, target_count, unresolved_count = build_fixture(
                manifest, args.fixture_dir, args.patch_dir
            )
            extracted_count = extract_crate_set(manifest, args.fixture_dir) if args.extract_crates else 0
            print(
                f"verified crate archives={archive_count}; "
                f"target file hashes={target_count}; "
                f"known unavailable target paths={unresolved_count}; "
                f"full crates extracted={extracted_count}; "
                f"fixture={args.fixture_dir}"
            )
        else:
            with tempfile.TemporaryDirectory(prefix="rustui-gpui-kit-0.8.1-") as temporary:
                archive_count, target_count, unresolved_count = build_fixture(
                    manifest, Path(temporary), args.patch_dir
                )
                extracted_count = extract_crate_set(manifest, Path(temporary)) if args.extract_crates else 0
                print(
                    f"verified crate archives={archive_count}; "
                    f"target file hashes={target_count}; "
                    f"known unavailable target paths={unresolved_count}; "
                    f"full crates extracted={extracted_count}; "
                    "fixture=temporary (removed on exit)"
                )
    except FixtureError as error:
        print(f"source fixture check failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
