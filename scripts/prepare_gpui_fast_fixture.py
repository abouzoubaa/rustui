#!/usr/bin/env python3
"""Download, hash-check and extract the pinned GPUI Fast 0.1.14 crates."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from prepare_gpui_kit_09_fixture import (
    FixtureError,
    download_archive,
    extract_verified_crate,
    load_manifest,
    verify_archive_file,
)


ROOT = Path(__file__).resolve().parent.parent
DEFAULT_MANIFEST = ROOT / "scripts/data/gpui-fast-0.1.14-sources.json"
DEFAULT_FIXTURE = ROOT / "target/headstart/f0-fast-fixture"


def prepare(manifest_path: Path, fixture: Path) -> tuple[int, int]:
    manifest = load_manifest(manifest_path)
    fixture.mkdir(parents=True, exist_ok=True)
    archives = fixture / "archives"
    crates = fixture / "crates"
    archives.mkdir(parents=True, exist_ok=True)
    crates.mkdir(parents=True, exist_ok=True)
    downloaded = extracted = 0
    for source in manifest["sources"]:
        name, version = source["name"], source["version"]
        archive = archives / f"{name}-{version}.crate"
        if not archive.is_file():
            download_archive(source, archive)
            downloaded += 1
        else:
            verify_archive_file(archive, source["archive_sha256"])
        destination = crates / f"{name}-{version}"
        if destination.exists():
            checksum_path = destination / ".cargo-checksum.json"
            if not checksum_path.is_file():
                raise FixtureError(f"Cargo checksum metadata is missing: {checksum_path}")
            checksum = json.loads(checksum_path.read_text(encoding="utf-8"))
            if checksum.get("package") != source["archive_sha256"]:
                raise FixtureError(f"extracted archive hash mismatch: {name}-{version}")
        else:
            extract_verified_crate(archive, source, destination)
            extracted += 1
    return downloaded, extracted


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--fixture-dir", type=Path, default=DEFAULT_FIXTURE)
    args = parser.parse_args()
    try:
        downloaded, extracted = prepare(args.manifest, args.fixture_dir)
    except FixtureError as error:
        print(f"GPUI Fast fixture failed: {error}", file=sys.stderr)
        return 1
    count = len(load_manifest(args.manifest)["sources"])
    print(
        f"verified GPUI Fast archives={count}; newly downloaded={downloaded}; "
        f"newly extracted={extracted}; fixture={args.fixture_dir}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
