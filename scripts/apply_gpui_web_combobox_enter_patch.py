#!/usr/bin/env python3
"""Replay the approved gpui-pre-web combobox Enter routing patch with pinned inputs."""
import argparse
import hashlib
import os
from pathlib import Path
import subprocess
import tempfile
import tomllib

SOURCE = Path("vendor/gpui-pre-web-0.3.9/src/a11y_dom.js")
PATCH = Path("patches/approved/gpui-web-combobox-enter-v1.patch")
PRE = "31cc24e332db33a84630bc24c4210eb745bbb09d787938bede416aa46b9cc834"
POST = "aae5743b4bdedd3b3432c33c8b435d9889cb40cc830119580b9c8d3e8506c0f5"
PATCH_HASH = "47325e55bb3c42b26b8ec7d10e1ffad726fa1661e16f7be74211193827fcc550"
PRESERVED = {
    "vendor/gpui-pre-web-0.3.9/Cargo.toml": "f51c51c22a0b75567b66c2a8078d14612545ecbeef002ca9ec1e07799bd198ff",
    "vendor/gpui-pre-web-0.3.9/.cargo-checksum.json": "aad9503f554e462d582461d008ea4bc0dda00021812ccf6b876734b55757579e",
}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def run(root, artifacts, mode):
    root, artifacts = root.resolve(), artifacts.resolve()
    package = tomllib.loads((root / "vendor/gpui-pre-web-0.3.9/Cargo.toml").read_text())["package"]
    selected = tomllib.loads((root / "Cargo.toml").read_text())["patch"]["crates-io"]["gpui-pre-web"]
    if (package.get("name"), package.get("version"), selected.get("path")) != ("gpui-pre-web", "0.3.9", "vendor/gpui-pre-web-0.3.9"):
        raise ValueError("pinned gpui-pre-web package selection mismatch")
    for path, expected in PRESERVED.items():
        if digest((root / path).read_bytes()) != expected:
            raise ValueError(f"upstream metadata changed: {path}")
    patch = artifacts / PATCH
    if digest(patch.read_bytes()) != PATCH_HASH:
        raise ValueError("Web combobox Enter patch hash mismatch")
    source = root / SOURCE
    if source.is_symlink():
        raise ValueError("Web semantic source must not be a symlink")
    old = source.read_bytes()
    current = digest(old)
    if current not in (PRE, POST):
        raise ValueError("Web semantic source is not the known preimage or postimage")
    reverse = current == POST
    if mode == "--verify" and not reverse:
        raise ValueError("Web combobox Enter patch postimage required")
    with tempfile.TemporaryDirectory(prefix="rustui-web-combobox-patch-") as folder:
        stage = Path(folder)
        staged = stage / SOURCE
        staged.parent.mkdir(parents=True)
        staged.write_bytes(old)
        command = ["git", "apply", "--whitespace=nowarn"] + (["--reverse"] if reverse else [])
        subprocess.run(command + ["--check", str(patch)], cwd=stage, check=True, capture_output=True)
        subprocess.run(command + [str(patch)], cwd=stage, check=True, capture_output=True)
        new = staged.read_bytes()
        if digest(new) != (PRE if reverse else POST):
            raise ValueError("Web combobox Enter patch replay hash mismatch")
        undo = ["git", "apply", "--whitespace=nowarn"] + ([] if reverse else ["--reverse"])
        subprocess.run(undo + [str(patch)], cwd=stage, check=True, capture_output=True)
        if staged.read_bytes() != old:
            raise ValueError("Web combobox Enter patch reversal mismatch")
    if mode in ("--apply", "--reverse"):
        if (mode == "--apply") == reverse:
            raise ValueError("Web semantic source already has the requested image")
        if source.read_bytes() != old:
            raise ValueError("Web semantic source changed during staging")
        fd, name = tempfile.mkstemp(prefix=".rustui-web-combobox-", dir=source.parent)
        try:
            os.fchmod(fd, source.stat().st_mode & 0o777)
            with os.fdopen(fd, "wb") as output:
                output.write(new)
            if source.read_bytes() != old:
                raise ValueError("Web semantic source changed during staging")
            os.replace(name, source)
        finally:
            if os.path.exists(name):
                os.unlink(name)
    print("Web combobox Enter patch replay verified")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    for option in ("--verify", "--apply", "--reverse"):
        group.add_argument(option, action="store_const", dest="mode", const=option)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--artifacts", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    run(args.root, args.artifacts, args.mode)
