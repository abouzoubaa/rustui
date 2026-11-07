#!/usr/bin/env python3
"""Check PRE, verify POST, apply or reverse the approved three-source Tree series.

The series assumes the approved aria-disabled patch is already in the pinned
vendor source. It never changes Cargo selection, registry checksums, or licenses.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import tomllib

ARIA_DISABLED_PATCH = Path("patches/approved/gpui-tree-aria-disabled-v1.patch")
ARIA_LEVEL_PATCH = Path("patches/approved/gpui-tree-aria-level-v1.patch")
ROOT_NAME_PATCH = Path("patches/approved/gpui-component-tree-accessible-name-v1.patch")
TEST_A11Y_PATCH = Path("patches/approved/gpui-pre-test-a11y-activation-v1.patch")
PATCH_HASHES = {
    ARIA_DISABLED_PATCH: "5762dfec8eba4b723bc1b851c527c75f18e78d76d050fd8bab4485a63ceb14d1",
    ARIA_LEVEL_PATCH: "7729270cd97bfc74039e05d5142d522b78cc9bd563a825ccaa33caeb2d8f77f7",
    ROOT_NAME_PATCH: "431b914cb22038afe3f8cf4e2e8ee5818c4adc85a361cd73cfb40c4951aaff83",
    TEST_A11Y_PATCH: "bb59d8b818433f24624c54e65ce99658ddb7d6308261ced15c65b621efc3f249",
}
BASE = Path("vendor/gpui-base-0.7.3/src/tree.rs")
COMPONENT = Path("vendor/gpui-component-0.7.3/src/tree.rs")
TEST_WINDOW = Path("vendor/gpui-pre-0.3.9/src/platform/test/window.rs")
FILES = (BASE, COMPONENT, TEST_WINDOW)
PRE = {
    BASE: "9ef2efa0a44af810eef3088343b0838179d5e952c6ca8734e5306d5c74736924",
    COMPONENT: "121c7b715b51d520e531682a14f9ba2791cd3a39554d68ab7cab289bfe7814f9",
    TEST_WINDOW: "3b359c9e7943ccb065096c718756b32400dac3394bfd9ab6f361dda8163e97bd",
}
POST = {
    BASE: "0a1321f50154fb145b325517f9be93f262e959d21a213d61ecfdfadec38e4747",
    COMPONENT: "12e2e2eca25f85878c4ab681e8fae4e58342c6ac9b3cc60a831e4c8894d53df4",
    TEST_WINDOW: "8b267a9dfb6aadbfb00430ce491a85c1446789726ad84e54366247afa953d4c4",
}
IDENTITY = {
    "vendor/gpui-pre-0.3.9/Cargo.toml": ("gpui-pre", "0.3.9"),
    "vendor/gpui-base-0.7.3/Cargo.toml": ("gpui-base", "0.7.3"),
    "vendor/gpui-component-0.7.3/Cargo.toml": ("gpui-component", "0.7.3"),
}
PROVENANCE = {
    "vendor/gpui-pre-0.3.9/.cargo-checksum.json": "7b8e889ae4ac0cbe651b654f5e39d9003969d09ae4a70aff4147f521eda8faa0",
    "vendor/gpui-pre-0.3.9/LICENSE-APACHE": "752daf2fb234ca4a1fa372c073fe127f44b7b90fd2529ae44273a64f9d53da7a",
    "vendor/gpui-base-0.7.3/.cargo-checksum.json": "9e96d55b0071b60576bc6ac18c3e9c48e6d0de5ff848821af7623bd151f7c3ed",
    "vendor/gpui-base-0.7.3/LICENSE-APACHE": "d1b0449e5478c574ba4f686c2656df7fe77d66821a61f8b6ed3378a58ed9a811",
    "vendor/gpui-component-0.7.3/.cargo-checksum.json": "d5b533efba197cc15bcb6756b935a786e846463c1ee431f876e8c334a7e638c6",
    "vendor/gpui-component-0.7.3/LICENSE-APACHE": "d1b0449e5478c574ba4f686c2656df7fe77d66821a61f8b6ed3378a58ed9a811",
    "vendor/gpui-pre-0.3.9/src/elements/div.rs": "de8298ce4e48f055984742a18f18206e385786f64ff495f2d5b176481033b84e",
}
REGISTRY_PACKAGES = {
    "gpui-base": "2e9955b6f4b048fd50ca15fc8e225d80bb00ba9ebc8fecdddde5ad26dbef9270",
    "gpui-component": "465a363235cd4062ef5f0953fc096fc764ed6f53dfbd9f409f669f93ea3b1ff9",
}
PATCH_ORDER = (TEST_A11Y_PATCH, ARIA_LEVEL_PATCH, ROOT_NAME_PATCH)


DISABLED_FILES = {
    Path('vendor/gpui-pre-0.3.9/src/elements/div.rs'): ('f320948b8f90e36b7100ac3e0e061235d1e9422aeb323918e313f7d69980cfa9', 'de8298ce4e48f055984742a18f18206e385786f64ff495f2d5b176481033b84e'),
    Path('vendor/gpui-base-0.7.3/src/tree.rs'): ('e74e4fcdef6ccec174e3e6ced50f108433e726f3e3d46f3539b1f41d5944e5df', '9ef2efa0a44af810eef3088343b0838179d5e952c6ca8734e5306d5c74736924'),
    Path('vendor/gpui-base-0.7.3/src/switch.rs'): ('d06394441975ac7eb4478d353762c527ca0e9d8034c7f5097902d9d976543567', 'aed20dabde162201be920d297dac451c31b8fc18385c60298f90123c1cae724d'),
}

def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_digest(path: Path) -> str:
    return digest(path.read_bytes())


def verify_identity(root: Path) -> None:
    for relative, expected in IDENTITY.items():
        package = tomllib.loads((root / relative).read_text(encoding="utf-8"))["package"]
        actual = (package.get("name"), package.get("version"))
        if actual != expected:
            raise ValueError(f"unexpected package identity for {relative}: {actual}")

    for relative, expected in PROVENANCE.items():
        if file_digest(root / relative) != expected:
            raise ValueError(f"upstream provenance changed: {relative}")

    lock = tomllib.loads((root / "Cargo.lock").read_text(encoding="utf-8"))
    for name, expected in REGISTRY_PACKAGES.items():
        next(
            package for package in lock["package"]
            if package.get("name") == name and package.get("version") == "0.7.3"
        )
        checksums = json.loads(
            (root / f"vendor/{name}-0.7.3/.cargo-checksum.json").read_text(encoding="utf-8")
        )
        if checksums.get("package") != expected:
            raise ValueError(f"unexpected registry archive checksum for {name}")

    manifest = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
    selected = manifest.get("patch", {}).get("crates-io", {})
    for name, expected in (("gpui-base", "vendor/gpui-base-0.7.3"), ("gpui-component", "vendor/gpui-component-0.7.3")):
        if selected.get(name, {}).get("path") != expected:
            raise ValueError(f"Cargo no longer selects the expected local {name} source")

    for relative, expected in PATCH_HASHES.items():
        if file_digest(root / relative) != expected:
            raise ValueError(f"patch artifact hash changed: {relative}")


def apply_patch(patch_root: Path, staged_root: Path, patch: Path, reverse: bool = False) -> None:
    command = ["git", "apply", "--unsafe-paths"]
    if reverse:
        command.append("-R")
    command.extend(["--check", str(patch_root / patch)])
    checked = subprocess.run(command, cwd=staged_root, capture_output=True, text=True, check=False)
    if checked.returncode:
        raise ValueError(f"exact patch check failed {patch}: {(checked.stdout + checked.stderr).strip()}")
    command.remove("--check")
    applied = subprocess.run(command, cwd=staged_root, capture_output=True, text=True, check=False)
    if applied.returncode:
        raise ValueError(f"exact patch application failed {patch}: {(applied.stdout + applied.stderr).strip()}")
    output = applied.stdout + applied.stderr
    if re.search(r"offset|fuzz|FAILED", output, re.IGNORECASE):
        raise ValueError(f"non-exact patch application {patch}: {output.strip()}")


def transform(root: Path, sources: dict[Path, bytes], reverse: bool) -> dict[Path, bytes]:
    with tempfile.TemporaryDirectory(prefix="rustui-tree-accessibility-series-") as temporary:
        staged_root = Path(temporary)
        for relative, contents in sources.items():
            staged = staged_root / relative
            staged.parent.mkdir(parents=True, exist_ok=True)
            staged.write_bytes(contents)
        patches = tuple(reversed(PATCH_ORDER)) if reverse else PATCH_ORDER
        for patch in patches:
            apply_patch(root, staged_root, patch, reverse=reverse)
        expected = PRE if reverse else POST
        result = {relative: (staged_root / relative).read_bytes() for relative in FILES}
        for relative, contents in result.items():
            if digest(contents) != expected[relative]:
                raise ValueError(f"unexpected {'reversed' if reverse else 'proposed'} image: {relative}")
        return result


def install(root: Path, updated: dict[Path, bytes]) -> None:
    originals = {relative: (root / relative).read_bytes() for relative in updated}
    temporary: dict[Path, Path] = {}
    try:
        for relative, contents in updated.items():
            target = root / relative
            temp = target.with_name(f".{target.name}.tree-a11y-tmp")
            temp.write_bytes(contents)
            temporary[relative] = temp
        for relative, temp in temporary.items():
            os.replace(temp, root / relative)
    except OSError:
        for relative, contents in originals.items():
            target = root / relative
            rollback = target.with_name(f".{target.name}.tree-a11y-rollback")
            rollback.write_bytes(contents)
            os.replace(rollback, target)
        raise
    finally:
        for temp in temporary.values():
            temp.unlink(missing_ok=True)


def verify_disabled_composition(root: Path, preimages: dict[Path, bytes]) -> None:
    """Prove the approved disabled baseline under the reversed three-source series."""
    inputs = {relative: preimages.get(relative, (root / relative).read_bytes()) for relative in DISABLED_FILES}
    for relative, contents in inputs.items():
        if digest(contents) != DISABLED_FILES[relative][1]:
            raise ValueError(f"unexpected approved aria-disabled baseline: {relative}")
    with tempfile.TemporaryDirectory(prefix="rustui-tree-disabled-composition-") as temporary:
        stage = Path(temporary)
        for relative, contents in inputs.items():
            target = stage / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(contents)
        apply_patch(root, stage, ARIA_DISABLED_PATCH, reverse=True)
        for relative, hashes in DISABLED_FILES.items():
            if file_digest(stage / relative) != hashes[0]:
                raise ValueError(f"aria-disabled reverse did not recover pristine source: {relative}")
        apply_patch(root, stage, ARIA_DISABLED_PATCH)
        for relative, contents in inputs.items():
            if (stage / relative).read_bytes() != contents:
                raise ValueError(f"aria-disabled composition did not replay exactly: {relative}")


def verify_post(root: Path, current: dict[Path, bytes]) -> None:
    for relative, contents in current.items():
        if digest(contents) != POST[relative]:
            raise ValueError(f"unexpected POST {relative} image: {digest(contents)}")
    restored = transform(root, current, reverse=True)
    verify_disabled_composition(root, restored)
    replayed = transform(root, restored, reverse=False)
    if replayed != current:
        raise ValueError("POST series reverse/replay did not restore exact composed images")


def run(root: Path, mode: str) -> None:
    root = root.resolve()
    verify_identity(root)
    current = {relative: (root / relative).read_bytes() for relative in FILES}
    expected = POST if mode in ("--reverse", "--verify") else PRE
    for relative, expected_hash in expected.items():
        if digest(current[relative]) != expected_hash:
            raise ValueError(f"unexpected {relative} image: {digest(current[relative])}")
    if mode == "--verify":
        verify_post(root, current)
        print("Approved three-source Tree accessibility POST and composed aria-disabled baseline verified; checkout unchanged")
        return
    if mode == "--check":
        verify_disabled_composition(root, current)
        proposed = transform(root, current, reverse=False)
        verify_post(root, proposed)
        restored = transform(root, proposed, reverse=True)
        if restored != current:
            raise ValueError("reverse series did not restore exact approved preimages")
        print("Approved three-source Tree accessibility PRE/apply/POST/reverse and baseline composition round-trip exactly; checkout unchanged")
        return
    if mode == "--apply":
        verify_disabled_composition(root, current)
    else:
        verify_post(root, current)
    updated = transform(root, current, reverse=mode == "--reverse")
    install(root, updated)
    print(f"Approved Tree accessibility series {'reversed' if mode == '--reverse' else 'applied'}; all three source hashes verified")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true")
    mode.add_argument("--verify", action="store_true")
    mode.add_argument("--apply", action="store_true")
    mode.add_argument("--reverse", action="store_true")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    selected = "--verify" if args.verify else "--check" if args.check else "--reverse" if args.reverse else "--apply"
    try:
        run(args.root, selected)
    except (OSError, ValueError, KeyError, StopIteration, json.JSONDecodeError, tomllib.TOMLDecodeError) as error:
        raise SystemExit(f"error: {error}") from error
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
