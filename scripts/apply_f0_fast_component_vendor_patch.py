#!/usr/bin/env python3
from pathlib import Path
from approved_single_source_patch import main
MANIFEST = Path("patches/candidate-kit-fast/gpui-component-vendor-gpui-pre-optional-v1.json")
MANIFEST_SHA256 = "904b4edec6e0f645eb08d915f2b4ef86b5f5347073b39daa8a4d24979c4da3c1"
if __name__ == "__main__":
    main(MANIFEST, MANIFEST_SHA256)
