#!/usr/bin/env python3
"""Invalidate Cargo unit stamps when an active Fast candidate changes.

Registry packages are treated as immutable by Cargo. Preserve artifacts, but
force patched packages and their reverse dependencies to be compiled again.
"""
import argparse
import fcntl
import hashlib
import os
from pathlib import Path
import sys
import tomllib


def invalidate(root: Path, target: Path) -> int:
    manifest = tomllib.loads((root / "Cargo.toml").read_text())
    dependency = manifest.get("workspace", {}).get("dependencies", {}).get("gpui", {})
    if not isinstance(dependency, dict) or dependency.get("package") != "gpui-fast":
        return 0
    candidates = root / "patches/candidate-kit-fast"
    digest = hashlib.sha256()
    for path in sorted(candidates.iterdir()):
        if path.suffix in (".json", ".patch"):
            digest.update(path.name.encode() + b"\0" + path.read_bytes())
    key = digest.hexdigest()
    target.mkdir(parents=True, exist_ok=True)
    marker = target / ".rustui-fast-patch-hash"
    with (target / ".rustui-fast-patch-lock").open("a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        if marker.is_file() and marker.read_text().strip() == key:
            return 0
        packages = tomllib.loads((root / "Cargo.lock").read_text())["package"]
        affected = {"gpui-fast"}
        while True:
            expanded = affected | {package["name"] for package in packages
                if any(dep.split(" ", 1)[0] in affected for dep in package.get("dependencies", []))}
            if expanded == affected:
                break
            affected = expanded
        count = 0
        # Normal/custom profiles, with or without an explicit target triple.
        for fingerprints in sorted(set(target.glob("*/.fingerprint")) | set(target.glob("*/*/.fingerprint"))):
            for unit in fingerprints.iterdir():
                if not unit.is_dir() or unit.name.rsplit("-", 1)[0] not in affected:
                    continue
                for stamp in unit.iterdir():
                    if stamp.is_file() and stamp.suffix == "" and stamp.name.startswith(("lib-", "test-lib-", "bin-", "test-bin-", "run-build-script-", "build-script-")):
                        stamp.unlink()
                        count += 1
        temporary = marker.with_suffix(".tmp")
        temporary.write_text(key + "\n")
        temporary.replace(marker)
        if count:
            print(f"Patched vendor cache: invalidated {count} unit stamps; compiled artifacts preserved", file=sys.stderr)
        return count


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("cargo_args", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    cargo_args = args.cargo_args
    if cargo_args[:1] == ["--"]:
        cargo_args = cargo_args[1:]
    if not cargo_args or cargo_args[0] in {"--version", "-V", "--help", "-h", "help", "fmt", "metadata", "tree", "locate-project", "pkgid", "search", "info"}:
        return
    root = args.root.resolve()
    # Fixtures outside this checkout have a different source graph/cache.
    if not Path.cwd().resolve().is_relative_to(root):
        return
    configured = tomllib.loads((root / ".cargo/config.toml").read_text()).get("build", {}).get("target-dir", "target")
    target = os.environ.get("CARGO_TARGET_DIR", configured)
    for index, argument in enumerate(cargo_args):
        if argument == "--target-dir" and index + 1 < len(cargo_args):
            target = cargo_args[index + 1]
        elif argument.startswith("--target-dir="):
            target = argument.split("=", 1)[1]
    target = Path(target)
    invalidate(root, target if target.is_absolute() else root / target)


if __name__ == "__main__":
    main()
