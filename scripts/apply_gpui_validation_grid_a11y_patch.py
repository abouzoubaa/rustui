#!/usr/bin/env python3
"""Replay the approved validation grid accessibility recovery patch against exact vendor images."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

MANIFEST = Path("patches/approved/a11y-validation-grid-v1.json")
MANIFEST_SHA256 = "c016d34e367114cee36498a240ca47ee139ad9fe9b649579688f81d7b2916aff"


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def install(path: Path, data: bytes) -> None:
    fd, name = tempfile.mkstemp(prefix=".validation-grid-accessibility-", dir=path.parent)
    try:
        os.fchmod(fd, path.stat().st_mode & 0o777)
        with os.fdopen(fd, "wb") as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(name, path)
    finally:
        if os.path.exists(name):
            os.unlink(name)


def run(root: Path, artifacts: Path, mode: str) -> None:
    root, artifacts = root.resolve(strict=True), artifacts.resolve(strict=True)
    manifest_bytes = (artifacts / MANIFEST).read_bytes()
    if digest(manifest_bytes) != MANIFEST_SHA256:
        raise ValueError("approved validation grid accessibility manifest hash mismatch")
    meta = json.loads(manifest_bytes)
    patch = artifacts / meta["patch"]
    if digest(patch.read_bytes()) != meta["patch_sha256"]:
        raise ValueError("approved validation grid accessibility patch hash mismatch")
    files = meta["files"]
    before = {}
    states = set()
    for relative, images in files.items():
        path = root / relative
        if path.is_symlink() or not path.is_file():
            raise ValueError(f"missing or symlinked source: {relative}")
        before[relative] = path.read_bytes()
        current = digest(before[relative])
        if current == images["pre"]:
            states.add("pre")
        elif current == images["post"]:
            states.add("post")
        else:
            raise ValueError(f"unknown source image: {relative}")
    if len(states) != 1:
        raise ValueError("partial validation grid accessibility patch state")
    state = states.pop()
    if mode == "--verify" and state != "post":
        raise ValueError("validation grid accessibility patch POST required")
    if mode == "--apply" and state != "pre":
        raise ValueError("validation grid accessibility patch already applied")
    if mode == "--reverse" and state != "post":
        raise ValueError("validation grid accessibility patch already reversed")
    reverse = state == "post"
    with tempfile.TemporaryDirectory(prefix="rustui-validation-grid-accessibility-") as folder:
        stage = Path(folder)
        for relative, data in before.items():
            path = stage / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        command = ["git", "apply", "--whitespace=nowarn"]
        if reverse:
            command.append("--reverse")
        subprocess.run(command + ["--check", str(patch)], cwd=stage, check=True, capture_output=True)
        subprocess.run(command + [str(patch)], cwd=stage, check=True, capture_output=True)
        after = {relative: (stage / relative).read_bytes() for relative in files}
        target = "pre" if reverse else "post"
        for relative, data in after.items():
            if digest(data) != files[relative][target]:
                raise ValueError(f"patch replay hash mismatch: {relative}")
        undo = ["git", "apply", "--whitespace=nowarn"]
        if not reverse:
            undo.append("--reverse")
        subprocess.run(undo + [str(patch)], cwd=stage, check=True, capture_output=True)
        if any((stage / relative).read_bytes() != data for relative, data in before.items()):
            raise ValueError("validation grid accessibility patch reverse replay mismatch")
    if mode in ("--apply", "--reverse"):
        if any((root / relative).read_bytes() != data for relative, data in before.items()):
            raise ValueError("source changed during patch staging")
        installed = []
        try:
            for relative, data in after.items():
                install(root / relative, data)
                installed.append(relative)
        except BaseException:
            for relative in reversed(installed):
                install(root / relative, before[relative])
            raise
    print(f"Approved validation grid accessibility patch exact source and replay verified ({target if mode in ('--apply', '--reverse') else state})")



def prior_source(root: Path, artifacts: Path, relative: str) -> bytes:
    """Verify the new layer and recover one prior image in a temporary tree."""
    import contextlib
    import io
    meta_bytes = (artifacts / MANIFEST).read_bytes()
    if digest(meta_bytes) != MANIFEST_SHA256:
        raise ValueError("accessibility stack manifest hash mismatch")
    meta = json.loads(meta_bytes)
    with tempfile.TemporaryDirectory(prefix="rustui-a11y-previous-") as folder:
        stage = Path(folder)
        for path in meta["files"]:
            target = stage / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes((root / path).read_bytes())
        with contextlib.redirect_stdout(io.StringIO()):
            run(stage, artifacts, "--reverse")
        return (stage / relative).read_bytes()

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--artifacts", type=Path)
    group = parser.add_mutually_exclusive_group(required=True)
    for mode in ("check", "verify", "apply", "reverse"):
        group.add_argument(f"--{mode}", action="store_true")
    args = parser.parse_args()
    mode = next(f"--{name}" for name in ("check", "verify", "apply", "reverse") if getattr(args, name))
    try:
        run(args.root, args.artifacts or args.root, mode)
    except (ValueError, OSError, subprocess.CalledProcessError, RuntimeError) as error:
        parser.exit(1, f"error: {error}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
