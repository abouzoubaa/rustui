#!/usr/bin/env python3
"""Replay G10 patches against pinned vendor and owned images together."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile


def digest(data):
    return hashlib.sha256(data).hexdigest()


def load(artifacts, manifest, manifest_sha256):
    raw = (artifacts / manifest).read_bytes()
    if digest(raw) != manifest_sha256:
        raise ValueError("approved manifest hash mismatch")
    meta = json.loads(raw)
    patch = artifacts / meta["patch"]
    if digest(patch.read_bytes()) != meta["patch_sha256"]:
        raise ValueError("approved patch hash mismatch")
    files = meta["files"]
    source = meta["source"]
    mirror = source.replace("vendor/gpui-pre-0.3.9/", "crates/rustui-gpui/", 1)
    if set(files) != {source, mirror} or source == mirror:
        raise ValueError("manifest must name exactly the vendor source and owned mirror")
    expected = {"pre": meta["pre_sha256"], "post": meta["post_sha256"]}
    if any(hashes != expected for hashes in files.values()):
        raise ValueError("vendor and owned image hashes must match")
    touched = subprocess.check_output(["git", "apply", "--numstat", str(patch)], text=True)
    if {line.split("\t", 2)[2] for line in touched.splitlines()} != set(files):
        raise ValueError("patch paths differ from manifest")
    return meta, patch


def run(root, artifacts, manifest, manifest_sha256, mode):
    root, artifacts = root.resolve(), artifacts.resolve()
    meta, patch = load(artifacts, manifest, manifest_sha256)
    before = {}
    for relative, hashes in meta["files"].items():
        source = root / relative
        if source.is_symlink() or not source.is_file():
            raise ValueError(f"source must be a regular file: {relative}")
        before[relative] = source.read_bytes()
        if digest(before[relative]) not in hashes.values():
            raise ValueError(f"source differs from the approved preimage and postimage: {relative}")
    states = {"pre" if digest(data) == meta["files"][name]["pre"] else "post" for name, data in before.items()}
    if len(states) != 1:
        raise ValueError("vendor and owned mirror are in different patch states")
    state = states.pop()
    if mode == "--verify" and state != "post":
        raise ValueError("approved postimage required")
    if mode in ("--apply", "--reverse") and (mode == "--apply") == (state == "post"):
        raise ValueError("sources are already in the requested state")
    with tempfile.TemporaryDirectory(prefix="g10-paired-replay-") as temporary:
        stage = Path(temporary)
        for relative, data in before.items():
            source = stage / relative
            source.parent.mkdir(parents=True)
            source.write_bytes(data)
        command = ["git", "apply", "--whitespace=nowarn"]
        forward = command + (["--reverse"] if state == "post" else [])
        subprocess.run(forward + ["--check", str(patch)], cwd=stage, check=True, capture_output=True)
        subprocess.run(forward + [str(patch)], cwd=stage, check=True, capture_output=True)
        after = {relative: (stage / relative).read_bytes() for relative in before}
        target = "pre" if state == "post" else "post"
        if any(digest(data) != meta["files"][name][target] for name, data in after.items()):
            raise ValueError("patch replay produced an unexpected source image")
        inverse = command + ([] if state == "post" else ["--reverse"])
        subprocess.run(inverse + [str(patch)], cwd=stage, check=True, capture_output=True)
        if any((stage / name).read_bytes() != data for name, data in before.items()):
            raise ValueError("patch reverse replay failed")
    if mode in ("--apply", "--reverse"):
        # Validate the complete pair before replacing either source.
        if any((root / name).read_bytes() != data for name, data in before.items()):
            raise ValueError("sources changed during patch replay")
        pending = []
        try:
            for relative, data in after.items():
                source = root / relative
                fd, temporary = tempfile.mkstemp(prefix=".g10-patch-", dir=source.parent)
                pending.append((temporary, source))
                os.fchmod(fd, source.stat().st_mode & 0o777)
                with os.fdopen(fd, "wb") as out:
                    out.write(data)
            for temporary, source in pending:
                os.replace(temporary, source)
        finally:
            for temporary, _ in pending:
                if os.path.exists(temporary):
                    os.unlink(temporary)
    print(f"Approved paired patch {mode}: {', '.join(before)}")


def vendor_prior_image(root, artifacts, manifest, manifest_sha256):
    """Read a prior vendor layer for historical stack proofs; never mutate root."""
    root, artifacts = Path(root), Path(artifacts)
    meta, _ = load(artifacts, manifest, manifest_sha256)
    data = (root / meta["source"]).read_bytes()
    with tempfile.TemporaryDirectory(prefix="g10-prior-image-") as temporary:
        stage = Path(temporary)
        for relative in meta["files"]:
            source = stage / relative
            source.parent.mkdir(parents=True)
            source.write_bytes(data)
        mode = "--reverse" if digest(data) == meta["post_sha256"] else "--check"
        run(stage, artifacts, manifest, manifest_sha256, mode)
        return (stage / meta["source"]).read_bytes()


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
