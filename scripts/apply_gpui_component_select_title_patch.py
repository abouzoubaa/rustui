#!/usr/bin/env python3
"""Apply or verify RustUI's exact gpui-component select committed-title patch."""

from __future__ import annotations

import argparse
import hashlib
from pathlib import Path
import re
import subprocess
import tomllib


PATCH = Path("patches/approved/gpui-component-select-committed-title-v1.patch")
PACKAGE = Path("vendor/gpui-component-0.7.3")
SOURCE = PACKAGE / "src/select.rs"
CHECKSUM_FILE = PACKAGE / ".cargo-checksum.json"
PATCH_SHA256 = "422921c1eccc8940abb941af6e890b1c7a759996bbaae57d9ff4819c47126631"
PREIMAGE_SHA256 = "a029028c500500d1df071c9b44a13cc05158515010bb92f234ef4c6fddba9f48"
POSTIMAGE_SHA256 = "1a1559e6b46cfb562b565d85600bef7731c3015ae5354d0dcc10454e3bc51efe"
COMPOSED_RTL_SHA256 = "aef4bdbc7dbeab626b1091be47ddfb1b330028b7904daa75173914fbb2a84a8e"
CHECKSUM_SHA256 = "d5b533efba197cc15bcb6756b935a786e846463c1ee431f876e8c334a7e638c6"


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def verify_identity(root: Path) -> tuple[Path, Path, Path]:
    package, source, checksum = root / PACKAGE, root / SOURCE, root / CHECKSUM_FILE
    metadata = tomllib.loads((package / "Cargo.toml").read_text(encoding="utf-8"))["package"]
    if metadata.get("name") != "gpui-component" or metadata.get("version") != "0.7.3":
        raise ValueError("patch requires gpui-component 0.7.3")
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
        if actual not in (POSTIMAGE_SHA256, COMPOSED_RTL_SHA256):
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
