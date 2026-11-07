#!/usr/bin/env python3
"""Replay the approved GPUI Base Slider first-drag movement patch."""

import argparse
import hashlib
import os
from pathlib import Path
import subprocess
import tempfile
import tomllib

SOURCE = Path("vendor/gpui-base-0.7.3/src/slider.rs")
PATCH = Path("patches/approved/gpui-base-slider-first-move-v1.patch")
PRE = "90baa9521946b0bcb1bea373319e2b43ed8915fa145dd7f5d2728565c48458a5"
POST = "e22b1a1ffa8c44f65c6b298c761c1ac07e44a031bad6bf4f1787b56623a09e7d"
PATCH_HASH = "855775a4096aa569836d88583f1afa7702768e1c1e06317c6873a857fc205145"


def digest(data):
    return hashlib.sha256(data).hexdigest()


def run(root, artifacts, mode):
    root, artifacts = root.resolve(), artifacts.resolve()
    source, patch = root / SOURCE, artifacts / PATCH
    manifest = tomllib.loads((root / "Cargo.toml").read_text())
    if manifest.get("patch", {}).get("crates-io", {}).get("gpui-base", {}).get("path") != "vendor/gpui-base-0.7.3":
        raise ValueError("the selected GPUI Base package is not the approved local snapshot")
    package = tomllib.loads((root / "vendor/gpui-base-0.7.3/Cargo.toml").read_text())
    if package.get("package", {}).get("version") != "0.7.3":
        raise ValueError("unexpected GPUI Base version")
    if source.is_symlink() or patch.is_symlink():
        raise ValueError("source and patch must be regular files")
    if digest(patch.read_bytes()) != PATCH_HASH:
        raise ValueError("slider patch hash mismatch")
    original = source.read_bytes()
    current = digest(original)
    if current not in (PRE, POST):
        raise ValueError("unknown or partial slider source image")
    if mode == "verify" and current != POST:
        raise ValueError("verification requires the patched slider image")

    with tempfile.TemporaryDirectory(prefix="rustui-slider-patch-") as folder:
        stage = Path(folder)
        target = stage / SOURCE
        target.parent.mkdir(parents=True)
        target.write_bytes(original)
        reverse = current == POST
        command = ["git", "apply", "--whitespace=nowarn"]
        if reverse:
            command.append("--reverse")
        subprocess.run(command + ["--check", str(patch)], cwd=stage, check=True, capture_output=True)
        subprocess.run(command + [str(patch)], cwd=stage, check=True, capture_output=True)
        changed = target.read_bytes()
        if digest(changed) != (PRE if reverse else POST):
            raise ValueError("slider patch image mismatch")
        undo = ["git", "apply", "--whitespace=nowarn"] + ([] if reverse else ["--reverse"])
        subprocess.run(undo + [str(patch)], cwd=stage, check=True, capture_output=True)
        if target.read_bytes() != original:
            raise ValueError("slider patch reversal mismatch")

    if mode in ("apply", "reverse"):
        if (mode == "apply") == reverse:
            raise ValueError("slider source is already in the requested state")
        if source.read_bytes() != original:
            raise ValueError("slider source changed during patch verification")
        fd, temporary = tempfile.mkstemp(prefix=".rustui-slider-", dir=source.parent)
        try:
            os.fchmod(fd, source.stat().st_mode & 0o777)
            with os.fdopen(fd, "wb") as output:
                output.write(changed)
            if source.read_bytes() != original:
                raise ValueError("slider source changed before patch promotion")
            os.replace(temporary, source)
        finally:
            if os.path.exists(temporary):
                os.unlink(temporary)
    print(f"slider first-move patch {mode} verified")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--artifacts", type=Path)
    modes = parser.add_mutually_exclusive_group(required=True)
    for mode in ("check", "apply", "verify", "reverse"):
        modes.add_argument(f"--{mode}", action="store_true")
    args = parser.parse_args()
    mode = next(mode for mode in ("check", "apply", "verify", "reverse") if getattr(args, mode))
    try:
        run(args.root, args.artifacts or args.root, mode)
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"error: {error}\n")


if __name__ == "__main__":
    main()
