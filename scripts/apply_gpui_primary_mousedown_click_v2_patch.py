#!/usr/bin/env python3
from pathlib import Path
from approved_single_source_patch import main

MANIFEST = Path("patches/approved/gpui-primary-mousedown-click-v2.json")
MANIFEST_SHA256 = "0eac43e8449f2a84699a698a5fc4d0c81c045ea12c9983c4ea34ec6f989cdc7d"

if __name__ == "__main__":
    main(MANIFEST, MANIFEST_SHA256)
