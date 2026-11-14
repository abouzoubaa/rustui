#!/usr/bin/env python3
from pathlib import Path
from approved_single_source_patch import main
MANIFEST = Path("patches/candidate-kit-fast/gpui-base-vendor-gpui-pre-optional-v1.json")
MANIFEST_SHA256 = "b403563a4f6e2ccd7570ebd542a93897b48ab1e60d960e1d2ccff224afeef9fa"
if __name__ == "__main__":
    main(MANIFEST, MANIFEST_SHA256)
