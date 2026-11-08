#!/usr/bin/env python3
from pathlib import Path
from approved_single_source_patch import main
MANIFEST = Path('patches/approved/r15-sidebar-disabled-v1.json')
MANIFEST_SHA256 = '41fb478578a23475c209bd2d53fe0060e37ac2b15a5141ee60409dcbfd562b32'
if __name__ == "__main__":
    main(MANIFEST, MANIFEST_SHA256)
