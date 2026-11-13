#!/usr/bin/env python3
"""Verify RustUI-owned GPUI source copies match their pinned vendor snapshots."""

from __future__ import annotations

import os
import sys
from pathlib import Path


OWNED_VENDOR_COPIES = {
    "crates/rustui-gpui": "vendor/gpui-pre-0.3.12",
    "crates/rustui-gpui-platform": "vendor/gpui-pre-platform-0.3.12",
    "crates/rustui-gpui-macos": "vendor/gpui-pre-macos-0.3.12",
    "crates/rustui-gpui-apple": "vendor/gpui-pre-apple-0.3.12",
    "crates/rustui-gpui-base": "vendor/gpui-base-0.9.1",
    "crates/rustui-gpui-component": "vendor/gpui-component-0.9.1",
    "crates/rustui-block": "vendor/block-0.1.6",
    "crates/rustui-cocoa": "vendor/cocoa-0.26.1",
    "crates/rustui-cocoa-foundation": "vendor/cocoa-foundation-0.2.1",
    "crates/rustui-metal": "vendor/metal-0.33.0",
    "crates/rustui-objc-foundation": "vendor/objc-foundation-0.1.1",
    "crates/rustui-gpui-kit-assets": "vendor/gpui-kit-assets-0.9.1",
    "crates/rustui-gpui-linux": "vendor/gpui-pre-linux-0.3.12",
    "crates/rustui-gpui-web": "vendor/gpui-pre-web-0.3.12",
    "crates/rustui-gpui-wgpu": "vendor/gpui-pre-wgpu-0.3.12",
    "crates/rustui-gpui-windows": "vendor/gpui-pre-windows-0.3.12",
}
IGNORED_RELATIVE_PATHS = {"Cargo.toml"}
OWNED_ONLY_IGNORED_PATHS = {
    # This test-only standalone lock is intentionally resolved offline for the
    # macOS crate and is not the published/vendor snapshot's source lock.
    "crates/rustui-gpui-macos": {"Cargo.lock"},
}


def snapshot(
    root: Path, ignored_relative_paths: set[str] | None = None
) -> dict[str, tuple[str, bytes | str | None]]:
    entries: dict[str, tuple[str, bytes | str | None]] = {}
    ignored = IGNORED_RELATIVE_PATHS | (ignored_relative_paths or set())
    for path in root.rglob("*"):
        relative = path.relative_to(root).as_posix()
        if relative in ignored:
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


def compare_trees(
    owned: Path,
    vendor: Path,
    ignored_relative_paths: set[str] | None = None,
) -> tuple[list[str], int]:
    """Return mismatched relative paths and the number of equal files checked."""
    if not owned.is_dir():
        return [f"owned copy is missing: {owned}"], 0
    if not vendor.is_dir():
        return [f"vendor source is missing: {vendor}"], 0

    owned_snapshot = snapshot(owned, ignored_relative_paths)
    vendor_snapshot = snapshot(vendor, ignored_relative_paths)
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
        mismatches, file_count = compare_trees(
            owned,
            vendor,
            OWNED_ONLY_IGNORED_PATHS.get(owned_relative),
        )
        if mismatches:
            preview = ", ".join(mismatches[:12])
            extra = len(mismatches) - min(len(mismatches), 12)
            if extra:
                preview += f", and {extra} more"
            errors.append(f"{owned_relative} differs from {vendor_relative}: {preview}")
        else:
            exceptions = "Cargo.toml"
            if OWNED_ONLY_IGNORED_PATHS.get(owned_relative):
                exceptions += " and standalone test Cargo.lock"
            print(
                f"PASS: {owned_relative} matches {vendor_relative} byte-for-byte "
                f"except {exceptions} ({file_count} files checked)"
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
