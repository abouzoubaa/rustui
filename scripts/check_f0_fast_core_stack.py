#!/usr/bin/env python3
"""Check, apply, or reverse the ordered GPUI Fast core candidate patches."""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
STACK = ROOT / "patches/candidate-kit-fast/core-stack.json"
APPLIER = ROOT / "scripts/apply_f0_fast_candidate_patch.py"

def run(action: str, root: Path = ROOT) -> None:
    stack = json.loads(STACK.read_text())
    manifests = [ROOT / "patches/candidate-kit-fast" / name for name in stack["patches"]]
    if action == "--check":
        final = {}
        for path in manifests:
            entry = json.loads(path.read_text())
            patch = path.parent / entry["patch"]
            if hashlib.sha256(patch.read_bytes()).hexdigest() != entry["patch_sha256"]:
                raise ValueError(f"patch hash mismatch: {patch}")
            for relative, hashes in entry["files"].items():
                final[relative] = hashes["post_sha256"]
        for relative, expected in final.items():
            actual = hashlib.sha256((root / relative).read_bytes()).hexdigest()
            if actual != expected:
                raise ValueError(f"final stack hash mismatch: {relative}")
    else:
        ordered = list(reversed(manifests)) if action == "--reverse" else manifests
        for manifest in ordered:
            subprocess.run([sys.executable, str(APPLIER), str(manifest), "--root", str(root), action], cwd=ROOT, check=True)
    print(f"F0 Fast core candidate stack={len(manifests)}; action={action}; exact hashes=PASS")

def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=ROOT)
    group = parser.add_mutually_exclusive_group(required=True)
    for action in ("--apply", "--check", "--reverse"):
        group.add_argument(action, action="store_true")
    args = parser.parse_args()
    action = next(x for x in ("--apply", "--check", "--reverse") if getattr(args, x[2:].replace('-', '_')))
    try:
        run(action, args.root)
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"error: {error}\n")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
