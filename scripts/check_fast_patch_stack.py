#!/usr/bin/env python3
"""Verify every approved Fast patch and replay the cumulative stack exactly."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import tempfile

from apply_f0_fast_candidate_patch import run as apply_files
from approved_single_source_patch import run as apply_single

ROOT = Path(__file__).resolve().parents[1]
PATCH_DIR = ROOT / "patches/approved/fast"


def load_manifests():
    entries = []
    chain = {}
    for name in json.loads((PATCH_DIR / "fast-stack.json").read_text())["patches"]:
        path = PATCH_DIR / name
        data = json.loads(path.read_text())
        patch = ROOT / data["patch"] if "source" in data else path.parent / data["patch"]
        if hashlib.sha256(patch.read_bytes()).hexdigest() != data["patch_sha256"]:
            raise ValueError(f"patch hash mismatch: {name}")
        files = data.get("files") or {data["source"]: {
            "pre_sha256": data["pre_sha256"], "post_sha256": data["post_sha256"]}}
        for relative, hashes in files.items():
            if relative in chain and chain[relative] != hashes["pre_sha256"]:
                raise ValueError(f"broken cumulative hash chain: {name}: {relative}")
            chain[relative] = hashes["post_sha256"]
        entries.append((path, data, files))
    return entries, chain


def check(root: Path = ROOT):
    entries, final = load_manifests()
    for relative, expected in final.items():
        if hashlib.sha256((root / relative).read_bytes()).hexdigest() != expected:
            raise ValueError(f"approved Fast postimage mismatch: {relative}")
    for name in json.loads((PATCH_DIR / "fast-stack.json").read_text()).get("alternative_postimage_proofs", []):
        path = PATCH_DIR / name
        data = json.loads(path.read_text())
        if hashlib.sha256((path.parent / data["patch"]).read_bytes()).hexdigest() != data["patch_sha256"]:
            raise ValueError(f"alternative patch hash mismatch: {name}")
        for relative, hashes in data["files"].items():
            if final.get(relative) != hashes["post_sha256"]:
                raise ValueError(f"alternative postimage differs: {name}: {relative}")
    print(f"Approved Fast stack: {len(entries)} ordered patches + 1 alternative; {len(final)} files; exact postimages=PASS")
    return entries, final


def verify(root: Path = ROOT):
    entries, final = check(root)
    with tempfile.TemporaryDirectory(prefix="rustui-approved-fast-proof-") as temporary:
        fixture = Path(temporary)
        for relative in final:
            target = fixture / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(root / relative, target)
        for action, ordered in (("--reverse", reversed(entries)), ("--apply", entries)):
            for path, data, files in ordered:
                if "source" in data:
                    apply_single(fixture, ROOT, path.relative_to(ROOT),
                                 hashlib.sha256(path.read_bytes()).hexdigest(), action)
                else:
                    apply_files(path, action, fixture)
                key = "pre_sha256" if action == "--reverse" else "post_sha256"
                for relative, hashes in files.items():
                    if hashlib.sha256((fixture / relative).read_bytes()).hexdigest() != hashes[key]:
                        raise ValueError(f"{action} image mismatch: {path.name}: {relative}")
        check(fixture)
        for relative in final:
            if (fixture / relative).read_bytes() != (root / relative).read_bytes():
                raise ValueError(f"cumulative round trip differs: {relative}")
    for name in json.loads((PATCH_DIR / "fast-stack.json").read_text()).get("alternative_postimage_proofs", []):
        path = PATCH_DIR / name
        data = json.loads(path.read_text())
        with tempfile.TemporaryDirectory(prefix="rustui-fast-alternative-proof-") as temporary:
            fixture = Path(temporary)
            for relative in data["files"]:
                target = fixture / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(root / relative, target)
            apply_files(path, "--check", fixture)
            apply_files(path, "--reverse", fixture)
            apply_files(path, "--apply", fixture)
            for relative in data["files"]:
                if (fixture / relative).read_bytes() != (root / relative).read_bytes():
                    raise ValueError(f"alternative round trip differs: {relative}")
    print("Approved Fast cumulative reverse/apply and alternative postimages: exact restoration=PASS")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true")
    mode.add_argument("--verify", action="store_true")
    args = parser.parse_args()
    try:
        verify() if args.verify else check()
    except (OSError, ValueError, KeyError) as error:
        parser.exit(1, f"error: {error}\n")
