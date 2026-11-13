#!/usr/bin/env python3
"""Apply or verify the approved GPUI Kit 0.8 macOS anchored-popup patch."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PATCH = ROOT / "patches/approved/kit09/gpui-macos-anchored-popup-v1-kit09.patch"
MANIFEST = ROOT / "patches/approved/kit09/gpui-macos-anchored-popup-v1-kit09.json"


class PatchError(ValueError):
    pass


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def _manifest() -> dict:
    try:
        value = json.loads(MANIFEST.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise PatchError(f"cannot read popup patch manifest: {error}") from error
    if value.get("schema_version") != 1 or value.get("candidate") != PATCH.name:
        raise PatchError("popup patch manifest identity is invalid")
    if not PATCH.is_file() or sha256(PATCH) != value.get("patch_sha256"):
        raise PatchError("popup patch is missing or its hash differs from the manifest")
    files = value.get("files")
    if not isinstance(files, dict) or not files:
        raise PatchError("popup patch manifest must bind at least one vendor or owned file")
    for relative, hashes in files.items():
        if set(hashes) != {"pre_sha256", "post_sha256"}:
            raise PatchError(f"popup patch file hashes are invalid: {relative}")
    return value


_MANIFEST_DATA = json.loads(MANIFEST.read_text(encoding="utf-8"))
PREIMAGES = {path: hashes["pre_sha256"] for path, hashes in _MANIFEST_DATA["files"].items()}
POSTIMAGES = {path: hashes["post_sha256"] for path, hashes in _MANIFEST_DATA["files"].items()}


def _state(root: Path, files: dict) -> dict[str, str]:
    return {relative: sha256(root / relative) for relative in files}


def _run_patch(root: Path, *, reverse: bool) -> None:
    command = ["git", "apply", "--unidiff-zero"]
    if reverse:
        command.append("--reverse")
    command.append(str(PATCH))
    try:
        subprocess.run(command, cwd=root, check=True, capture_output=True, text=True)
    except subprocess.CalledProcessError as error:
        raise PatchError(error.stderr.strip() or error.stdout.strip()) from error


def _temporary_copy(root: Path, files: dict) -> tuple[Path, tempfile.TemporaryDirectory]:
    temporary = tempfile.TemporaryDirectory(prefix="rustui-gpui-macos-popup-kit08-")
    working = Path(temporary.name)
    for relative in files:
        source = root / relative
        target = working / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, target)
    return working, temporary


def apply_patch(root: Path = ROOT, *, reverse: bool = False, check_only: bool = False) -> str:
    manifest = _manifest()
    files = manifest["files"]
    preimages = {path: hashes["pre_sha256"] for path, hashes in files.items()}
    postimages = {path: hashes["post_sha256"] for path, hashes in files.items()}
    current = _state(root, files)
    expected = postimages if reverse else preimages
    result = preimages if reverse else postimages
    if current == result:
        return "already-reversed" if reverse else "already-applied"
    if current != expected:
        state = "postimage" if reverse else "preimage"
        raise PatchError(f"backend sources are not the known {state}: {current}")

    working, temporary = _temporary_copy(root, files)
    try:
        _run_patch(working, reverse=reverse)
        actual = _state(working, files)
        if actual != result:
            raise PatchError(f"popup patch image mismatch: {actual}")
        if check_only:
            return "verified"
        for relative in files:
            destination = root / relative
            replacement = working / relative
            with tempfile.NamedTemporaryFile(dir=destination.parent, delete=False) as output:
                temporary_file = Path(output.name)
                output.write(replacement.read_bytes())
            os.chmod(temporary_file, destination.stat().st_mode)
            os.replace(temporary_file, destination)
        if _state(root, files) != result:
            raise PatchError("popup source hashes changed while installing the verified patch")
        return "reversed" if reverse else "applied"
    finally:
        temporary.cleanup()


def verify_applied(root: Path = ROOT) -> str:
    manifest = _manifest()
    current = _state(root, manifest["files"])
    expected = {path: hashes["post_sha256"] for path, hashes in manifest["files"].items()}
    if current != expected:
        raise PatchError(f"approved popup patch is not active: {current}")
    return "verified"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", help="verify in a temporary copy")
    parser.add_argument("--reverse", action="store_true", help="reverse the patch")
    parser.add_argument("--verify", action="store_true", help="verify approved postimages are active")
    arguments = parser.parse_args()
    if arguments.verify and (arguments.check or arguments.reverse):
        parser.error("--verify cannot be combined with --check or --reverse")
    try:
        if arguments.verify:
            print(verify_applied())
        else:
            print(apply_patch(reverse=arguments.reverse, check_only=arguments.check))
    except (OSError, PatchError) as error:
        parser.error(str(error))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
