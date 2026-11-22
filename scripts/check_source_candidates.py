#!/usr/bin/env python3
"""Explicit integration source verification; approved-only release checks stay unchanged."""
import argparse
from pathlib import Path
import shlex
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def integration_body(source: str, candidate_stack: Path) -> str:
    body = source.split("  source)\n", 1)[1].split("\n    ;;", 1)[0]
    approved = "python3 scripts/apply_gpui_kit_09_stack.py --check"
    if body.count(approved) != 1:
        raise ValueError("source verifier changed; expected exactly one approved Kit check")
    return body.replace(approved, approved + " --candidate-stack " + shlex.quote(str(candidate_stack)), 1)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate-stack", type=Path, required=True)
    args = parser.parse_args()
    body = integration_body((ROOT / "scripts/verify-release.sh").read_text(), args.candidate_stack.resolve())
    # Every live source check is reused verbatim; only the explicitly authorized
    # Kit candidate option differs. No release script or CI default is rewritten.
    header = "set -euo pipefail\ncd " + shlex.quote(str(ROOT)) + "\n. scripts/pinned-toolchain.sh\n"
    return subprocess.run(["bash", "-c", header + body], cwd=ROOT).returncode


if __name__ == "__main__":
    raise SystemExit(main())
