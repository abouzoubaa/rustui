#!/usr/bin/env python3
from pathlib import Path
from approved_single_source_patch import main
MANIFEST = Path('patches/approved/r15b-menu-item-disabled-v1.json')
MANIFEST_SHA256 = '653da81219a984af7a58855332097ac3944724cc9760989bb3f7ee5ca0cf9a5b'
if __name__ == "__main__":
    main(MANIFEST, MANIFEST_SHA256)
