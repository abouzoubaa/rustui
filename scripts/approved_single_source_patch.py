#!/usr/bin/env python3
"""Replay an approved single-source patch against exact pinned images."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile


def digest(data):
    return hashlib.sha256(data).hexdigest()


def run(root, artifacts, manifest, manifest_sha256, mode):
    root, artifacts = root.resolve(), artifacts.resolve()
    raw = (artifacts / manifest).read_bytes()
    if digest(raw) != manifest_sha256:
        raise ValueError("approved manifest hash mismatch")
    meta = json.loads(raw)
    patch = artifacts / meta["patch"]
    if digest(patch.read_bytes()) != meta["patch_sha256"]:
        raise ValueError("approved patch hash mismatch")
    source = root / meta["source"]
    if source.is_symlink() or not source.is_file():
        raise ValueError("source must be a regular file")
    before = source.read_bytes()
    current = digest(before)
    pre, post = meta["pre_sha256"], meta["post_sha256"]
    if current not in (pre, post):
        raise ValueError("source differs from the approved preimage and postimage")
    if mode == "--verify" and current != post:
        raise ValueError("approved postimage required")
    reverse = current == post
    with tempfile.TemporaryDirectory(prefix="approved-source-patch-") as tmp:
        stage = Path(tmp)
        staged = stage / meta["source"]
        staged.parent.mkdir(parents=True)
        staged.write_bytes(before)
        command = ["git", "apply", "--whitespace=nowarn"]
        if reverse:
            command.append("--reverse")
        subprocess.run(command + ["--check", str(patch)], cwd=stage, check=True, capture_output=True)
        subprocess.run(command + [str(patch)], cwd=stage, check=True, capture_output=True)
        after = staged.read_bytes()
        if digest(after) != (pre if reverse else post):
            raise ValueError("patch replay produced an unexpected source image")
        inverse = ["git", "apply", "--whitespace=nowarn"]
        if not reverse:
            inverse.append("--reverse")
        subprocess.run(inverse + [str(patch)], cwd=stage, check=True, capture_output=True)
        if staged.read_bytes() != before:
            raise ValueError("patch reverse replay failed")
    if mode in ("--apply", "--reverse"):
        if (mode == "--apply") == reverse:
            raise ValueError("source is already in the requested state")
        fd, temporary = tempfile.mkstemp(prefix=".approved-patch-", dir=source.parent)
        try:
            os.fchmod(fd, source.stat().st_mode & 0o777)
            with os.fdopen(fd, "wb") as out:
                out.write(after)
            if source.read_bytes() != before:
                raise ValueError("source changed during patch replay")
            os.replace(temporary, source)
        finally:
            if os.path.exists(temporary):
                os.unlink(temporary)
    print(f"Approved source patch {mode}: {meta['source']}")


def main(manifest, manifest_sha256):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--artifacts", type=Path)
    modes = parser.add_mutually_exclusive_group(required=True)
    for mode in ("check", "verify", "apply", "reverse"):
        modes.add_argument("--" + mode, action="store_true")
    args = parser.parse_args()
    mode = next("--" + name for name in ("check", "verify", "apply", "reverse") if getattr(args, name))
    try:
        run(args.root, args.artifacts or args.root, manifest, manifest_sha256, mode)
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"error: {error}\n")
