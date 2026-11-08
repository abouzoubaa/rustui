#!/usr/bin/env python3
"""Check the approved disabled-tree keyboard patch without modifying source."""
import argparse
from pathlib import Path
import subprocess

from approved_single_source_patch import run
from apply_gpui_tree_disabled_keyboard_v2_patch import MANIFEST, MANIFEST_SHA256


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    try:
        run(args.root, args.root, MANIFEST, MANIFEST_SHA256, "--check")
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"error: {error}\n")


if __name__ == "__main__":
    main()
