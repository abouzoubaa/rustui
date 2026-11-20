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


def manifest_wiring():
    names = json.loads((WIRING.parent / "fast-stack.json").read_text())["patches"]
    names = names[names.index(WIRING.name):]
    paths = [WIRING.parent / name for name in names]
    for path in paths:
        data = json.loads(path.read_text())
        if any(not name.endswith("/Cargo.toml") for name in data["files"]):
            raise ValueError(f"ownership tail must contain manifest-only patches: {path.name}")
        if hashlib.sha256((path.parent / data["patch"]).read_bytes()).hexdigest() != data["patch_sha256"]:
            raise ValueError(f"ownership patch hash mismatch: {path.name}")
    return paths


def check_manifest_images(root):
    final = {}
    for path in manifest_wiring():
        final.update({name: hashes["post_sha256"]
                      for name, hashes in json.loads(path.read_text())["files"].items()})
    for name, expected in final.items():
        if hashlib.sha256((root / name).read_bytes()).hexdigest() != expected:
            raise ValueError(f"owned manifest postimage differs: {name}")


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
    wiring = {}
    for path in manifest_wiring():
        wiring.update(json.loads(path.read_text())["files"])
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
        applied = []
        try:
            for copy in copies:
                destination = root / copy["owned"]
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copytree(root / copy["vendor"], destination)
                created.append(destination)
            for path in manifest_wiring():
                apply_patch(path, "--apply", root)
                applied.append(path)
        except Exception:
            for path in reversed(applied):
                apply_patch(path, "--reverse", root)
            for destination in created:
                shutil.rmtree(destination)
            raise
        check_copies(root, wired=True)
    elif action == "--reverse":
        check_copies(root, wired=True)
        check_manifest_images(root)
        for path in reversed(manifest_wiring()):
            apply_patch(path, "--reverse", root)
        check_copies(root, wired=False)
        for copy in copies:
            shutil.rmtree(root / copy["owned"])
        print("Owned Fast source copies removed; Kit manifests restored exactly")
    else:
        check_copies(root, wired=True)
        check_manifest_images(root)


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
