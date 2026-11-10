#!/usr/bin/env python3
"""Install pinned GPUI Kit 0.8.1 crate archives into vendor and owned mirrors."""

from __future__ import annotations

import argparse
import json
import shutil
import sys
from pathlib import Path

from prepare_gpui_kit_08_fixture import (
    DEFAULT_MANIFEST,
    FixtureError,
    extract_verified_crate,
    load_manifest,
)


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
}
VERSION_REWRITES = {
    'version = "=0.3.9"': 'version = "=0.3.10"',
    'version = "=0.7.3"': 'version = "=0.8.1"',
    'version = "0.3.9"': 'version = "0.3.10"',
    'version = "0.7.3"': 'version = "0.8.1"',
    'version = "0.7.1"': 'version = "0.8.1"',
}


def install(fixture: Path, manifest_path: Path, apply: bool) -> None:
    manifest = load_manifest(manifest_path)
    sources = manifest["sources"]
    vendor = ROOT / "vendor"
    for source in sources:
        package, version = source["name"], source["version"]
        archive = fixture / "archives" / f"{package}-{version}.crate"
        extracted = fixture / "crates" / f"{package}-{version}"
        if not extracted.is_dir():
            raise FixtureError(f"verified extraction is missing: {extracted}")
        # Recheck the archive and extraction metadata before installing it.
        extract_verified_crate(archive, source, fixture / "install-check" / f"{package}-{version}")
        shutil.rmtree(fixture / "install-check" / f"{package}-{version}")
        destination = vendor / f"{package}-{version}"
        if not apply:
            if not destination.is_dir():
                raise FixtureError(f"not installed: {destination}")
            continue
        for old in vendor.glob(f"{package}-*"):
            if old.is_dir() and old.name != destination.name:
                shutil.rmtree(old)
        if destination.exists():
            shutil.rmtree(destination)
        shutil.copytree(extracted, destination)

    if not apply:
        return

    for source in sources:
        package = source["name"]
        relative = OWNED.get(package)
        if relative is None:
            continue
        source_dir = vendor / f"{package}-{source['version']}"
        owned = ROOT / relative
        owned.mkdir(parents=True, exist_ok=True)
        for child in list(owned.iterdir()):
            if child.name != "Cargo.toml":
                if child.is_dir():
                    shutil.rmtree(child)
                else:
                    child.unlink()
        for child in source_dir.iterdir():
            if child.name == "Cargo.toml":
                continue
            target = owned / child.name
            if child.is_dir():
                shutil.copytree(child, target)
            else:
                shutil.copy2(child, target)
        cargo = owned / "Cargo.toml"
        if not cargo.exists():
            shutil.copy2(source_dir / "Cargo.toml", cargo)
        content = cargo.read_text(encoding="utf-8")
        for old, new in VERSION_REWRITES.items():
            content = content.replace(old, new)
        content = content.replace('version = "0.7.3"', f'version = "{source["version"]}"')
        content = content.replace('version = "0.3.9"', f'version = "{source["version"]}"')
        cargo.write_text(content, encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture-dir", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--apply", action="store_true", help="install the verified source archives")
    args = parser.parse_args()
    try:
        install(args.fixture_dir, args.manifest, args.apply)
    except (FixtureError, OSError, json.JSONDecodeError) as error:
        print(f"GPUI Kit source installation failed: {error}", file=sys.stderr)
        return 1
    print("installed pinned GPUI Kit 0.8.1 / gpui-pre 0.3.10 sources" if args.apply else "source inventory verified")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
