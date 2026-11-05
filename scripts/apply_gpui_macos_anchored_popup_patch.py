#!/usr/bin/env python3
"""Apply approved v4 native anchored-popup patch to the pinned macOS backend."""

from __future__ import annotations

import argparse
import hashlib
import os
import shutil
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PATCH = ROOT / "patches/approved/gpui-macos-anchored-popup-v4.patch"
PATCH_SHA256 = "d83e5fdfa57c0eeaa104b00d65ea717b3f5ca7d6e117cd2ac79f427ebd11febc"
PREIMAGES = {
    "vendor/gpui-pre-macos-0.3.9/src/platform.rs": "1bf0adb67969e7389063de4d54efa70e9bb0a7ac7331a74ba74245d3bb87d801",
    "vendor/gpui-pre-macos-0.3.9/src/window.rs": "12d18266972087f25e876a97751af1903401ad2f9b8d9c84eab7fad9495fbd00",
}
POSTIMAGES = {
    "vendor/gpui-pre-macos-0.3.9/src/platform.rs": "6f7b88ef87d757c022a505f5b33104e727bd4e8e4ebdc6b7f3c9a5e0823c8aa6",
    "vendor/gpui-pre-macos-0.3.9/src/window.rs": "ab826e99056215713a220895f9917e5b2e39af4b4d49ccdd0fd341bb7c60daaa",
}


class PatchError(ValueError):
    pass


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _state(root: Path) -> dict[str, str]:
    return {relative: sha256(root / relative) for relative in PREIMAGES}


def _pinned_preimages() -> dict[str, bytes]:
    preimages = {}
    for relative, expected_hash in PREIMAGES.items():
        content = subprocess.check_output(
            ["git", "show", f"HEAD:{relative}"], cwd=ROOT
        )
        actual_hash = hashlib.sha256(content).hexdigest()
        if actual_hash != expected_hash:
            raise PatchError(f"pinned source preimage differs from the approved hash: {relative}")
        preimages[relative] = content
    return preimages


def _apply_in_temporary_root(root: Path, *, reverse: bool) -> tuple[Path, tempfile.TemporaryDirectory]:
    temp = tempfile.TemporaryDirectory(prefix="rustui-gpui-macos-popup-")
    working = Path(temp.name)
    for relative in PREIMAGES:
        target = working / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(root / relative, target)
    args = ["patch", "-p1", "--input", str(PATCH)]
    if reverse:
        args.insert(1, "--reverse")
    try:
        subprocess.run(args, cwd=working, check=True, capture_output=True, text=True)
    except subprocess.CalledProcessError as error:
        temp.cleanup()
        raise PatchError(error.stderr.strip() or error.stdout.strip()) from error
    return working, temp


def apply_patch(root: Path = ROOT, *, reverse: bool = False, check_only: bool = False) -> str:
    if not PATCH.is_file() or sha256(PATCH) != PATCH_SHA256:
        raise PatchError("popup patch is missing or its hash differs from the approved patch")
    expected = POSTIMAGES if reverse else PREIMAGES
    result = PREIMAGES if reverse else POSTIMAGES
    current = _state(root)
    if current == result:
        return "already-reversed" if reverse else "already-applied"
    if current != expected:
        raise PatchError(f"backend sources are not the known {'postimage' if reverse else 'preimage'}: {current}")

    if reverse:
        preimages = _pinned_preimages()
        if check_only:
            return "verified"
        for relative, content in preimages.items():
            destination = root / relative
            with tempfile.NamedTemporaryFile(dir=destination.parent, delete=False) as output:
                temporary = Path(output.name)
                output.write(content)
            os.chmod(temporary, destination.stat().st_mode)
            os.replace(temporary, destination)
        if _state(root) != PREIMAGES:
            raise PatchError("pinned source preimages were not restored exactly")
        return "reversed"

    working, temp = _apply_in_temporary_root(root, reverse=reverse)
    try:
        actual = _state(working)
        if actual != result:
            raise PatchError(f"popup patch postimage mismatch: {actual}")
        if check_only:
            return "verified"
        for relative in PREIMAGES:
            destination = root / relative
            replacement = working / relative
            with tempfile.NamedTemporaryFile(dir=destination.parent, delete=False) as output:
                temporary = Path(output.name)
                output.write(replacement.read_bytes())
            os.chmod(temporary, destination.stat().st_mode)
            os.replace(temporary, destination)
        if _state(root) != result:
            raise PatchError("backend source hash changed while installing the verified patch")
        return "reversed" if reverse else "applied"
    finally:
        temp.cleanup()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--check", action="store_true", help="verify replay without changing backend sources")
    parser.add_argument("--reverse", action="store_true", help="restore exact pinned source preimages")
    args = parser.parse_args()
    try:
        print(apply_patch(args.root.resolve(), reverse=args.reverse, check_only=args.check))
    except (OSError, PatchError) as error:
        parser.error(str(error))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
