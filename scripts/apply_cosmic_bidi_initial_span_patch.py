#!/usr/bin/env python3
"""Apply or verify RustUI's exact cosmic-text initial bidi span patch."""

from __future__ import annotations

import argparse
import hashlib
from pathlib import Path
import re
import subprocess
import tomllib


PATCH = Path("patches/approved/cosmic-text-initial-bidi-span-v1.patch")
PACKAGE = Path("vendor/cosmic-text-0.19.0")
SOURCE = PACKAGE / "src/shape.rs"
CHECKSUM_FILE = PACKAGE / ".cargo-checksum.json"
PATCH_SHA256 = "25e91c7a89f5963c4e93b8203bdf4a20025248e6d40f2858f9695599eb7e07ce"
PREIMAGE_SHA256 = "d94983d9b811c8f7f8b3d3dd8e2dc118d260c28eca829aba9af036ee68582837"
POSTIMAGE_SHA256 = "6d183b0243467f7033d32ae61d7bba44c34da9fa4b3cecf514962e5207dc681b"
CHECKSUM_SHA256 = "bc591ed045e132d9d1cea4c3c6e0957191a1634ea5abf8690227072e39ac1b8f"


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def verify_identity(root: Path) -> tuple[Path, Path, Path]:
    package, source, checksum = root / PACKAGE, root / SOURCE, root / CHECKSUM_FILE
    metadata = tomllib.loads((package / "Cargo.toml").read_text(encoding="utf-8"))["package"]
    if metadata.get("name") != "cosmic-text" or metadata.get("version") != "0.19.0":
        raise ValueError("patch requires cosmic-text 0.19.0")
    if sha256(root / PATCH) != PATCH_SHA256:
        raise ValueError("approved patch SHA-256 mismatch")
    if sha256(checksum) != CHECKSUM_SHA256:
        raise ValueError("pristine upstream checksum metadata changed")
    return package, source, checksum


def apply(root: Path, reverse: bool = False) -> str:
    _, source, _ = verify_identity(root)
    actual = sha256(source)
    expected = POSTIMAGE_SHA256 if reverse else PREIMAGE_SHA256
    result_hash = PREIMAGE_SHA256 if reverse else POSTIMAGE_SHA256
    if actual == result_hash:
        return f"already {'reverted' if reverse else 'applied'}: {actual}"
    if actual != expected:
        raise ValueError(f"unexpected source SHA-256: {actual}")

    command = ["patch", "--batch", "--fuzz=0", "-p1", "-i", str(root / PATCH)]
    if reverse:
        command.insert(2, "-R")
    result = subprocess.run(command, cwd=root, capture_output=True, text=True, check=False)
    output = result.stdout + result.stderr
    if result.returncode != 0 or re.search(r"offset|fuzz|FAILED", output, re.IGNORECASE):
        raise ValueError(f"exact patch application failed ({result.returncode}): {output}")
    actual = sha256(source)
    if actual != result_hash:
        raise ValueError(f"unexpected post-application SHA-256: {actual}")
    return f"{'reverted' if reverse else 'applied'}: {actual}\n{output}".rstrip()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    action = parser.add_mutually_exclusive_group(required=True)
    action.add_argument("--apply", action="store_true")
    action.add_argument("--reverse", action="store_true")
    action.add_argument("--check", action="store_true")
    args = parser.parse_args()
    root = args.root.resolve()
    if args.check:
        _, source, _ = verify_identity(root)
        actual = sha256(source)
        if actual != POSTIMAGE_SHA256:
            raise ValueError(f"patched source SHA-256 mismatch: {actual}")
        print(f"verified patched source: {actual}")
        return 0
    print(apply(root, reverse=args.reverse))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, KeyError, tomllib.TOMLDecodeError) as error:
        raise SystemExit(f"error: {error}") from error
