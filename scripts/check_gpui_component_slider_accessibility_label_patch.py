#!/usr/bin/env python3
"""Check the promoted Slider label patch against its exact approved images."""
from pathlib import Path
import subprocess
import sys

if __name__ == "__main__":
    raise SystemExit(subprocess.call([
        sys.executable,
        str(Path(__file__).with_name("apply_gpui_component_slider_accessibility_label_v2_patch.py")),
        "--check", *sys.argv[1:],
    ]))
