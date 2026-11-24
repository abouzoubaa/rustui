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
APPROVED_CATALOG = Path("scripts/fixtures/kit09-approved-cargo-state.json")
APPROVED_CATALOG_SHA256 = "b394930d4741e9f8892e1da7adf139a953a1538fe34b7ef8b3ff40f8fe9102ba"
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


def approved_images(stack: dict, root: Path):
    expected = {name: hashes["post_sha256"] for name, hashes in stack["files"].items()}
    catalog_path = root / APPROVED_CATALOG
    anchors = dict(expected)
    prefixes = {"/".join(Path(name).parts[:2]) for name in expected if name.startswith("vendor/")}
    if catalog_path.is_file():
        if root == ROOT and sha(catalog_path) != APPROVED_CATALOG_SHA256:
            raise RuntimeError("approved Kit Cargo catalog hash mismatch")
        catalog = json.loads(catalog_path.read_text())
        for name, digest in catalog["files"].items():
            if name in expected and expected[name] != digest:
                raise RuntimeError(f"approved Kit catalog disagrees with approved stack: {name}")
            expected[name] = digest
        anchors.update(catalog["cargo_anchors"])
        prefixes = set(catalog["crate_prefixes"])
    elif root == ROOT:
        raise RuntimeError("approved Kit Cargo catalog missing")
    return expected, anchors, prefixes


def image_bytes(root: Path, name: str):
    path = root / name
    if path.is_symlink():
        raise RuntimeError(f"non-regular candidate target: {name}")
    if not path.exists():
        return None
    if not path.is_file():
        raise RuntimeError(f"non-regular candidate target: {name}")
    return path.read_bytes()


def check_images(root: Path, expected: dict, prefixes: set):
    names = set(expected)
    for prefix in prefixes:
        folder = root / prefix
        if folder.is_dir():
            names.update(str(p.relative_to(root)) for p in folder.rglob("*") if p.is_file() or p.is_symlink())
    for name in names:
        data = image_bytes(root, name)
        digest = hashlib.sha256(data).hexdigest() if data is not None else None
        if digest != expected.get(name):
            raise RuntimeError(f"Kit 0.9 postimage mismatch: {name}")


def check_postimages(stack: dict, root: Path = ROOT, candidate_stack: Path | None = None) -> None:
    """Default: approved bytes only. Candidates require an explicit integration option."""
    root = root.resolve()
    approved, anchors, prefixes = approved_images(stack, root)
    if candidate_stack is None:
        check_images(root, approved, prefixes)
        return
    from check_fast_patch_stack import load_manifests
    from apply_f0_fast_candidate_patch import run as apply_candidate
    candidate_stack = Path(candidate_stack)
    if not candidate_stack.is_absolute():
        candidate_stack = root / candidate_stack
    entries, _ = load_manifests(candidate_stack)
    candidates = [(path, files) for path, _, files in entries
                  if not path.is_relative_to(root / "patches/approved")]
    names = {name for _, files in candidates for name in files}
    for name in names:
        if Path(name).is_absolute() or ".." in Path(name).parts:
            raise ValueError(f"unsafe candidate target: {name}")
        image_bytes(root, name)
    def kit(name):
        return any(name.startswith(prefix + "/") for prefix in prefixes)
    expected = dict(approved)
    touched = set()
    for path, files in candidates:
        for name, hashes in files.items():
            if not kit(name):
                continue
            before = hashes["pre_sha256"]
            if name not in touched and before is not None:
                vendor_name = name.replace("crates/rustui-gpui-component/", "vendor/gpui-component-0.9.1/").replace("crates/rustui-gpui-base/", "vendor/gpui-base-0.9.1/")
                known = vendor_name in stack["files"]
                anchor = approved.get(name) if known else anchors.get(name)
                if anchor is None:
                    raise RuntimeError(f"Kit candidate lacks an approved base image: {name}")
                if before != anchor:
                    raise RuntimeError(f"Kit candidate does not chain from approved Cargo image: {path.name}: {name}")
            if before != expected.get(name):
                raise RuntimeError(f"Kit candidate does not chain from approved image: {path.name}: {name}")
            expected[name] = hashes["post_sha256"]
            touched.add(name)
    check_images(root, expected, prefixes)
    # Copy every approved Kit file, every candidate target (including Fast),
    # and every actual Kit file. Nothing unknown can disappear from the proof.
    names.update(approved)
    for prefix in prefixes:
        folder = root / prefix
        if folder.is_dir():
            names.update(str(p.relative_to(root)) for p in folder.rglob("*") if p.is_file() or p.is_symlink())
    with tempfile.TemporaryDirectory(prefix="rustui-kit-candidate-proof-") as directory:
        fixture = Path(directory)
        original = {name: image_bytes(root, name) for name in names}
        for name, data in original.items():
            if data is not None:
                target = fixture / name
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(data)
        for path, _ in reversed(candidates):
            apply_candidate(path, "--reverse", fixture)
        # Approved state first, including every catalogued source and metadata
        # byte. A new candidate file must now be absent.
        check_images(fixture, approved, prefixes)
        baseline = {name: image_bytes(fixture, name) for name in names}
        for path, _ in candidates:
            apply_candidate(path, "--apply", fixture)
        for name, data in original.items():
            if image_bytes(fixture, name) != data:
                raise RuntimeError(f"Kit candidate round trip differs: {name}")
        for path, _ in reversed(candidates):
            apply_candidate(path, "--reverse", fixture)
        check_images(fixture, approved, prefixes)
        for name, data in baseline.items():
            if image_bytes(fixture, name) != data:
                raise RuntimeError(f"Kit candidate reverse differs from approved bytes/absence: {name}")
    print("Kit explicit candidate chain: Cargo anchors, post-hashes and exact approved reverse=PASS")


def operate(action: str, candidate_stack: Path | None = None) -> None:
    stack = load_stack()
    if action == "--check":
        check_postimages(stack, candidate_stack=candidate_stack)
        print(f"Kit 0.9 approved patches={len(stack['patches'])}; files={len(stack['files'])}; " + ("approved baseline + explicit candidates=PASS" if candidate_stack is not None else "approved-only state=PASS"))
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
    parser.add_argument("--candidate-stack", type=Path, help="explicit integration-only candidate chain; --check only")
    args = parser.parse_args()
    if args.candidate_stack and not args.check:
        parser.error("--candidate-stack is supported only with --check")
    action = "--apply" if args.apply else "--reverse" if args.reverse else "--check"
    try:
        operate(action, args.candidate_stack)
    except (OSError, ValueError, RuntimeError, json.JSONDecodeError, subprocess.CalledProcessError) as error:
        print(f"GPUI Kit 0.9 patch stack failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
