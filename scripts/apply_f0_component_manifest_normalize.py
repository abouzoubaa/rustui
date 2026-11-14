#!/usr/bin/env python3
"""Replay the Fast candidate's exact GPUI Component manifest normalization."""
import hashlib
import json
import argparse
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "patches/candidate-kit-fast/gpui-component-vendor-manifest-normalize-v1.json"


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--reverse", action="store_true")
    args = parser.parse_args()
    selected = sum((args.check, args.apply, args.reverse))
    if selected != 1:
        parser.error("choose exactly one of --check, --apply, or --reverse")
    root = args.root.resolve()
    action = "--check" if args.check else "--apply" if args.apply else "--reverse"
    meta = json.loads(MANIFEST.read_text())
    patch = ROOT / "patches/candidate-kit-fast" / meta["patch"]
    if digest(patch) != meta["patch_sha256"]:
        raise RuntimeError("candidate patch hash mismatch")
    reverse = action == "--reverse"
    if action not in {"--check", "--apply", "--reverse"}:
        raise RuntimeError("expected --check, --apply, or --reverse")
    for name, hashes in meta["files"].items():
        expected = hashes["post_sha256"] if action == "--check" or reverse else hashes["pre_sha256"]
        if digest(root / name) != expected:
            raise RuntimeError(f"unexpected source image: {name}")
    if action != "--check":
        command = ["git", "apply"]
        if reverse:
            command.append("--reverse")
        subprocess.run([*command, "--check", str(patch)], cwd=root, check=True)
        subprocess.run([*command, str(patch)], cwd=root, check=True)
        result = "pre_sha256" if reverse else "post_sha256"
        for name, hashes in meta["files"].items():
            if digest(root / name) != hashes[result]:
                raise RuntimeError(f"patch produced an unexpected image: {name}")
    print(f"GPUI Component manifest normalization {action}: PASS")


if __name__ == "__main__":
    main()
