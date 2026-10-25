#!/usr/bin/env python3
"""Apply or verify the approved GPUI Web keyboard-focus patch."""

from __future__ import annotations

import argparse
import hashlib
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import tomllib


PATCH = Path("patches/approved/gpui-pre-web-keyboard-focus-activation-v3.patch")
PATCH_SHA256 = "df97bcbf43490e1e60e582da87b5bd63ebc2eb275e662f5c49d622f77b7de59a"
CHECKSUM = Path("vendor/gpui-pre-web-0.3.9/.cargo-checksum.json")
CHECKSUM_SHA256 = "aad9503f554e462d582461d008ea4bc0dda00021812ccf6b876734b55757579e"
FILES = {
    Path("vendor/gpui-pre-web-0.3.9/src/window.rs"): (
        "eda543e585912fcb012e2a3559d8fb2e9ad7570ba58746747af380d044085cb8",
        "3779ba8d5647009b80d0ea52dc7823806640741d3a1cfa9a0a7752ec35fe7dd8",
    ),
    Path("vendor/gpui-pre-web-0.3.9/src/events.rs"): (
        "9fdfc3695a93083fe881153dcc66356df00eb09761190664cc093520bc3ae001",
        "3dca61104bce15bffdd7d9155af769dc166c68508fb6458b4952261b2c3244cb",
    ),
}


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify_identity(root: Path) -> None:
    package = tomllib.loads(
        (root / "vendor/gpui-pre-web-0.3.9/Cargo.toml").read_text(encoding="utf-8")
    )["package"]
    if (package.get("name"), package.get("version")) != ("gpui-pre-web", "0.3.9"):
        raise ValueError("patch requires pinned gpui-pre-web 0.3.9")
    if digest(root / PATCH) != PATCH_SHA256:
        raise ValueError("approved patch SHA-256 mismatch")
    if digest(root / CHECKSUM) != CHECKSUM_SHA256:
        raise ValueError("pristine upstream package checksum metadata changed")
    cargo = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
    if cargo.get("patch", {}).get("crates-io", {}).get("gpui-pre-web") != {
        "path": "vendor/gpui-pre-web-0.3.9"
    }:
        raise ValueError("Cargo.toml must select the pinned local gpui-pre-web package")


def stage_patch(root: Path, reverse: bool) -> dict[Path, bytes]:
    originals = {path: (root / path).read_bytes() for path in FILES}
    with tempfile.TemporaryDirectory(prefix="rustui-gpui-web-keyboard-") as directory:
        stage = Path(directory)
        for relative, content in originals.items():
            target = stage / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(content)
        command = ["patch", "--batch", "--fuzz=0"]
        if reverse:
            command.append("-R")
        command.extend(["-p1", "-i", str((root / PATCH).resolve())])
        result = subprocess.run(command, cwd=stage, capture_output=True, text=True)
        output = result.stdout + result.stderr
        if result.returncode or any(token in output.lower() for token in ("offset", "fuzz", "failed")):
            raise ValueError(f"exact patch application failed: {output.strip()}")
        result_files = {}
        expected_index = 0 if reverse else 1
        for relative in FILES:
            target = stage / relative
            content = target.read_bytes()
            if hashlib.sha256(content).hexdigest() != FILES[relative][expected_index]:
                raise ValueError(f"unexpected patch postimage: {relative}")
            result_files[relative] = content
        return result_files


def install(root: Path, files: dict[Path, bytes]) -> None:
    temporary = {}
    try:
        for relative, content in files.items():
            target = root / relative
            temp = target.with_name(f".{target.name}.keyboard-patch-tmp")
            temp.write_bytes(content)
            temporary[target] = temp
        for target, temp in temporary.items():
            os.replace(temp, target)
    finally:
        for temp in temporary.values():
            temp.unlink(missing_ok=True)


def run(root: Path, reverse: bool, check: bool) -> None:
    root = root.resolve()
    verify_identity(root)
    if check and any(digest(root / relative) != hashes[1] for relative, hashes in FILES.items()):
        result = subprocess.run([sys.executable,str(root / "scripts/apply_gpui_web_semantic_adapter.py"),"--verify-installed","--root",str(root)],capture_output=True,text=True)
        if result.returncode:raise ValueError(f"composed Web verification failed: {result.stdout}{result.stderr}")
        print("approved M855 keyboard baseline and exact composed Web stack verified; checkout unchanged")
        return
    expected_index = 1 if reverse or check else 0
    for relative, hashes in FILES.items():
        if digest(root / relative) != hashes[expected_index]:raise ValueError(f"unexpected source SHA-256: {relative}")
    if check:
        print("approved GPUI Web keyboard patch and pristine checksums verified")
        return
    install(root, stage_patch(root, reverse))
    print(f"approved GPUI Web keyboard patch {'reversed' if reverse else 'applied'}")



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
    except (OSError, ValueError, KeyError, tomllib.TOMLDecodeError, subprocess.SubprocessError) as error:
        parser.exit(1, f"error: {error}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
