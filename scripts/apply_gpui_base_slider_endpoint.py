#!/usr/bin/env python3
"""Replay the approved GPUI Base Slider stepped endpoint range patch."""

import argparse
import hashlib
import os
from pathlib import Path
import subprocess
import tempfile
import tomllib

SOURCE = Path("vendor/gpui-base-0.7.3/src/slider.rs")
PATCH = Path("patches/approved/gpui-base-slider-endpoint-v1.patch")
PRE = "45ed0395f566732a6bfb4d0327560d1d33b9f24116cf8c08e1ab702d4eff3bf0"
POST = "f9c3b88ae5cda1ea65499fc1d6ddb198584cf1831b92336eea2c2566bbb15f1f"
PATCH_HASH = "d46f0feb3d0a4d8c376c889fc66d709675528cdf09b96a2309d0a1bd7869f8f2"


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
        command = ["git", "apply", "--unidiff-zero", "--whitespace=nowarn"]
        if reverse:
            command.append("--reverse")
        subprocess.run(command + ["--check", str(patch)], cwd=stage, check=True, capture_output=True)
        subprocess.run(command + [str(patch)], cwd=stage, check=True, capture_output=True)
        changed = target.read_bytes()
        if digest(changed) != (PRE if reverse else POST):
            raise ValueError("slider patch image mismatch")
        undo = ["git", "apply", "--unidiff-zero", "--whitespace=nowarn"] + ([] if reverse else ["--reverse"])
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
    print(f"slider endpoint patch {mode} verified")


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
