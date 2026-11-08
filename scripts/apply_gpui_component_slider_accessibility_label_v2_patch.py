#!/usr/bin/env python3
from pathlib import Path
from approved_single_source_patch import main

if __name__ == "__main__":
    main(Path("patches/approved/gpui-component-slider-accessibility-label-v2.json"), "836588600d8691376782c8458104a37b869e775e59c7804c835b9d88e7f0a201")
