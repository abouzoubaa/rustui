#!/usr/bin/env python3
"""Replay the approved GPUI Base first-move resize patch without changing revisions."""

import argparse
import hashlib
import os
import subprocess
import tempfile
import tomllib
from pathlib import Path

SOURCE = Path("vendor/gpui-base-0.7.3/src/resizable/panel.rs")
PATCH = Path("patches/approved/gpui-base-resize-first-move-v3.patch")
PRE = "c54be541d3d77e76b286227f6d4cc01856f75a9a549e799a63887f736ef9f0c4"
POST = "0e02f15458b15194300f4ae1ed6c14960aa0e24164e73b624b6a06b4a7c97274"
PATCH_HASH = "acfcfc996105d8d27b0bcfbf6bb9a7b78f83a3e9e56be0d1ea5d316a7fea0dc6"
PRESERVED = {
    "vendor/gpui-base-0.7.3/Cargo.toml": "b6fa9bb529c7e049df902890f12f5e15b298aebea6a46bb8e5101d6cade298cb",
    "vendor/gpui-base-0.7.3/.cargo-checksum.json": "9e96d55b0071b60576bc6ac18c3e9c48e6d0de5ff848821af7623bd151f7c3ed",
}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def run(root, artifacts, mode):
    root, artifacts = root.resolve(), artifacts.resolve()
    package = tomllib.loads((root / "vendor/gpui-base-0.7.3/Cargo.toml").read_text())["package"]
    selected = tomllib.loads((root / "Cargo.toml").read_text())["workspace"]["dependencies"]["gpui-base"]
    selected_path = selected.get("path")
    selected_package = tomllib.loads((root / selected_path / "Cargo.toml").read_text())["package"] if selected_path else {}
    if (
        package.get("name"), package.get("version"), selected_path,
        selected_package.get("name"), selected_package.get("version")
    ) != ("gpui-base", "0.7.3", "crates/rustui-gpui-base", selected.get("package", "gpui-base"), "0.7.3"):
        raise ValueError("owned GPUI Base selection mismatch")
    for path, expected in PRESERVED.items():
        if digest((root / path).read_bytes()) != expected:
            raise ValueError(f"upstream metadata changed: {path}")
    patch = artifacts / PATCH
    if digest(patch.read_bytes()) != PATCH_HASH:
        raise ValueError("resize patch hash mismatch")
    source = root / SOURCE
    if source.is_symlink():
        raise ValueError("resize source must not be a symlink")
    old = source.read_bytes()
    current = digest(old)
    if current not in (PRE, POST):
        raise ValueError("unknown or partial resize source")
    reverse = current == POST
    if mode == "--verify" and not reverse:
        raise ValueError("resize patch postimage required")
    with tempfile.TemporaryDirectory(prefix="rustui-resize-patch-") as folder:
        stage = Path(folder)
        staged = stage / SOURCE
        staged.parent.mkdir(parents=True)
        staged.write_bytes(old)
        command = ["git", "apply", "--unidiff-zero", "--whitespace=nowarn"] + (["--reverse"] if reverse else [])
        subprocess.run(command + ["--check", str(patch)], cwd=stage, check=True, capture_output=True)
        subprocess.run(command + [str(patch)], cwd=stage, check=True, capture_output=True)
        new = staged.read_bytes()
        if digest(new) != (PRE if reverse else POST):
            raise ValueError("resize patch replay hash mismatch")
        undo = ["git", "apply", "--unidiff-zero", "--whitespace=nowarn"] + ([] if reverse else ["--reverse"])
        subprocess.run(undo + [str(patch)], cwd=stage, check=True, capture_output=True)
        if staged.read_bytes() != old:
            raise ValueError("resize patch reversal mismatch")
    if mode in ("--apply", "--reverse"):
        if (mode == "--apply") == reverse:
            raise ValueError("resize source already has requested image")
        if source.read_bytes() != old:
            raise ValueError("resize source changed during staging")
        fd, name = tempfile.mkstemp(prefix=".rustui-resize-", dir=source.parent)
        try:
            os.fchmod(fd, source.stat().st_mode & 0o777)
            with os.fdopen(fd, "wb") as output:
                output.write(new)
            if source.read_bytes() != old:
                raise ValueError("resize source changed during staging")
            os.replace(name, source)
        finally:
            Path(name).unlink(missing_ok=True)
    print("GPUI Base resize first-move patch replay verified")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--artifacts", type=Path)
    group = parser.add_mutually_exclusive_group(required=True)
    for mode in ("--check", "--apply", "--verify", "--reverse"):
        group.add_argument(mode, action="store_true")
    args = parser.parse_args()
    mode = next(mode for mode in ("--check", "--apply", "--verify", "--reverse") if getattr(args, mode[2:]))
    try:
        run(args.root, args.artifacts or args.root, mode)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"error: {error}\n")


if __name__ == "__main__":
    main()
