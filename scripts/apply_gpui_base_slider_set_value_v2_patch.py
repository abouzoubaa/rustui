#!/usr/bin/env python3
from pathlib import Path
from approved_single_source_patch import main

if __name__ == "__main__":
    main(Path("patches/approved/gpui-base-slider-set-value-v2.json"), "2bb7c2ca54ea2cf158818282b1a3574a3e683e5e2dfe51c3e358c247296f2c0c")
