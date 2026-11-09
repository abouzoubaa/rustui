#!/usr/bin/env python3
"""Verify G10's tree relation inspection and the underlying R15 layer without mutation."""
import argparse
from pathlib import Path
import subprocess
import sys
import tempfile
from g10_owned_source_patch import vendor_prior_image
from apply_g10_labelled_by_tree_v1_patch import MANIFEST, MANIFEST_SHA256
ROOT = Path(__file__).resolve().parents[1]
SOURCE = Path("vendor/gpui-pre-0.3.9/src/window/a11y/debug.rs")
def run(script, root, artifacts, mode):
    subprocess.run([sys.executable, str(ROOT / "scripts" / script), "--root", str(root), "--artifacts", str(artifacts), mode], check=True)
def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--artifacts", type=Path, default=ROOT)
    args = parser.parse_args()
    run("apply_g10_labelled_by_tree_v1_patch.py", args.root, args.artifacts, "--verify")
    with tempfile.TemporaryDirectory(prefix="g10-grid-debug-stack-") as tmp:
        stage = Path(tmp)
        source = stage / SOURCE
        source.parent.mkdir(parents=True)
        source.write_bytes(vendor_prior_image(args.root, args.artifacts, MANIFEST, MANIFEST_SHA256))
        run("apply_r15_test_tree_disabled_v1_patch.py", stage, args.artifacts, "--verify")
if __name__ == "__main__":
    main()
