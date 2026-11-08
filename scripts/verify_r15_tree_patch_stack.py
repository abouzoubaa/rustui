#!/usr/bin/env python3
"""Verify R15 and the underlying P2 Tree layer with exact replay, without mutation."""
import argparse
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
ROOT = Path(__file__).resolve().parents[1]
SOURCE = Path("vendor/gpui-base-0.7.3/src/tree.rs")
def run(script, root, artifacts, mode):
    subprocess.run([sys.executable, str(ROOT / "scripts" / script), "--root", str(root), "--artifacts", str(artifacts), mode], check=True)
def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--artifacts", type=Path, default=ROOT)
    args = parser.parse_args()
    run("apply_r15_tree_disabled_focus_v1_patch.py", args.root, args.artifacts, "--verify")
    with tempfile.TemporaryDirectory(prefix="r15-tree-stack-") as temporary:
        stage = Path(temporary)
        source = stage / SOURCE
        source.parent.mkdir(parents=True)
        shutil.copyfile(args.root / SOURCE, source)
        run("apply_r15_tree_disabled_focus_v1_patch.py", stage, args.artifacts, "--reverse")
        run("apply_gpui_tree_disabled_keyboard_v2_patch.py", stage, args.artifacts, "--verify")
if __name__ == "__main__":
    main()
