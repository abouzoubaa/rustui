#!/usr/bin/env python3
from pathlib import Path
from approved_single_source_patch import main
MANIFEST = Path("patches/candidate-kit-fast/gpui-component-owned-gpui-pre-optional-v1.json")
MANIFEST_SHA256 = "891fda7e305ec8ed43563d39dba79373edaffecc016fcaa9192cceeb125eb3b9"
if __name__ == "__main__":
    main(MANIFEST, MANIFEST_SHA256)
