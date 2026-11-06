#!/usr/bin/env python3
"""Verify the proposed disabled-Tree keyboard patch on a disposable source copy."""

from __future__ import annotations

import argparse
import hashlib
from pathlib import Path
import re
import shutil
import subprocess
import tempfile


PATCH = Path("patches/proposed/gpui-tree-disabled-keyboard-v1.patch")
PATCH_SHA256 = "6734386f8db13f51f5330a05e3f5c505f59f22218caeff869a800f6fea7dc47b"
SOURCE = Path("vendor/gpui-base-0.7.3/src/tree.rs")
PREIMAGE_SHA256 = "9ef2efa0a44af810eef3088343b0838179d5e952c6ca8734e5306d5c74736924"
POSTIMAGE_SHA256 = "4b1c4d38493f0c26b9b3c08d1baa89372de33b013320426ae8f07ede3be89db6"
REGRESSION = "disabled_folders_ignore_expansion_requests"


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def verify(root: Path) -> None:
    root = root.resolve()
    if sha256(root / PATCH) != PATCH_SHA256:
        raise ValueError("proposed patch SHA-256 mismatch")
    source = root / SOURCE
    if sha256(source) != PREIMAGE_SHA256:
        raise ValueError("Tree source is not the expected M269 postimage")

    with tempfile.TemporaryDirectory(prefix="rustui-tree-disabled-keyboard-") as temp:
        stage = Path(temp)
        staged_source = stage / SOURCE
        staged_source.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, staged_source)
        result = subprocess.run(
            ["patch", "--batch", "--fuzz=0", "-p1", "-i", str(root / PATCH)],
            cwd=stage,
            capture_output=True,
            text=True,
            check=False,
        )
        output = result.stdout + result.stderr
        if result.returncode or re.search(r"offset|fuzz|FAILED", output, re.IGNORECASE):
            raise ValueError(f"isolated patch application failed: {output.strip()}")
        if sha256(staged_source) != POSTIMAGE_SHA256:
            raise ValueError("unexpected isolated Tree postimage hash")
        contents = staged_source.read_text(encoding="utf-8")
        if REGRESSION not in contents:
            raise ValueError("focused disabled-folder regression is missing")
        formatting = subprocess.run(
            ["rustfmt", "--check", "--edition=2024", str(staged_source)],
            capture_output=True,
            text=True,
            check=False,
        )
        if formatting.returncode:
            raise ValueError(f"isolated Rust formatting check failed: {formatting.stdout}{formatting.stderr}")

    print("proposed Tree disabled-keyboard patch applies exactly to an isolated copy")
    print(f"preimage:  {PREIMAGE_SHA256}")
    print(f"postimage: {POSTIMAGE_SHA256}")
    print(f"regression: {REGRESSION}")
    print("rustfmt: passed on isolated postimage")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    try:
        verify(args.root)
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        raise SystemExit(f"error: {error}") from error
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
