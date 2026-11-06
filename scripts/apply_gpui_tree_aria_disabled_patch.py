#!/usr/bin/env python3
"""Apply or reverse the exact paired GPUI Tree aria-disabled patch."""

from __future__ import annotations

import argparse
import hashlib
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import tomllib


PATCH = Path("patches/approved/gpui-tree-aria-disabled-v1.patch")
PATCH_SHA256 = "5762dfec8eba4b723bc1b851c527c75f18e78d76d050fd8bab4485a63ceb14d1"
FILES = {
    "vendor/gpui-pre-0.3.9/src/elements/div.rs": (
        "f320948b8f90e36b7100ac3e0e061235d1e9422aeb323918e313f7d69980cfa9",
        "de8298ce4e48f055984742a18f18206e385786f64ff495f2d5b176481033b84e",
    ),
    "vendor/gpui-base-0.7.3/src/tree.rs": (
        "e74e4fcdef6ccec174e3e6ced50f108433e726f3e3d46f3539b1f41d5944e5df",
        "9ef2efa0a44af810eef3088343b0838179d5e952c6ca8734e5306d5c74736924",
    ),
    "vendor/gpui-base-0.7.3/src/switch.rs": (
        "d06394441975ac7eb4478d353762c527ca0e9d8034c7f5097902d9d976543567",
        "aed20dabde162201be920d297dac451c31b8fc18385c60298f90123c1cae724d",
    ),
}
PROVENANCE = {
    "vendor/gpui-pre-0.3.9/LICENSE-APACHE": "752daf2fb234ca4a1fa372c073fe127f44b7b90fd2529ae44273a64f9d53da7a",
    "vendor/gpui-base-0.7.3/LICENSE-APACHE": "d1b0449e5478c574ba4f686c2656df7fe77d66821a61f8b6ed3378a58ed9a811",
    "vendor/gpui-pre-0.3.9/.cargo-checksum.json": "7b8e889ae4ac0cbe651b654f5e39d9003969d09ae4a70aff4147f521eda8faa0",
    "vendor/gpui-base-0.7.3/.cargo-checksum.json": "9e96d55b0071b60576bc6ac18c3e9c48e6d0de5ff848821af7623bd151f7c3ed",
}
PACKAGES = {
    "vendor/gpui-pre-0.3.9/Cargo.toml": ("gpui-pre", "0.3.9"),
    "vendor/gpui-base-0.7.3/Cargo.toml": ("gpui-base", "0.7.3"),
}


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def verify_identity(root: Path) -> None:
    if sha256(root / PATCH) != PATCH_SHA256:
        raise ValueError("approved patch SHA-256 mismatch")
    for rel, expected in PROVENANCE.items():
        if sha256(root / rel) != expected:
            raise ValueError(f"upstream provenance file changed: {rel}")
    for rel, expected in PACKAGES.items():
        package = tomllib.loads((root / rel).read_text(encoding="utf-8"))["package"]
        if (package.get("name"), package.get("version")) != expected:
            raise ValueError(f"unexpected package identity: {rel}")


def stage_patch(root: Path, reverse: bool) -> dict[str, bytes]:
    with tempfile.TemporaryDirectory(prefix="rustui-gpui-tree-patch-") as temp:
        staged_root = Path(temp)
        for rel in FILES:
            staged = staged_root / rel
            staged.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(root / rel, staged)
        command = ["patch", "--batch", "--fuzz=0", "-p1", "-i", str(root / PATCH)]
        if reverse:
            command.insert(2, "-R")
        result = subprocess.run(command, cwd=staged_root, capture_output=True, text=True, check=False)
        output = result.stdout + result.stderr
        if result.returncode or re.search(r"offset|fuzz|FAILED", output, re.IGNORECASE):
            raise ValueError(f"exact patch application failed ({result.returncode}): {output.strip()}")
        expected_index = 0 if reverse else 1
        staged_files = {rel: (staged_root / rel).read_bytes() for rel in FILES}
        for rel, contents in staged_files.items():
            digest = hashlib.sha256(contents).hexdigest()
            if digest != FILES[rel][expected_index]:
                raise ValueError(f"unexpected staged hash for {rel}: {digest}")
        return staged_files


def install_files(root: Path, staged_files: dict[str, bytes]) -> None:
    originals = {rel: (root / rel).read_bytes() for rel in staged_files}
    temporary: dict[str, Path] = {}
    try:
        for rel, contents in staged_files.items():
            target = root / rel
            temp_path = target.with_name(f".{target.name}.tree-patch-tmp")
            temp_path.write_bytes(contents)
            temporary[rel] = temp_path
        for rel, temp_path in temporary.items():
            os.replace(temp_path, root / rel)
    except OSError:
        for rel, contents in originals.items():
            (root / rel).write_bytes(contents)
        raise
    finally:
        for temp_path in temporary.values():
            temp_path.unlink(missing_ok=True)


def run(root: Path, reverse: bool, check: bool) -> None:
    root = root.resolve()
    verify_identity(root)
    expected_index = 1 if reverse or check else 0
    for rel, hashes in FILES.items():
        actual = sha256(root / rel)
        expected = hashes[expected_index]
        if actual != expected:
            raise ValueError(f"unexpected source hash for {rel}: {actual}")
    if check:
        print("approved Tree aria-disabled patch is applied and verified")
        return
    staged = stage_patch(root, reverse)
    install_files(root, staged)
    mode = "reversed" if reverse else "applied"
    print(f"Tree aria-disabled patch {mode}; all source hashes verified")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--apply", action="store_true")
    mode.add_argument("--reverse", action="store_true")
    mode.add_argument("--check", action="store_true")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    try:
        run(args.root, reverse=args.reverse, check=args.check)
    except (OSError, ValueError, KeyError, tomllib.TOMLDecodeError) as error:
        raise SystemExit(f"error: {error}") from error
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
