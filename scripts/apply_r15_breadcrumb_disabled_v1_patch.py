#!/usr/bin/env python3
from pathlib import Path
from approved_single_source_patch import main
MANIFEST = Path('patches/approved/r15-breadcrumb-disabled-v1.json')
MANIFEST_SHA256 = '590e939ccce141982e23562c6ab7bd1ed0ea7227a90450b2fcc864d8b942a993'
if __name__ == "__main__":
    main(MANIFEST, MANIFEST_SHA256)
