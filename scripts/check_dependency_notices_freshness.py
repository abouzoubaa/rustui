#!/usr/bin/env python3
"""Validate the packaged notice snapshot against the current lockfile."""
import hashlib
import json
from pathlib import Path

from package_release_notices import FONT_LICENSES, validate_collection

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def check(root):
    snapshot = root / "docs/release/current-notices"
    binding = json.loads((snapshot / "lockfile.json").read_text())
    if binding.get("schema_version") != 1:
        raise ValueError("unsupported notice lockfile binding")
    if binding.get("cargo_lock_sha256") != digest(root / "Cargo.lock"):
        raise ValueError("dependency notices are out of date with Cargo.lock")
    supplement = digest(root / "docs/release/notice-supplements/manifest.json")
    if binding.get("supplement_manifest_sha256") != supplement:
        raise ValueError("dependency notices are out of date with supplements")
    report = validate_collection(snapshot / "dependency-notices")
    if report["unresolved_gaps"]:
        raise ValueError("dependency notices contain unresolved gaps")
    if report["inputs"].get("cargo_lock_sha256") != binding.get("cargo_lock_sha256"):
        raise ValueError("notice collection used a different Cargo.lock")
    if report["inputs"].get("supplement_manifest_sha256") != supplement:
        raise ValueError("notice collection used different supplements")
    for name in FONT_LICENSES:
        if (snapshot / "fonts" / name).read_bytes() != (root / "crates/rustui/assets/fonts" / name).read_bytes():
            raise ValueError(f"packaged font license is out of date: {name}")
    return len(report["packages"])


if __name__ == "__main__":
    try:
        count = check(ROOT)
    except (OSError, ValueError, KeyError, TypeError, json.JSONDecodeError) as error:
        raise SystemExit(f"Dependency notice check failed: {error}")
    print(f"Dependency notices match Cargo.lock: {count} observed packages.")
