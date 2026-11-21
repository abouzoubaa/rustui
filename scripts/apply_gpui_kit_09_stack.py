#!/usr/bin/env python3
"""Apply, verify, or reverse the promoted GPUI Kit 0.9 patch stack."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import tempfile
import shutil
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
STACK = ROOT / "patches/approved/kit09/kit09-stack.json"
MIRRORS = {
    "gpui-pre": "crates/rustui-gpui",
    "gpui-pre-linux": "crates/rustui-gpui-linux",
    "gpui-pre-windows": "crates/rustui-gpui-windows",
    "gpui-pre-web": "crates/rustui-gpui-web",
    "gpui-pre-wgpu": "crates/rustui-gpui-wgpu",
    "gpui-base": "crates/rustui-gpui-base",
    "gpui-component": "crates/rustui-gpui-component",
    "gpui-kit-assets": "crates/rustui-gpui-kit-assets",
}


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_stack() -> dict:
    stack = json.loads(STACK.read_text(encoding="utf-8"))
    for item in stack["patches"]:
        patch = ROOT / "patches/approved/kit09" / item["patch"]
        if not patch.is_file() or sha(patch) != item["patch_sha256"]:
            raise RuntimeError(f"Kit 0.9 approved patch missing or hash mismatch: {patch}")
        if not patch.with_suffix(".json").is_file():
            raise RuntimeError(f"Kit 0.9 per-patch manifest missing: {patch.with_suffix('.json')}")
    return stack


def refresh_mirrors(stack: dict) -> int:
    version_by_package = {}
    for target in stack["files"]:
        parts = Path(target).parts
        if len(parts) > 2 and parts[0] == "vendor":
            directory = parts[1]
            for package in sorted(MIRRORS, key=len, reverse=True):
                if directory.startswith(package + "-"):
                    version_by_package[package] = directory[len(package) + 1 :]
                    break
    refreshed = 0
    for package, relative in MIRRORS.items():
        version = version_by_package.get(package)
        if version is None:
            continue
        source = ROOT / "vendor" / f"{package}-{version}"
        destination = ROOT / relative
        if not source.is_dir() or not destination.is_dir():
            raise RuntimeError(f"cannot refresh owned mirror: {source} -> {destination}")
        for child in list(destination.iterdir()):
            if child.name == "Cargo.toml":
                continue
            if child.is_dir():
                import shutil
                shutil.rmtree(child)
            else:
                child.unlink()
        for child in source.iterdir():
            if child.name == "Cargo.toml":
                continue
            target = destination / child.name
            if child.is_dir():
                import shutil
                shutil.copytree(child, target)
            else:
                import shutil
                shutil.copy2(child, target)
        refreshed += 1
    return refreshed


def check_postimages(stack: dict, root: Path = ROOT) -> None:
    """Validate approved Kit images plus explicitly listed candidate layers.

    A candidate must start at the approved hash, chain exactly, and reverse to
    the approved image (including absence of newly created files). The approved
    manifest is never rewritten to bless a candidate's current contents.
    """
    root = root.resolve()
    expected = {name: hashes["post_sha256"] for name, hashes in stack["files"].items()}
    prefixes = {"/".join(Path(name).parts[:2]) + "/" for name in expected
                if Path(name).parts[0] == "vendor"}
    candidate_stack = root / "patches/candidate-fast-damage/damage-stack.json"
    candidates = []
    changed_kit = False
    if candidate_stack.is_file():
        from check_fast_patch_stack import load_manifests
        entries, _ = load_manifests(candidate_stack)
        candidates = [(path, files) for path, _, files in entries
                      if not path.is_relative_to(root / "patches/approved")]
        for path, files in candidates:
            for name, hashes in files.items():
                if not any(name.startswith(prefix) for prefix in prefixes):
                    continue
                if name not in expected and hashes["pre_sha256"] is not None:
                    # The historical Kit table omitted Cargo checksum files.
                    # Pin their otherwise-untracked preimage through the explicit
                    # candidate manifest. Approved source hashes stay mandatory;
                    # the reverse proof restores this metadata byte-for-byte too.
                    if Path(name).name != ".cargo-checksum.json":
                        raise RuntimeError(f"Kit candidate lacks an approved base image: {name}")
                    expected[name] = hashes["pre_sha256"]
                if hashes["pre_sha256"] != expected.get(name):
                    raise RuntimeError(f"Kit candidate does not chain from approved image: {path.name}: {name}")
                expected[name] = hashes["post_sha256"]
                changed_kit = True
    for target, postimage in expected.items():
        path = root / target
        if not path.is_file() or sha(path) != postimage:
            raise RuntimeError(f"Kit 0.9 postimage mismatch: {target}")
    if not changed_kit:
        return

    from apply_f0_fast_candidate_patch import run as apply_candidate
    names = {name for _, files in candidates for name in files}
    for name in names:
        if Path(name).is_absolute() or ".." in Path(name).parts:
            raise ValueError(f"unsafe candidate target: {name}")
        if (root / name).is_symlink() or not (root / name).is_file():
            raise RuntimeError(f"non-regular candidate target: {name}")
    with tempfile.TemporaryDirectory(prefix="rustui-kit-candidate-proof-") as directory:
        fixture = Path(directory)
        originals = {}
        for name in names:
            source = root / name
            originals[name] = source.read_bytes()
            destination = fixture / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, destination)
        for path, _ in reversed(candidates):
            apply_candidate(path, "--reverse", fixture)
        for name, hashes in stack["files"].items():
            if name in names and sha(fixture / name) != hashes["post_sha256"]:
                raise RuntimeError(f"Kit candidate reverse differs from approved image: {name}")
        for path, _ in candidates:
            apply_candidate(path, "--apply", fixture)
        for name, original in originals.items():
            if (fixture / name).read_bytes() != original:
                raise RuntimeError(f"Kit candidate round trip differs: {name}")
    print("Kit candidate chain and reverse/apply: exact approved restoration=PASS")


def operate(action: str) -> None:
    stack = load_stack()
    if action == "--check":
        check_postimages(stack)
        print(f"Kit 0.9 approved patches={len(stack['patches'])}; files={len(stack['files'])}; applied state=PASS")
        return

    entries = list(reversed(stack["patches"])) if action == "--reverse" else stack["patches"]
    for entry in entries:
        patch = ROOT / "patches/approved/kit09" / entry["patch"]
        expected_key = "post_sha256" if action == "--reverse" else "pre_sha256"
        result_key = "pre_sha256" if action == "--reverse" else "post_sha256"
        command = ["git", "apply", "--reverse"] if action == "--reverse" else ["git", "apply"]
        for target, hashes in entry["files"].items():
            path = ROOT / target
            if not path.is_file() or sha(path) != hashes[expected_key]:
                raise RuntimeError(f"{action} image mismatch: {target}")
        subprocess.run(["git", "apply", "--check", *(command[2:]), str(patch)], cwd=ROOT, check=True)
        subprocess.run([*command, str(patch)], cwd=ROOT, check=True)
        for target, hashes in entry["files"].items():
            if sha(ROOT / target) != hashes[result_key]:
                raise RuntimeError(f"{action} result mismatch: {target}")
    refreshed = refresh_mirrors(stack)
    print(f"Kit 0.9 approved patches={len(entries)}; action={action}; hashes=PASS; mirrors refreshed={refreshed}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--apply", action="store_true")
    group.add_argument("--check", action="store_true")
    group.add_argument("--reverse", action="store_true")
    args = parser.parse_args()
    action = "--apply" if args.apply else "--reverse" if args.reverse else "--check"
    try:
        operate(action)
    except (OSError, ValueError, RuntimeError, json.JSONDecodeError, subprocess.CalledProcessError) as error:
        print(f"GPUI Kit 0.9 patch stack failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
