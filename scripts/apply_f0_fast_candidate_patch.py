#!/usr/bin/env python3
"""Replay a hash-pinned Fast candidate patch against vendor files."""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]

def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()

def read_target(path: Path) -> bytes | None:
    if path.is_symlink() or (path.exists() and not path.is_file()):
        raise ValueError(f"non-regular target: {path}")
    return path.read_bytes() if path.exists() else None

def state(data: bytes | None) -> str | None:
    return digest(data) if data is not None else None

def run(manifest_path: Path, action: str, root: Path = ROOT) -> None:
    root = root.resolve()
    manifest_path = manifest_path.resolve()
    manifest = json.loads(manifest_path.read_text())
    patch = manifest_path.parent / manifest["patch"]
    patch_bytes = patch.read_bytes()
    if digest(patch_bytes) != manifest["patch_sha256"]:
        raise ValueError(f"patch hash mismatch: {patch}")
    files = manifest["files"]
    current = {}
    for relative, hashes in files.items():
        if Path(relative).is_absolute() or ".." in Path(relative).parts:
            raise ValueError(f"unsafe target: {relative}")
        if hashes["post_sha256"] is None:
            raise ValueError(f"postimage must be a file: {relative}")
        path = root / relative
        data = read_target(path)
        actual = state(data)
        if actual not in (hashes["pre_sha256"], hashes["post_sha256"]):
            raise ValueError(f"unexpected hash for {relative}: {actual}")
        current[relative] = data
    reverse = action == "--reverse"
    if action == "--check":
        reverse = all(state(current[k]) == files[k]["post_sha256"] for k in files)
    expected = "post_sha256" if reverse else "pre_sha256"
    result = "pre_sha256" if reverse else "post_sha256"
    if action == "--apply" and all(state(current[k]) == files[k]["post_sha256"] for k in files):
        raise ValueError("patch already applied")
    if action == "--reverse" and all(state(current[k]) == files[k]["pre_sha256"] for k in files):
        raise ValueError("patch is already reversed")
    if any(state(data) != files[name][expected] for name, data in current.items()):
        raise ValueError("mixed patch states")
    with tempfile.TemporaryDirectory(prefix="f0-fast-patch-") as temp:
        stage = Path(temp)
        for relative, data in current.items():
            if data is None:
                continue
            target = stage / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        command = ["git", "apply", "--whitespace=nowarn"]
        if reverse:
            command.append("--reverse")
        subprocess.run(command + ["--check", str(patch)], cwd=stage, check=True, capture_output=True)
        subprocess.run(command + [str(patch)], cwd=stage, check=True, capture_output=True)
        for relative, hashes in files.items():
            target = stage / relative
            if state(read_target(target)) != hashes[result]:
                raise ValueError(f"patch result hash mismatch: {relative}")
        result_data = {relative: read_target(stage / relative) for relative in files}
        inverse = ["git", "apply", "--whitespace=nowarn"]
        if not reverse:
            inverse.append("--reverse")
        subprocess.run(inverse + [str(patch)], cwd=stage, check=True, capture_output=True)
        if any(read_target(stage / name) != data for name, data in current.items()):
            raise ValueError("reverse did not restore exact preimage (including file absence)")
        if action != "--check":
            for relative in files:
                path = root / relative
                if result_data[relative] is None:
                    path.unlink()
                    continue
                path.parent.mkdir(parents=True, exist_ok=True)
                temporary = path.with_suffix(path.suffix + ".f0-tmp")
                temporary.write_bytes(result_data[relative])
                os.chmod(temporary, path.stat().st_mode & 0o777 if path.exists() else 0o644)
                os.replace(temporary, path)
    print(f"F0 Fast candidate {action}: {manifest.get('name')}; files={len(files)}; hashes=PASS")

def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("manifest", type=Path)
    parser.add_argument("--root", type=Path, default=ROOT)
    group = parser.add_mutually_exclusive_group(required=True)
    for action in ("--apply", "--check", "--reverse"):
        group.add_argument(action, action="store_true")
    args = parser.parse_args()
    action = next(x for x in ("--apply", "--check", "--reverse") if getattr(args, x[2:].replace('-', '_')))
    try:
        run(args.manifest, action, args.root)
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"error: {error}\n")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
