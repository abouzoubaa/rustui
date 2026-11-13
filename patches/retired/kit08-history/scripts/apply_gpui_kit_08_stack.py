#!/usr/bin/env python3
"""Apply, verify, or reverse the approved GPUI Kit 0.8 patch stack."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path

from refresh_owned_gpui_kit_08 import refresh

ROOT = Path(__file__).resolve().parents[1]
STACK = ROOT / "patches/approved/kit08-stack.json"


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_stack() -> dict:
    data = json.loads(STACK.read_text(encoding="utf-8"))
    for entry in data["patches"]:
        patch = ROOT / "patches/approved" / entry["patch"]
        manifest_path = patch.with_suffix(".json")
        if sha(patch) != entry["patch_sha256"]:
            raise RuntimeError(f"approved patch hash mismatch: {patch}")
        if not manifest_path.is_file():
            raise RuntimeError(f"approved manifest missing: {manifest_path}")
        for target in entry["files"]:
            if not (ROOT / target).is_file():
                raise RuntimeError(f"approved patch target missing: {target}")
    return data


def operate(action: str) -> None:
    stack = load_stack()
    entries = stack["patches"]
    if action == "--check":
        for target, hashes in stack["files"].items():
            actual = sha(ROOT / target)
            if actual != hashes["post_sha256"]:
                raise RuntimeError(f"postimage mismatch for {target}: {actual}")
        print(f"Kit 0.8 approved patches={len(entries)}; files={len(stack['files'])}; applied state=PASS")
        return

    ordered = list(reversed(entries)) if action == "--reverse" else entries
    for entry in ordered:
        patch = ROOT / "patches/approved" / entry["patch"]
        if action == "--reverse":
            expected_key, command = "post_sha256", ["git", "apply", "--reverse"]
        else:
            expected_key, command = "pre_sha256", ["git", "apply"]
        for target, hashes in entry["files"].items():
            actual = sha(ROOT / target)
            if actual != hashes[expected_key]:
                raise RuntimeError(f"{action} preimage mismatch for {target}: {actual}")
        subprocess.run(["git", "apply", "--check", *(command[2:]), str(patch)], cwd=ROOT, check=True)
        subprocess.run([*command, str(patch)], cwd=ROOT, check=True)
        result_key = "pre_sha256" if action == "--reverse" else "post_sha256"
        for target, hashes in entry["files"].items():
            actual = sha(ROOT / target)
            if actual != hashes[result_key]:
                raise RuntimeError(f"{action} postimage mismatch for {target}: {actual}")
    refresh(ROOT / "scripts/data/gpui-kit-0.8.1-sources.json")
    print(f"Kit 0.8 approved patches={len(entries)}; action={action}; hashes=PASS; mirrors=refreshed")


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
    except (OSError, RuntimeError, json.JSONDecodeError, subprocess.CalledProcessError) as error:
        print(f"GPUI Kit 0.8 patch stack failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
