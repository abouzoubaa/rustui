#!/usr/bin/env python3
from pathlib import Path
from approved_single_source_patch import main
MANIFEST = Path("patches/candidate-kit-fast/gpui-base-owned-gpui-pre-optional-v1.json")
MANIFEST_SHA256 = "7714bcc1387f881665615e5d6819af7f220985263f13e84f2cca5e96d51c9761"
if __name__ == "__main__":
    main(MANIFEST, MANIFEST_SHA256)
