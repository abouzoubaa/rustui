#!/usr/bin/env python3
"""Read-only compatibility entry point; standalone Tree aria-level mutation is retired."""
from pathlib import Path
import argparse
import runpy


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true")
    mode.add_argument("--apply", action="store_true")
    mode.add_argument("--reverse", action="store_true")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    if not args.check:
        parser.error("standalone aria-level mutation is retired; use apply_gpui_tree_accessibility_series.py --apply/--reverse explicitly")
    root = args.root.resolve()
    series = runpy.run_path(str(root / "scripts/apply_gpui_tree_accessibility_series.py"))
    current = {path: series["file_digest"](root / path) for path in series["FILES"]}
    series["run"](root, "--verify" if current == series["POST"] else "--check")


if __name__ == "__main__":
    main()
