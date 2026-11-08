#!/usr/bin/env python3
from pathlib import Path
from approved_single_source_patch import main
MANIFEST = Path('patches/approved/r15b-popup-menu-recovery-v1.json')
MANIFEST_SHA256 = '469524c712c0514819b75ba7cef98389f7f0ff787835bc49cba6acd1c5be7825'
if __name__ == "__main__":
    main(MANIFEST, MANIFEST_SHA256)
