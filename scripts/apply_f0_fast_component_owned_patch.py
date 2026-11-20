#!/usr/bin/env python3
from pathlib import Path
from approved_single_source_patch import main
MANIFEST = Path("patches/approved/fast/gpui-component-owned-gpui-pre-optional-v1.json")
MANIFEST_SHA256 = "5047d4d9bf97ad8d7644b59d00f724dc077bb78e5d0c9a7c958a9220f1878b77"
if __name__ == "__main__":
    main(MANIFEST, MANIFEST_SHA256)
