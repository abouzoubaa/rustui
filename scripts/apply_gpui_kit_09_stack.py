#!/usr/bin/env python3
"""Apply, verify, or reverse the promoted GPUI Kit 0.9 patch stack."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
STACK = ROOT / "patches/approved/kit09/kit09-stack.json"
MIRRORS = {
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


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_stack() -> dict:
    stack = json.loads(STACK.read_text(encoding="utf-8"))
    for item in stack["patches"]:
        patch = ROOT / "patches/approved/kit09" / item["patch"]
        if not patch.is_file() or sha(patch) != item["patch_sha256"]:
            raise RuntimeError(f"Kit 0.9 approved patch missing or hash mismatch: {patch}")
        if not patch.with_suffix(".json").is_file():
            raise RuntimeError(f"Kit 0.9 per-patch manifest missing: {patch.with_suffix('.json')}")
    return stack


def refresh_mirrors(stack: dict) -> int:
    version_by_package = {}
    for target in stack["files"]:
        parts = Path(target).parts
        if len(parts) > 2 and parts[0] == "vendor":
            directory = parts[1]
            for package in sorted(MIRRORS, key=len, reverse=True):
                if directory.startswith(package + "-"):
                    version_by_package[package] = directory[len(package) + 1 :]
                    break
    refreshed = 0
    for package, relative in MIRRORS.items():
        version = version_by_package.get(package)
        if version is None:
            continue
        source = ROOT / "vendor" / f"{package}-{version}"
        destination = ROOT / relative
        if not source.is_dir() or not destination.is_dir():
            raise RuntimeError(f"cannot refresh owned mirror: {source} -> {destination}")
        for child in list(destination.iterdir()):
            if child.name == "Cargo.toml":
                continue
            if child.is_dir():
                import shutil
                shutil.rmtree(child)
            else:
                child.unlink()
        for child in source.iterdir():
            if child.name == "Cargo.toml":
                continue
            target = destination / child.name
            if child.is_dir():
                import shutil
                shutil.copytree(child, target)
            else:
                import shutil
                shutil.copy2(child, target)
        refreshed += 1
    return refreshed


def operate(action: str) -> None:
    stack = load_stack()
    if action == "--check":
        for target, hashes in stack["files"].items():
            path = ROOT / target
            if not path.is_file() or sha(path) != hashes["post_sha256"]:
                raise RuntimeError(f"Kit 0.9 postimage mismatch: {target}")
        print(f"Kit 0.9 approved patches={len(stack['patches'])}; files={len(stack['files'])}; applied state=PASS")
        return

    entries = list(reversed(stack["patches"])) if action == "--reverse" else stack["patches"]
    for entry in entries:
        patch = ROOT / "patches/approved/kit09" / entry["patch"]
        expected_key = "post_sha256" if action == "--reverse" else "pre_sha256"
        result_key = "pre_sha256" if action == "--reverse" else "post_sha256"
        command = ["git", "apply", "--reverse"] if action == "--reverse" else ["git", "apply"]
        for target, hashes in entry["files"].items():
            path = ROOT / target
            if not path.is_file() or sha(path) != hashes[expected_key]:
                raise RuntimeError(f"{action} image mismatch: {target}")
        subprocess.run(["git", "apply", "--check", *(command[2:]), str(patch)], cwd=ROOT, check=True)
        subprocess.run([*command, str(patch)], cwd=ROOT, check=True)
        for target, hashes in entry["files"].items():
            if sha(ROOT / target) != hashes[result_key]:
                raise RuntimeError(f"{action} result mismatch: {target}")
    refreshed = refresh_mirrors(stack)
    print(f"Kit 0.9 approved patches={len(entries)}; action={action}; hashes=PASS; mirrors refreshed={refreshed}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--apply", action="store_true")
    group.add_argument("--check", action="store_true")
    group.add_argument("--reverse", action="store_true")
    args = parser.parse_args()
    action = "--apply" if args.apply else "--reverse" if args.reverse else "--check"
    try:
        operate(action)
    except (OSError, RuntimeError, json.JSONDecodeError, subprocess.CalledProcessError) as error:
        print(f"GPUI Kit 0.9 patch stack failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
