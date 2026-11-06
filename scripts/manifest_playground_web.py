#!/usr/bin/env python3
"""Record source and artifact hashes for a RustUI Playground Web bundle."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys


SOURCE_INPUTS = (
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    ".cargo/config.toml",
    "scripts/cargo-headstart",
    "scripts/build-playground-web.sh",
    "scripts/manifest_playground_web.py",
    "scripts/collect_dependency_notices.py",
    "scripts/package_release_notices.py",
)
SOURCE_ROOTS = (
    "assets", "crates", "apps", "themes", "locales",
    "vendor/gpui-pre-0.3.9", "vendor/gpui-base-0.7.3",
    "vendor/gpui-component-0.7.3", "vendor/gpui-pre-web-0.3.9",
    "vendor/gpui-kit-assets-0.7.3",
    "docs/release/dependency-notices",
)
IGNORED_DIRS = {".git", "node_modules", "target", "graft", ".trail", "__pycache__"}


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def record_file(root: Path, relative: str) -> dict[str, object]:
    path = root / relative
    record: dict[str, object] = {"path": relative}
    try:
        if path.is_symlink():
            raise ValueError(f"manifest input must not be a symlink: {relative}")
        if not path.is_file():
            record["status"] = "missing"
            return record
        record["sha256"] = sha256_file(path)
        record["size_bytes"] = path.stat().st_size
        record["status"] = "captured"
    except OSError as error:
        record["status"] = "unreadable"
        record["error"] = str(error)
    return record


def walk_files(root: Path):
    def reject_walk_error(error: OSError):
        raise error

    for current, directories, filenames in os.walk(root, followlinks=False, onerror=reject_walk_error):
        directories[:] = sorted(name for name in directories if name not in IGNORED_DIRS)
        for name in directories:
            if (Path(current) / name).is_symlink():
                raise ValueError(f"manifest directory must not be a symlink: {Path(current) / name}")
        for name in sorted(filenames):
            yield Path(current) / name


def source_paths(root: Path) -> list[str]:
    paths = set(SOURCE_INPUTS)
    for source_root in SOURCE_ROOTS:
        base = root / source_root
        if not base.is_dir() or base.is_symlink():
            raise ValueError(f"missing or symlinked manifest source directory: {source_root}")
        for path in walk_files(base):
            paths.add(path.relative_to(root).as_posix())
    return sorted(paths)


def source_record(root: Path) -> dict[str, object]:
    files = [record_file(root, relative) for relative in source_paths(root)]
    require_captured(files)
    digest = hashlib.sha256()
    for record in files:
        digest.update(str(record["path"]).encode("utf-8"))
        digest.update(b"\0")
        digest.update(str(record.get("status", "")).encode("ascii"))
        digest.update(b"\0")
        digest.update(str(record.get("sha256", "")).encode("ascii"))
        digest.update(b"\n")

    try:
        commit = subprocess.run(
            ["git", "-C", str(root), "rev-parse", "HEAD"],
            check=True,
            capture_output=True,
            text=True,
            timeout=5,
        ).stdout.strip()
    except (OSError, subprocess.SubprocessError):
        commit = None
    return {"git_commit": commit, "sha256": digest.hexdigest(), "files": files}


def bundle_files(bundle: Path) -> list[dict[str, object]]:
    files: list[dict[str, object]] = []
    for path in walk_files(bundle):
        relative = path.relative_to(bundle).as_posix()
        if relative != "bundle-manifest.json":
            files.append(record_file(bundle, relative))
    require_captured(files)
    return files


def require_captured(files: list[dict[str, object]]) -> None:
    for record in files:
        if record["status"] != "captured":
            raise ValueError(f"manifest input could not be captured: {record['path']} ({record['status']})")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--bundle-dir", type=Path)
    parser.add_argument("--source-baseline", type=Path)
    args = parser.parse_args()

    root = args.root.expanduser().resolve()
    if not root.is_dir():
        parser.error(f"workspace is not a directory: {root}")
    try:
        source = source_record(root)
    except (OSError, ValueError) as error:
        parser.error(str(error))
    if args.source_baseline is not None:
        baseline_path = args.source_baseline.expanduser().resolve()
        try:
            baseline = json.loads(baseline_path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as error:
            parser.error(f"cannot read source baseline {baseline_path}: {error}")
        previous_source = baseline.get("source", {})
        if (
            source["git_commit"] != previous_source.get("git_commit")
            or source["sha256"] != previous_source.get("sha256")
        ):
            print("RustUI sources changed during the Web bundle build; preserving the existing Web bundle.", file=sys.stderr)
            return 1

    document: dict[str, object] = {"schema_version": 1, "source": source}
    if args.bundle_dir is not None:
        bundle = args.bundle_dir.expanduser().resolve()
        if not bundle.is_dir():
            parser.error(f"bundle directory is not a directory: {bundle}")
        try:
            document["bundle"] = {"files": bundle_files(bundle)}
        except (OSError, ValueError) as error:
            parser.error(str(error))

    destination = args.output.expanduser().resolve()
    destination.parent.mkdir(parents=True, exist_ok=True)
    with destination.open("w", encoding="utf-8") as output:
        json.dump(document, output, indent=2, sort_keys=True)
        output.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
