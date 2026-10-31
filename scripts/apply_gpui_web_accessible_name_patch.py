#!/usr/bin/env python3
"""Verify or apply the proposed GPUI Web accessible-name patch."""

from __future__ import annotations

import argparse
import hashlib
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import tomllib


PATCH = Path("patches/approved/gpui-pre-web-accessible-name-v1.patch")
PATCH_SHA256 = "8fbb44e848e423978c7a216dcd3e814fdf477e520657dba7a7a8bd8653ed9881"
PACKAGES = {
    "vendor/gpui-pre-0.3.9": {
        "name": "gpui-pre",
        "version": "0.3.9",
        "checksum": "7b8e889ae4ac0cbe651b654f5e39d9003969d09ae4a70aff4147f521eda8faa0",
    },
    "vendor/gpui-pre-web-0.3.9": {
        "name": "gpui-pre-web",
        "version": "0.3.9",
        "checksum": "aad9503f554e462d582461d008ea4bc0dda00021812ccf6b876734b55757579e",
    },
}
FILES = {
    Path("vendor/gpui-pre-0.3.9/src/window.rs"): (
        "44f32319b9385fa6dbc15c36b3edea00a26208309b37283ae4f9bd4bead674f8",
        "66d78f6ca924808cba8a9121fd6e44e6537ccf87865c6b443b11a4fe1c140855",
    ),
    Path("vendor/gpui-pre-web-0.3.9/src/window.rs"): (
        "3779ba8d5647009b80d0ea52dc7823806640741d3a1cfa9a0a7752ec35fe7dd8",
        "07f241ba3476ce5b3e76927e768a00a84207869115482b7dba1297971d9d4bfa",
    ),
    Path("vendor/gpui-pre-web-0.3.9/src/ime_mirror.rs"): (
        "4061ebd319b5928c2a25f659c4f1db9adb193d5873bbbfeba36468ca2706895d",
        "5647cf3f467837f1b1747aac15fb0f4e6bac1bbe862d040427ba9e48b841088f",
    ),
}


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify_identity(root: Path) -> None:
    for relative, expected in PACKAGES.items():
        package_root = root / relative
        package = tomllib.loads((package_root / "Cargo.toml").read_text(encoding="utf-8"))["package"]
        if (package.get("name"), package.get("version")) != (expected["name"], expected["version"]):
            raise ValueError(f"patch requires {expected['name']} {expected['version']}")
        if digest(package_root / ".cargo-checksum.json") != expected["checksum"]:
            raise ValueError(f"pristine upstream checksum metadata changed: {relative}")
    if digest(root / PATCH) != PATCH_SHA256:
        raise ValueError("proposed patch SHA-256 mismatch")
    cargo = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
    if cargo.get("patch", {}).get("crates-io", {}).get("gpui-pre-web") != {
        "path": "vendor/gpui-pre-web-0.3.9"
    }:
        raise ValueError("Cargo.toml must retain the approved M855 local Web package override")


def stage_patch(root: Path) -> dict[Path, bytes]:
    originals = {path: (root / path).read_bytes() for path in FILES}
    with tempfile.TemporaryDirectory(prefix="rustui-gpui-web-accessible-name-") as directory:
        stage = Path(directory)
        for relative, content in originals.items():
            target = stage / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(content)
        command = ["patch", "--batch", "--fuzz=0", "-p1", "-i", str((root / PATCH).resolve())]
        result = subprocess.run(command, cwd=stage, capture_output=True, text=True)
        output = result.stdout + result.stderr
        if result.returncode or any(token in output.lower() for token in ("offset", "fuzz", "failed")):
            raise ValueError(f"exact patch application failed: {output.strip()}")
        result_files = {}
        for relative, hashes in FILES.items():
            content = (stage / relative).read_bytes()
            if hashlib.sha256(content).hexdigest() != hashes[1]:
                raise ValueError(f"unexpected patch postimage: {relative}")
            result_files[relative] = content
        return result_files


def install(root: Path, files: dict[Path, bytes]) -> None:
    temporary = {}
    try:
        for relative, content in files.items():
            target = root / relative
            temp = target.with_name(f".{target.name}.accessible-name-patch-tmp")
            temp.write_bytes(content)
            temporary[target] = temp
        for target, temp in temporary.items():
            os.replace(temp, target)
    finally:
        for temp in temporary.values():
            temp.unlink(missing_ok=True)


def run(root: Path, apply: bool) -> None:
    root = root.resolve()
    verify_identity(root)
    if not apply and any(digest(root / relative) != hashes[0] for relative, hashes in FILES.items()):
        result = subprocess.run([sys.executable,str(root / "scripts/apply_gpui_web_semantic_adapter.py"),"--verify-installed","--root",str(root)],capture_output=True,text=True)
        if result.returncode:raise ValueError(f"composed Web verification failed: {result.stdout}{result.stderr}")
        print("approved accessible-name patch and exact composed Web stack verified; checkout unchanged")
        return
    for relative, hashes in FILES.items():
        if digest(root / relative) != hashes[0]:raise ValueError(f"unexpected source SHA-256: {relative}")
    staged=stage_patch(root)
    if apply:install(root,staged);print("approved GPUI Web accessible-name patch applied")
    else:print("approved GPUI Web accessible-name PRE round trip verified; checkout unchanged")



def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--apply", action="store_true")
    mode.add_argument("--check", action="store_true")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    try:
        run(args.root, apply=args.apply)
    except (OSError, ValueError, KeyError, tomllib.TOMLDecodeError, subprocess.SubprocessError) as error:
        parser.exit(1, f"error: {error}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
