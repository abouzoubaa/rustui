#!/usr/bin/env python3
"""Replay the approved gpui-component Select RTL alignment patch with pinned inputs."""
import argparse
import hashlib
import os
from pathlib import Path
import subprocess
import tempfile
import tomllib

SOURCE = Path("vendor/gpui-component-0.7.3/src/select.rs")
PATCH = Path("patches/approved/gpui-component-select-rtl-alignment-v3.patch")
PRE = "1a1559e6b46cfb562b565d85600bef7731c3015ae5354d0dcc10454e3bc51efe"
POST = "aef4bdbc7dbeab626b1091be47ddfb1b330028b7904daa75173914fbb2a84a8e"
PATCH_HASH = "ccc86d278b2ee52ed8b371867cf91e6e7e6c25fc222a8194e9a07c2c4c373a1c"
PRESERVED = {
    "vendor/gpui-component-0.7.3/Cargo.toml": "192f4156aee8738567c748d6b1c5bc44c80e4ba5369a41e5349e370d83b35026",
    "vendor/gpui-component-0.7.3/.cargo-checksum.json": "d5b533efba197cc15bcb6756b935a786e846463c1ee431f876e8c334a7e638c6",
}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def run(root, artifacts, mode):
    root, artifacts = root.resolve(), artifacts.resolve()
    package = tomllib.loads((root / "vendor/gpui-component-0.7.3/Cargo.toml").read_text())["package"]
    selected = tomllib.loads((root / "Cargo.toml").read_text())["patch"]["crates-io"]["gpui-component"]
    if (package.get("name"), package.get("version"), selected.get("path")) != ("gpui-component", "0.7.3", "vendor/gpui-component-0.7.3"):
        raise ValueError("pinned gpui-component package selection mismatch")
    for path, expected in PRESERVED.items():
        if digest((root / path).read_bytes()) != expected:
            raise ValueError(f"upstream metadata changed: {path}")
    patch = artifacts / PATCH
    if digest(patch.read_bytes()) != PATCH_HASH:
        raise ValueError("Select RTL patch hash mismatch")
    source = root / SOURCE
    if source.is_symlink():
        raise ValueError("Select source must not be a symlink")
    old = source.read_bytes()
    current = digest(old)
    if current not in (PRE, POST):
        raise ValueError("Select source is not the known preimage or postimage")
    reverse = current == POST
    if mode == "--verify" and not reverse:
        raise ValueError("Select RTL patch postimage required")
    with tempfile.TemporaryDirectory(prefix="rustui-select-rtl-patch-") as folder:
        stage = Path(folder)
        staged = stage / SOURCE
        staged.parent.mkdir(parents=True)
        staged.write_bytes(old)
        command = ["git", "apply", "--whitespace=nowarn"] + (["--reverse"] if reverse else [])
        subprocess.run(command + ["--check", str(patch)], cwd=stage, check=True, capture_output=True)
        subprocess.run(command + [str(patch)], cwd=stage, check=True, capture_output=True)
        new = staged.read_bytes()
        if digest(new) != (PRE if reverse else POST):
            raise ValueError("Select RTL patch replay hash mismatch")
        undo = ["git", "apply", "--whitespace=nowarn"] + ([] if reverse else ["--reverse"])
        subprocess.run(undo + [str(patch)], cwd=stage, check=True, capture_output=True)
        if staged.read_bytes() != old:
            raise ValueError("Select RTL patch reversal mismatch")
    if mode in ("--apply", "--reverse"):
        if (mode == "--apply") == reverse:
            raise ValueError("Select source already has the requested image")
        if source.read_bytes() != old:
            raise ValueError("Select source changed during staging")
        fd, name = tempfile.mkstemp(prefix=".rustui-select-rtl-", dir=source.parent)
        try:
            os.fchmod(fd, source.stat().st_mode & 0o777)
            with os.fdopen(fd, "wb") as output:
                output.write(new)
            if source.read_bytes() != old:
                raise ValueError("Select source changed during staging")
            os.replace(name, source)
        finally:
            if os.path.exists(name):
                os.unlink(name)
    print("Select RTL patch replay verified")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    for option in ("--verify", "--apply", "--reverse"):
        group.add_argument(option, action="store_const", dest="mode", const=option)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--artifacts", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    run(args.root, args.artifacts, args.mode)
