#!/usr/bin/env python3
"""Apply/check/reverse the byte-preserving legacy mirror archive patch."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "patches/approved/fast-retirement/archive-unused-mirrors-v1.json"


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load():
    data = json.loads(MANIFEST.read_text())
    patch = MANIFEST.parent / data["patch"]
    if digest(patch) != data["patch_sha256"]:
        raise ValueError("legacy archive patch hash mismatch")
    return data, patch


def check(root, archived):
    data, _ = load()
    for move in data["moves"]:
        present = move["destination"] if archived else move["source"]
        absent = move["source"] if archived else move["destination"]
        key = "post_sha256" if archived else "pre_sha256"
        path = root / present
        if path.is_symlink() or not path.is_file() or digest(path) != move[key]:
            raise ValueError(f"legacy archive image differs: {present}")
        if (root / absent).exists():
            raise ValueError(f"legacy archive destination already exists: {absent}")
    for directory in data["directories"]:
        relative = "archive/pre-fast/" + directory if archived else directory
        expected = {move["destination"] if archived else move["source"]
                    for move in data["moves"] if move["source"].startswith(directory + "/")}
        actual = {path.relative_to(root).as_posix() for path in (root / relative).rglob("*")
                  if path.is_file() or path.is_symlink()}
        if actual != expected:
            raise ValueError(f"unexpected legacy archive files: {relative}")
    print(f"Legacy archive: {len(data['directories'])} directories; {len(data['moves'])} exact file images; archived={archived}")


def operate(action, root=ROOT):
    data, patch = load()
    if action == "--check":
        check(root, archived=True)
        return
    reverse = action == "--reverse"
    check(root, archived=reverse)
    command = ["git", "apply"] + (["--reverse"] if reverse else [])
    subprocess.run(command + ["--check", str(patch)], cwd=root, check=True)
    subprocess.run(command + [str(patch)], cwd=root, check=True)
    check(root, archived=not reverse)
    for relative in data["directories"]:
        directory = root / ("archive/pre-fast/" + relative if reverse else relative)
        if directory.exists():
            for path in sorted(directory.rglob("*"), key=lambda p: len(p.parts), reverse=True):
                if path.is_dir():
                    path.rmdir()
            directory.rmdir()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    group = parser.add_mutually_exclusive_group(required=True)
    for name in ("apply", "check", "reverse"):
        group.add_argument("--" + name, action="store_true")
    args = parser.parse_args()
    action = next("--" + name for name in ("apply", "check", "reverse") if getattr(args, name))
    try:
        operate(action, args.root.resolve())
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"error: {error}\n")
