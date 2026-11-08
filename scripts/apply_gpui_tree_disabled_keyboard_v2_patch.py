#!/usr/bin/env python3
from pathlib import Path
from approved_single_source_patch import main

MANIFEST = Path("patches/approved/gpui-tree-disabled-keyboard-v2.json")
MANIFEST_SHA256 = "8efa0776e0f35ef6f6e7e72541af1acb08e649ff45e4fd3fc7abac8c997d1ef5"

if __name__ == "__main__":
    main(MANIFEST, MANIFEST_SHA256)
