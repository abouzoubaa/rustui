#!/usr/bin/env python3
from pathlib import Path
from approved_single_source_patch import main
MANIFEST = Path("patches/approved/fast/gpui-base-owned-gpui-pre-optional-v1.json")
MANIFEST_SHA256 = "7c84d33f52d29c1bc47e676bb581c77e97bc142d0259bf5c193dbb97140ddaf7"
if __name__ == "__main__":
    main(MANIFEST, MANIFEST_SHA256)
