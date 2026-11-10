#!/usr/bin/env python3
"""Refresh the RustUI-owned GPUI source mirrors from their verified vendor trees."""

from __future__ import annotations

import json
import shutil
import sys
from pathlib import Path

from prepare_gpui_kit_08_fixture import DEFAULT_MANIFEST, load_manifest

ROOT = Path(__file__).resolve().parents[1]
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
    "block": "crates/rustui-block",
}
VERSION_REWRITES = (
    ('version = "=0.3.9"', 'version = "=0.3.10"'),
    ('version = "=0.7.3"', 'version = "=0.8.1"'),
    ('version = "0.3.9"', 'version = "0.3.10"'),
    ('version = "0.7.3"', 'version = "0.8.1"'),
    ('version = "0.7.1"', 'version = "0.8.1"'),
)


def refresh(manifest_path: Path) -> int:
    manifest = load_manifest(manifest_path)
    count = 0
    for source in manifest["sources"]:
        package = source["name"]
        relative = OWNED.get(package)
        if relative is None:
            continue
        source_dir = ROOT / "vendor" / f"{package}-{source['version']}"
        destination = ROOT / relative
        if not source_dir.is_dir() or not destination.is_dir():
            raise RuntimeError(f"source or owned mirror missing: {source_dir} -> {destination}")
        for child in destination.iterdir():
            if child.name == "Cargo.toml":
                continue
            if child.is_dir():
                shutil.rmtree(child)
            else:
                child.unlink()
        for child in source_dir.iterdir():
            if child.name == "Cargo.toml":
                continue
            target = destination / child.name
            if child.is_dir():
                shutil.copytree(child, target)
            else:
                shutil.copy2(child, target)
        cargo = destination / "Cargo.toml"
        content = cargo.read_text(encoding="utf-8")
        for old, new in VERSION_REWRITES:
            content = content.replace(old, new)
        if package == "gpui-pre":
            content = content.replace('version = "0.13.0"', 'version = "0.15.0"')
        else:
            content = content.replace('version = "0.15.0"', 'version = "0.13.0"')
        content = content.replace('version = "0.7.3"', f'version = "{source["version"]}"')
        content = content.replace('version = "0.3.9"', f'version = "{source["version"]}"')
        cargo.write_text(content, encoding="utf-8")
        count += 1
    return count


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    args = parser.parse_args()
    try:
        number = refresh(args.manifest)
    except (OSError, RuntimeError, json.JSONDecodeError) as error:
        print(f"owned GPUI mirror refresh failed: {error}", file=sys.stderr)
        raise SystemExit(1)
    print(f"refreshed owned GPUI mirrors={number}")
