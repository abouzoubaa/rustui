#!/usr/bin/env python3
"""Verify RustUI-owned GPUI source copies match their pinned vendor snapshots."""

from __future__ import annotations

import os
import sys
from pathlib import Path


OWNED_VENDOR_COPIES = {
    "crates/rustui-gpui": "vendor/gpui-pre-0.3.9",
    "crates/rustui-gpui-platform": "vendor/gpui-pre-platform-0.3.9",
    "crates/rustui-gpui-macos": "vendor/gpui-pre-macos-0.3.9",
    "crates/rustui-gpui-apple": "vendor/gpui-pre-apple-0.3.9",
    "crates/rustui-gpui-base": "vendor/gpui-base-0.7.3",
    "crates/rustui-gpui-component": "vendor/gpui-component-0.7.3",
    "crates/rustui-block": "vendor/block-0.1.6",
}
IGNORED_RELATIVE_PATHS = {"Cargo.toml"}


def snapshot(root: Path) -> dict[str, tuple[str, bytes | str | None]]:
    entries: dict[str, tuple[str, bytes | str | None]] = {}
    for path in root.rglob("*"):
        relative = path.relative_to(root).as_posix()
        if relative in IGNORED_RELATIVE_PATHS:
            continue
        if path.is_symlink():
            entries[relative] = ("symlink", os.readlink(path))
        elif path.is_dir():
            entries[relative] = ("directory", None)
        elif path.is_file():
            entries[relative] = ("file", path.read_bytes())
        else:
            entries[relative] = ("other", None)
    return entries


def compare_trees(owned: Path, vendor: Path) -> tuple[list[str], int]:
    """Return mismatched relative paths and the number of equal files checked."""
    if not owned.is_dir():
        return [f"owned copy is missing: {owned}"], 0
    if not vendor.is_dir():
        return [f"vendor source is missing: {vendor}"], 0

    owned_snapshot = snapshot(owned)
    vendor_snapshot = snapshot(vendor)
    mismatches = [
        path
        for path in sorted(owned_snapshot.keys() | vendor_snapshot.keys())
        if owned_snapshot.get(path) != vendor_snapshot.get(path)
    ]
    file_count = sum(kind == "file" for kind, _ in vendor_snapshot.values())
    return mismatches, file_count


def verify_repository(root: Path) -> list[str]:
    errors: list[str] = []
    for owned_relative, vendor_relative in OWNED_VENDOR_COPIES.items():
        owned = root / owned_relative
        vendor = root / vendor_relative
        mismatches, file_count = compare_trees(owned, vendor)
        if mismatches:
            preview = ", ".join(mismatches[:12])
            extra = len(mismatches) - min(len(mismatches), 12)
            if extra:
                preview += f", and {extra} more"
            errors.append(f"{owned_relative} differs from {vendor_relative}: {preview}")
        else:
            print(
                f"PASS: {owned_relative} matches {vendor_relative} byte-for-byte "
                f"except Cargo.toml ({file_count} files checked)"
            )
    return errors


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    errors = verify_repository(root)
    if errors:
        for error in errors:
            print(f"FAIL: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
