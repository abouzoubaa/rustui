#!/usr/bin/env python3
"""Replay the approved block 0.1.6 stack-symbol patch with pinned inputs."""
import argparse
import hashlib
import os
from pathlib import Path
import subprocess
import tempfile
import tomllib

SOURCE = Path("vendor/block-0.1.6/src/lib.rs")
PATCH = Path("patches/approved/block-0.1.6-stack-symbol-v1.patch")
PRE = "eb31678adf63b53109d9b94eba23699fd5f9ebfdb950f6e1a57ad51bb6a146fa"
POST = "798f4b286defc8fa722247a4b6c031d2a9b5144fc916e2e5c4cbb0ed85c8b4dd"
PATCH_HASH = "d0fc7338cf3e8cc97477157e1cff3719257566105909a2a3f2af788c03e5fac6"
PRESERVED = {
    "vendor/block-0.1.6/Cargo.toml": "24df2f33b48c9e756a4f864f882c1ab07a159b6c2b790f45af2e8162499b6719",
    "vendor/block-0.1.6/.cargo-checksum.json": "436ff3319faf156bd9a969481c9b0bf0cbebe3b22719c1353a3513447bab4fbf",
}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def run(root, artifacts, mode):
    root, artifacts = root.resolve(), artifacts.resolve()
    package = tomllib.loads((root / "vendor/block-0.1.6/Cargo.toml").read_text())["package"]
    selected = tomllib.loads((root / "Cargo.toml").read_text())["patch"]["crates-io"]["block"]
    if (package.get("name"), package.get("version"), selected.get("path")) != ("block", "0.1.6", "vendor/block-0.1.6"):
        raise ValueError("pinned block package selection mismatch")
    for path, expected in PRESERVED.items():
        if digest((root / path).read_bytes()) != expected:
            raise ValueError(f"upstream metadata changed: {path}")
    patch = artifacts / PATCH
    if digest(patch.read_bytes()) != PATCH_HASH:
        raise ValueError("block patch hash mismatch")
    source = root / SOURCE
    if source.is_symlink():
        raise ValueError("block source must not be a symlink")
    old = source.read_bytes()
    current = digest(old)
    if current not in (PRE, POST):
        raise ValueError("block source is not the known preimage or postimage")
    reverse = current == POST
    if mode == "--verify" and not reverse:
        raise ValueError("block patch postimage required")
    with tempfile.TemporaryDirectory(prefix="rustui-block-patch-") as folder:
        stage = Path(folder)
        staged = stage / SOURCE
        staged.parent.mkdir(parents=True)
        staged.write_bytes(old)
        command = ["git", "apply", "--whitespace=nowarn"] + (["--reverse"] if reverse else [])
        subprocess.run(command + ["--check", str(patch)], cwd=stage, check=True, capture_output=True)
        subprocess.run(command + [str(patch)], cwd=stage, check=True, capture_output=True)
        new = staged.read_bytes()
        if digest(new) != (PRE if reverse else POST):
            raise ValueError("block patch replay hash mismatch")
        undo = ["git", "apply", "--whitespace=nowarn"] + ([] if reverse else ["--reverse"])
        subprocess.run(undo + [str(patch)], cwd=stage, check=True, capture_output=True)
        if staged.read_bytes() != old:
            raise ValueError("block patch reversal mismatch")
    if mode in ("--apply", "--reverse"):
        if (mode == "--apply") == reverse:
            raise ValueError("block source already has the requested image")
        if source.read_bytes() != old:
            raise ValueError("block source changed during staging")
        fd, name = tempfile.mkstemp(prefix=".rustui-block-", dir=source.parent)
        try:
            os.fchmod(fd, source.stat().st_mode & 0o777)
            with os.fdopen(fd, "wb") as output:
                output.write(new)
            if source.read_bytes() != old:
                raise ValueError("block source changed during staging")
            os.replace(name, source)
        finally:
            if os.path.exists(name):
                os.unlink(name)
    print("block stack-symbol patch replay verified")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    for option in ("--verify", "--apply", "--reverse"):
        group.add_argument(option, action="store_const", dest="mode", const=option)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--artifacts", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    run(args.root, args.artifacts, args.mode)
