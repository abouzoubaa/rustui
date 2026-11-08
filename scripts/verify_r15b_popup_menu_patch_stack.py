#!/usr/bin/env python3
"""Verify R15b and the underlying P1 disabled-confirm layer with exact replay, without mutation."""
import argparse
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
ROOT = Path(__file__).resolve().parents[1]
SOURCE = Path("vendor/gpui-component-0.7.3/src/menu/popup_menu.rs")
def run(script, root, artifacts, mode):
    subprocess.run([sys.executable, str(ROOT / "scripts" / script), "--root", str(root), "--artifacts", str(artifacts), mode], check=True)
def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--artifacts", type=Path, default=ROOT)
    args = parser.parse_args()
    run("apply_r15b_popup_menu_recovery_v1_patch.py", args.root, args.artifacts, "--verify")
    with tempfile.TemporaryDirectory(prefix="r15b-popup-menu-stack-") as temporary:
        stage = Path(temporary)
        source = stage / SOURCE
        source.parent.mkdir(parents=True)
        shutil.copyfile(args.root / SOURCE, source)
        run("apply_r15b_popup_menu_recovery_v1_patch.py", stage, args.artifacts, "--reverse")
        run("apply_gpui_component_popup_menu_disabled_confirm_v1_patch.py", stage, args.artifacts, "--verify")
if __name__ == "__main__":
    main()
