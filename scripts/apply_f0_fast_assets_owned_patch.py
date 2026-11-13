#!/usr/bin/env python3
from pathlib import Path
from approved_single_source_patch import main
MANIFEST = Path("patches/candidate-kit-fast/gpui-kit-assets-owned-gpui-pre-optional-v1.json")
MANIFEST_SHA256 = "c872410eed10bbbbdfa2fbe4449d62628b3396bf1362c3d5691bf2b18c82d180"
if __name__ == "__main__":
    main(MANIFEST, MANIFEST_SHA256)
