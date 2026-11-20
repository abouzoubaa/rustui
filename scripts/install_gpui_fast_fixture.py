#!/usr/bin/env python3
"""Install only the hash-verified GPUI Fast crates from the F0 fixture."""

from __future__ import annotations

import argparse
import json
import shutil
import sys
import tempfile
from pathlib import Path

from prepare_gpui_kit_09_fixture import FixtureError, load_manifest, verify_archive_file


ROOT = Path(__file__).resolve().parent.parent
DEFAULT_MANIFEST = ROOT / "scripts/data/gpui-fast-0.1.14-sources.json"
DEFAULT_FIXTURE = ROOT / "target/headstart/f0-fast-fixture"


def install(manifest_path: Path, fixture: Path, root: Path) -> tuple[int, int]:
    manifest = load_manifest(manifest_path)
    vendor = root / "vendor"
    sources = []
    for source in manifest["sources"]:
        name, version = source["name"], source["version"]
        archive = fixture / "archives" / f"{name}-{version}.crate"
        extracted = fixture / "crates" / f"{name}-{version}"
        verify_archive_file(archive, source["archive_sha256"])
        checksum_path = extracted / ".cargo-checksum.json"
        if not checksum_path.is_file():
            raise FixtureError(f"verified extraction is missing: {extracted}")
        checksum = json.loads(checksum_path.read_text(encoding="utf-8"))
        if checksum.get("package") != source["archive_sha256"]:
            raise FixtureError(f"extracted package hash mismatch: {name}-{version}")
        destination = vendor / f"{name}-{version}"
        if destination.exists():
            current = destination / ".cargo-checksum.json"
            if not current.is_file() or json.loads(current.read_text()).get("package") != source["archive_sha256"]:
                raise FixtureError(f"refusing to replace a different vendored package: {destination}")
        sources.append((extracted, destination, destination.exists()))

    added = kept = 0
    try:
        with tempfile.TemporaryDirectory(prefix="rustui-f0-fast-vendor-", dir=vendor) as temporary:
            staged: list[tuple[Path, Path]] = []
            for extracted, destination, exists in sources:
                if exists:
                    kept += 1
                    continue
                temporary_path = Path(temporary) / destination.name
                shutil.copytree(extracted, temporary_path)
                staged.append((temporary_path, destination))
            installed: list[Path] = []
            try:
                for temporary_path, destination in staged:
                    temporary_path.rename(destination)
                    installed.append(destination)
                    added += 1
            except OSError:
                for destination in installed:
                    shutil.rmtree(destination)
                raise
    except OSError as error:
        raise FixtureError(f"cannot install GPUI Fast vendor crates: {error}") from error
    return added, kept


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--fixture-dir", type=Path, default=DEFAULT_FIXTURE)
    parser.add_argument("--root", type=Path, default=ROOT)
    args = parser.parse_args()
    try:
        added, kept = install(args.manifest, args.fixture_dir, args.root)
    except FixtureError as error:
        print(f"GPUI Fast install failed: {error}", file=sys.stderr)
        return 1
    print(f"GPUI Fast vendor packages added={added}; already verified={kept}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
