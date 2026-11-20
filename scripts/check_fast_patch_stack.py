#!/usr/bin/env python3
"""Verify every approved Fast patch and replay the cumulative stack exactly."""
from __future__ import annotations

import argparse
from contextlib import contextmanager
import hashlib
import json
from pathlib import Path
import shutil
import tempfile

from apply_f0_fast_candidate_patch import run as apply_files
from approved_single_source_patch import run as apply_single

ROOT = Path(__file__).resolve().parents[1]
PATCH_DIR = ROOT / "patches/approved/fast"
STACK = PATCH_DIR / "fast-stack.json"
DAMAGE_STACK = ROOT / "patches/candidate-fast-damage/damage-stack.json"


def active_stack():
    return DAMAGE_STACK if DAMAGE_STACK.exists() else STACK


def stack_paths(stack_path):
    stack_path = Path(stack_path).resolve()
    data = json.loads(stack_path.read_text())
    base = stack_path.parent / data["base_stack"] if "base_stack" in data else None
    return (stack_paths(base) if base else []) + [stack_path]


def load_manifests(stack_path=STACK):
    entries = []
    chain = {}
    paths = [stack.parent / name for stack in stack_paths(stack_path)
             for name in json.loads(stack.read_text())["patches"]]
    for path in paths:
        name = path.name
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


def alternative_paths(stack_path):
    return [stack.parent / name for stack in stack_paths(stack_path)
            for name in json.loads(stack.read_text()).get("alternative_postimage_proofs", [])]


@contextmanager
def materialized_manifests(root, stack_path):
    """Keep candidate-derived ownership wiring alive for the whole proof."""
    from refresh_owned_fast_sources import manifest_wiring, wired_manifests
    entries, _ = load_manifests(stack_path)
    with wired_manifests(root, stack_path) as effective:
        replacements = dict(zip(manifest_wiring(), effective))
        prepared, final = [], {}
        for path, data, files in entries:
            if path in replacements:
                path = replacements[path]
                data = json.loads(path.read_text())
                files = data["files"]
            prepared.append((path, data, files))
            final.update({name: hashes["post_sha256"] for name, hashes in files.items()})
        yield prepared, final


def check(root: Path = ROOT, stack_path=None, prepared=None):
    stack_path = stack_path or active_stack()
    if prepared is None:
        with materialized_manifests(root, stack_path) as images:
            return check(root, stack_path, images)
    entries, final = prepared
    for relative, expected in final.items():
        if hashlib.sha256((root / relative).read_bytes()).hexdigest() != expected:
            raise ValueError(f"Fast stack postimage mismatch: {relative}")
    alternatives = alternative_paths(stack_path)
    for path in alternatives:
        name = path.name
        data = json.loads(path.read_text())
        if hashlib.sha256((path.parent / data["patch"]).read_bytes()).hexdigest() != data["patch_sha256"]:
            raise ValueError(f"alternative patch hash mismatch: {name}")
        for relative, hashes in data["files"].items():
            if final.get(relative) != hashes["post_sha256"]:
                raise ValueError(f"alternative postimage differs: {name}: {relative}")
    print(f"Fast stack: {len(entries)} ordered patches + {len(alternatives)} alternatives; {len(final)} files; exact postimages=PASS")
    return entries, final


def verify(root: Path = ROOT, stack_path=None):
    stack_path = stack_path or active_stack()
    with materialized_manifests(root, stack_path) as images:
        _verify(root, stack_path, images)


def _verify(root, stack_path, images):
    entries, final = check(root, stack_path, images)
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
                    target = fixture / relative
                    actual = hashlib.sha256(target.read_bytes()).hexdigest() if target.exists() else None
                    if actual != hashes[key]:
                        raise ValueError(f"{action} image mismatch: {path.name}: {relative}")
        check(fixture, stack_path, images)
        for relative in final:
            if (fixture / relative).read_bytes() != (root / relative).read_bytes():
                raise ValueError(f"cumulative round trip differs: {relative}")
    for path in alternative_paths(stack_path):
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
    print("Fast cumulative reverse/apply and alternative postimages: exact restoration=PASS")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--stack", type=Path, default=active_stack())
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true")
    mode.add_argument("--verify", action="store_true")
    args = parser.parse_args()
    try:
        verify(stack_path=args.stack) if args.verify else check(stack_path=args.stack)
    except (OSError, ValueError, KeyError) as error:
        parser.exit(1, f"error: {error}\n")
