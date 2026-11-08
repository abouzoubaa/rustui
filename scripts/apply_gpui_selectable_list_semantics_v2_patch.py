#!/usr/bin/env python3
from pathlib import Path
from approved_single_source_patch import main

MANIFEST = Path("patches/approved/gpui-selectable-list-semantics-v2.json")
MANIFEST_SHA256 = "2778616074f6ad3ea7bc06c179e046940a8b8b43012bc850831114f70e439a52"

if __name__ == "__main__":
    main(MANIFEST, MANIFEST_SHA256)
