#!/usr/bin/env python3
"""Check, apply, or reverse the proposed gpui-base Tree aria-level patch."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import tomllib


PATCH = Path("patches/proposed/gpui-tree-aria-level-v1.patch")
PATCH_SHA256 = "67324d2ff67e53c6bc62b7a41609957bca559f0f70e0964fcec17de2ca48f57b"
TREE = Path("vendor/gpui-base-0.7.3/src/tree.rs")
TREE_PREIMAGE_SHA256 = "9ef2efa0a44af810eef3088343b0838179d5e952c6ca8734e5306d5c74736924"
TREE_POSTIMAGE_SHA256 = "4dae8754abef34b1173e0b69b4dfb80483ae61545e2dd3614fc259dfd38fe6fb"
REGISTRY_CHECKSUM = "2e9955b6f4b048fd50ca15fc8e225d80bb00ba9ebc8fecdddde5ad26dbef9270"
PRISTINE_TREE_SHA256 = "e74e4fcdef6ccec174e3e6ced50f108433e726f3e3d46f3539b1f41d5944e5df"
PROVENANCE = {
    Path("vendor/gpui-base-0.7.3/LICENSE-APACHE"): "d1b0449e5478c574ba4f686c2656df7fe77d66821a61f8b6ed3378a58ed9a811",
    Path("vendor/gpui-base-0.7.3/.cargo-checksum.json"): "9e96d55b0071b60576bc6ac18c3e9c48e6d0de5ff848821af7623bd151f7c3ed",
}


def sha256_bytes(contents: bytes) -> str:
    return hashlib.sha256(contents).hexdigest()


def sha256_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def verify_identity(root: Path) -> None:
    patch_path = root / PATCH
    if sha256_file(patch_path) != PATCH_SHA256:
        raise ValueError("proposed patch SHA-256 mismatch")

    package = tomllib.loads(
        (root / "vendor/gpui-base-0.7.3/Cargo.toml").read_text(encoding="utf-8")
    )["package"]
    if (package.get("name"), package.get("version")) != ("gpui-base", "0.7.3"):
        raise ValueError("unexpected gpui-base package identity")

    for relative, expected in PROVENANCE.items():
        if sha256_file(root / relative) != expected:
            raise ValueError(f"upstream provenance file changed: {relative}")

    checksum = json.loads(
        (root / "vendor/gpui-base-0.7.3/.cargo-checksum.json").read_text(encoding="utf-8")
    )
    if checksum.get("package") != REGISTRY_CHECKSUM:
        raise ValueError("unexpected pristine gpui-base registry checksum")
    if checksum.get("files", {}).get("src/tree.rs") != PRISTINE_TREE_SHA256:
        raise ValueError("pristine Tree source checksum metadata changed")


def transform(root: Path, source: bytes, reverse: bool) -> bytes:
    with tempfile.TemporaryDirectory(prefix="rustui-gpui-tree-aria-level-") as directory:
        staged_root = Path(directory)
        staged_tree = staged_root / TREE
        staged_tree.parent.mkdir(parents=True, exist_ok=True)
        staged_tree.write_bytes(source)

        command = ["patch", "--batch", "--fuzz=0", "-p1", "-i", str(root / PATCH)]
        if reverse:
            command.insert(2, "-R")
        result = subprocess.run(
            command, cwd=staged_root, capture_output=True, text=True, check=False
        )
        output = result.stdout + result.stderr
        if result.returncode or re.search(r"offset|fuzz|FAILED", output, re.IGNORECASE):
            raise ValueError(f"exact patch operation failed ({result.returncode}): {output.strip()}")
        return staged_tree.read_bytes()


def install(root: Path, contents: bytes) -> None:
    target = root / TREE
    temporary: Path | None = None
    try:
        with tempfile.NamedTemporaryFile(
            dir=target.parent, prefix=".tree-aria-level-", delete=False
        ) as stream:
            temporary = Path(stream.name)
            stream.write(contents)
        os.replace(temporary, target)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def run(root: Path, mode: str) -> None:
    root = root.resolve()
    verify_identity(root)
    current = (root / TREE).read_bytes()
    current_hash = sha256_bytes(current)

    if mode == "--check":
        if current_hash != TREE_PREIMAGE_SHA256:
            raise ValueError(f"proposal requires the exact preimage; found {current_hash}")
        proposed = transform(root, current, reverse=False)
        if sha256_bytes(proposed) != TREE_POSTIMAGE_SHA256:
            raise ValueError("unexpected proposed Tree postimage hash")
        restored = transform(root, proposed, reverse=True)
        if sha256_bytes(restored) != TREE_PREIMAGE_SHA256:
            raise ValueError("reverse check did not restore the exact Tree preimage")
        print("proposed Tree aria-level patch applies and reverses with exact hashes; checkout unchanged")
        return

    reverse = mode == "--reverse"
    expected = TREE_POSTIMAGE_SHA256 if reverse else TREE_PREIMAGE_SHA256
    target_hash = TREE_PREIMAGE_SHA256 if reverse else TREE_POSTIMAGE_SHA256
    if current_hash != expected:
        raise ValueError(f"unexpected Tree source hash: {current_hash}")
    updated = transform(root, current, reverse=reverse)
    if sha256_bytes(updated) != target_hash:
        raise ValueError("unexpected transformed Tree source hash")
    install(root, updated)
    print(f"Tree aria-level patch {'reversed' if reverse else 'applied'}; source hash verified")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--apply", action="store_true")
    mode.add_argument("--reverse", action="store_true")
    mode.add_argument("--check", action="store_true")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    selected = "--apply" if args.apply else "--reverse" if args.reverse else "--check"
    try:
        run(args.root, selected)
    except (OSError, ValueError, KeyError, json.JSONDecodeError, tomllib.TOMLDecodeError) as error:
        raise SystemExit(f"error: {error}") from error
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
