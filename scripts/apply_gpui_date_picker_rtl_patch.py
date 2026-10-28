#!/usr/bin/env python3
"""Replay the approved gpui-component DatePicker RTL alignment patch with pinned inputs."""
import argparse
import hashlib
import os
from pathlib import Path
import subprocess
import tempfile
import tomllib

SOURCE = Path("vendor/gpui-component-0.7.3/src/time/date_picker.rs")
PATCH = Path("patches/approved/gpui-component-date-picker-rtl-alignment-v1.patch")
PRE = "83e639bbaf3194f81c33864f8e885f683faa403154e40253454215ed8462467d"
POST = "7a887f3f139ba9a658d3a42de45a768e7c4def7406bea7ddd540dc88ef0d1df0"
PATCH_HASH = "70fd646f76e0e4e2a279c7c914af0e29302cd7d5d3c52affb24b1a1a5199aa15"
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
        raise ValueError("DatePicker RTL patch hash mismatch")
    source = root / SOURCE
    if source.is_symlink():
        raise ValueError("DatePicker source must not be a symlink")
    old = source.read_bytes()
    current = digest(old)
    if current not in (PRE, POST):
        raise ValueError("DatePicker source is not the known preimage or postimage")
    reverse = current == POST
    if mode == "--verify" and not reverse:
        raise ValueError("DatePicker RTL patch postimage required")
    with tempfile.TemporaryDirectory(prefix="rustui-date-rtl-patch-") as folder:
        stage = Path(folder)
        staged = stage / SOURCE
        staged.parent.mkdir(parents=True)
        staged.write_bytes(old)
        command = ["git", "apply", "--whitespace=nowarn"] + (["--reverse"] if reverse else [])
        subprocess.run(command + ["--check", str(patch)], cwd=stage, check=True, capture_output=True)
        subprocess.run(command + [str(patch)], cwd=stage, check=True, capture_output=True)
        new = staged.read_bytes()
        if digest(new) != (PRE if reverse else POST):
            raise ValueError("DatePicker RTL patch replay hash mismatch")
        undo = ["git", "apply", "--whitespace=nowarn"] + ([] if reverse else ["--reverse"])
        subprocess.run(undo + [str(patch)], cwd=stage, check=True, capture_output=True)
        if staged.read_bytes() != old:
            raise ValueError("DatePicker RTL patch reversal mismatch")
    if mode in ("--apply", "--reverse"):
        if (mode == "--apply") == reverse:
            raise ValueError("DatePicker source already has the requested image")
        if source.read_bytes() != old:
            raise ValueError("DatePicker source changed during staging")
        fd, name = tempfile.mkstemp(prefix=".rustui-date-rtl-", dir=source.parent)
        try:
            os.fchmod(fd, source.stat().st_mode & 0o777)
            with os.fdopen(fd, "wb") as output:
                output.write(new)
            if source.read_bytes() != old:
                raise ValueError("DatePicker source changed during staging")
            os.replace(name, source)
        finally:
            if os.path.exists(name):
                os.unlink(name)
    print("DatePicker RTL patch replay verified")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    for option in ("--verify", "--apply", "--reverse"):
        group.add_argument(option, action="store_const", dest="mode", const=option)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--artifacts", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    run(args.root, args.artifacts, args.mode)
