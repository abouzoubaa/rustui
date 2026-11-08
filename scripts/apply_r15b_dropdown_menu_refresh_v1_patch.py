#!/usr/bin/env python3
from pathlib import Path
from approved_single_source_patch import main
MANIFEST = Path('patches/approved/r15b-dropdown-menu-refresh-v1.json')
MANIFEST_SHA256 = '6682140e18b4295c4b64ed200aed824fad0b8dc3fc7543a449d4bb4818b71442'
if __name__ == "__main__":
    main(MANIFEST, MANIFEST_SHA256)
