#!/usr/bin/env python3
"""Create/check/reverse owned Fast copies using pinned source and patch images."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import shutil

from apply_f0_fast_candidate_patch import run as apply_patch

ROOT = Path(__file__).resolve().parents[1]
SOURCES = ROOT / "patches/approved/fast/owned-fast-sources.json"
WIRING = ROOT / "patches/approved/fast/owned-fast-path-wiring-v1.json"


def hashes(directory: Path):
    result = {}
    for path in sorted(directory.rglob("*")):
        if path.is_symlink():
            raise ValueError(f"unexpected owned/source symlink: {path}")
        if path.is_file():
            result[path.relative_to(directory).as_posix()] = hashlib.sha256(path.read_bytes()).hexdigest()
    return result


def check_copies(root: Path, wired: bool):
    copies = json.loads(SOURCES.read_text())["copies"]
    wiring = json.loads(WIRING.read_text())["files"]
    for copy in copies:
        expected = copy["files"]
        if hashes(root / copy["vendor"]) != expected:
            raise ValueError(f"pinned Fast vendor snapshot differs: {copy['vendor']}")
        owned_expected = dict(expected)
        manifest = copy["owned"] + "/Cargo.toml"
        if wired and manifest in wiring:
            owned_expected["Cargo.toml"] = wiring[manifest]["post_sha256"]
        if hashes(root / copy["owned"]) != owned_expected:
            raise ValueError(f"owned Fast snapshot differs: {copy['owned']}")
    print(f"Owned Fast copies={len(copies)}; complete source/license snapshots=PASS; wired={wired}")


def operate(action: str, root: Path = ROOT):
    copies = json.loads(SOURCES.read_text())["copies"]
    if action == "--apply":
        for copy in copies:
            if hashes(root / copy["vendor"]) != copy["files"]:
                raise ValueError(f"pinned Fast vendor snapshot differs: {copy['vendor']}")
            if (root / copy["owned"]).exists():
                raise ValueError(f"refusing to overwrite owned Fast copy: {copy['owned']}")
        created = []
        try:
            for copy in copies:
                destination = root / copy["owned"]
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copytree(root / copy["vendor"], destination)
                created.append(destination)
            apply_patch(WIRING, "--apply", root)
        except Exception:
            for destination in created:
                shutil.rmtree(destination)
            raise
        check_copies(root, wired=True)
    elif action == "--reverse":
        check_copies(root, wired=True)
        apply_patch(WIRING, "--reverse", root)
        check_copies(root, wired=False)
        for copy in copies:
            shutil.rmtree(root / copy["owned"])
        print("Owned Fast source copies removed; Kit manifests restored exactly")
    else:
        check_copies(root, wired=True)
        apply_patch(WIRING, "--check", root)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    mode = parser.add_mutually_exclusive_group(required=True)
    for option in ("apply", "check", "reverse"):
        mode.add_argument("--" + option, action="store_true")
    args = parser.parse_args()
    action = next("--" + key for key in ("apply", "check", "reverse") if getattr(args, key))
    try:
        operate(action, args.root.resolve())
    except (OSError, ValueError, KeyError) as error:
        parser.exit(1, f"error: {error}\n")
