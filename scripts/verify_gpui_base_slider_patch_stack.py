#!/usr/bin/env python3
"""Verify the composed slider patches without modifying the selected source."""
import argparse
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SOURCE = Path("vendor/gpui-base-0.7.3/src/slider.rs")
FIRST = ROOT / "scripts/apply_gpui_base_slider_first_move.py"
ZERO = ROOT / "scripts/apply_gpui_base_slider_zero_track.py"
SNAP = ROOT / "scripts/apply_gpui_base_slider_snap_position.py"
ACCESSIBILITY = ROOT / "scripts/apply_gpui_base_slider_set_value_v2_patch.py"
ENDPOINT = ROOT / "scripts/apply_gpui_base_slider_endpoint.py"


def run(script, root, artifacts, mode):
    subprocess.run(
        [sys.executable, str(script), "--root", str(root), "--artifacts", str(artifacts), mode],
        check=True,
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--artifacts", type=Path, default=ROOT)
    args = parser.parse_args()
    source_root = args.root.resolve()
    artifacts = args.artifacts.resolve()
    run(ACCESSIBILITY, source_root, artifacts, "--verify")
    with tempfile.TemporaryDirectory(prefix="rustui-slider-stack-") as directory:
        root = Path(directory)
        source = root / SOURCE
        source.parent.mkdir(parents=True)
        shutil.copyfile(source_root / SOURCE, source)
        shutil.copyfile(source_root / "vendor/gpui-base-0.7.3/Cargo.toml", source.parent.parent / "Cargo.toml")
        shutil.copyfile(source_root / "Cargo.toml", root / "Cargo.toml")
        run(ACCESSIBILITY, root, artifacts, "--reverse")
        run(ENDPOINT, root, artifacts, "--verify")
        run(ENDPOINT, root, artifacts, "--reverse")
        run(SNAP, root, artifacts, "--verify")
        run(SNAP, root, artifacts, "--reverse")
        run(FIRST, root, artifacts, "--verify")
        run(FIRST, root, artifacts, "--reverse")
        run(ZERO, root, artifacts, "--verify")


if __name__ == "__main__":
    main()
