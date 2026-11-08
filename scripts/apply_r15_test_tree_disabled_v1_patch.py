#!/usr/bin/env python3
from pathlib import Path
from approved_single_source_patch import main
MANIFEST = Path('patches/approved/r15-test-tree-disabled-v1.json')
MANIFEST_SHA256 = '63fdc55a25ee7c2db64e48dc72afd44260fda8c6e3477a125c77d95defa0f425'
if __name__ == "__main__":
    main(MANIFEST, MANIFEST_SHA256)
