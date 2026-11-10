#!/usr/bin/env python3
"""Copy explicitly named, lockfile-pinned registry crates into vendor/."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def install(crate_ids: list[str], registry_root: Path, source_lock: Path) -> None:
    lock = tomllib.loads(source_lock.read_text(encoding="utf-8"))
    packages = {
        f"{package['name']}-{package['version']}": package
        for package in lock["package"]
        if package.get("source", "").startswith("registry+")
    }
    for crate_id in crate_ids:
        package = packages.get(crate_id)
        if package is None:
            raise RuntimeError(f"crate is not locked from a registry: {crate_id}")
        source = next(registry_root.glob(f"*/{crate_id}"), None)
        if source is None:
            raise RuntimeError(f"cached registry source is missing: {crate_id}")
        checksum_path = source / ".cargo-checksum.json"
        if checksum_path.is_file():
            checksum = json.loads(checksum_path.read_text(encoding="utf-8"))
        else:
            archive = next((Path.home() / ".cargo/registry/cache").glob(f"*/{crate_id}.crate"), None)
            if archive is None or hashlib.sha256(archive.read_bytes()).hexdigest() != package.get("checksum"):
                raise RuntimeError(f"registry archive does not match Cargo.lock: {crate_id}")
            checksum = {
                "files": {
                    path.relative_to(source).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
                    for path in sorted(source.rglob("*"))
                    if path.is_file() and path.name != ".cargo-checksum.json"
                },
                "package": package["checksum"],
            }
        if checksum.get("package") != package.get("checksum"):
            raise RuntimeError(f"registry checksum does not match Cargo.lock: {crate_id}")
        target = ROOT / "vendor" / crate_id
        if target.exists():
            raise RuntimeError(f"vendor destination already exists: {target}")
        shutil.copytree(source, target)
        if not (target / ".cargo-checksum.json").is_file():
            (target / ".cargo-checksum.json").write_text(json.dumps(checksum, sort_keys=True, separators=(",", ":")) + "\n", encoding="utf-8")
        print(f"INSTALLED {crate_id} checksum={checksum['package']}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("crate", nargs="+", help="locked name-version, for example block-buffer-0.12.1")
    parser.add_argument("--registry-root", type=Path, default=Path.home() / ".cargo/registry/src")
    parser.add_argument("--source-lock", type=Path, default=ROOT / "Cargo.lock")
    args = parser.parse_args()
    try:
        install(args.crate, args.registry_root, args.source_lock)
    except (OSError, RuntimeError, json.JSONDecodeError) as error:
        print(f"locked vendor install failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
