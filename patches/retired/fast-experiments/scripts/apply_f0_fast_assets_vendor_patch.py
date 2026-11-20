#!/usr/bin/env python3
from pathlib import Path
from approved_single_source_patch import main
MANIFEST = Path("patches/candidate-kit-fast/gpui-kit-assets-vendor-gpui-pre-optional-v1.json")
MANIFEST_SHA256 = "2d4bfbc6f6c2662d501c7c31f72c944281311a18a1e4fb3a0020a8532dfb2b20"
if __name__ == "__main__":
    main(MANIFEST, MANIFEST_SHA256)
