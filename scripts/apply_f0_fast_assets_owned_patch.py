#!/usr/bin/env python3
from pathlib import Path
from approved_single_source_patch import main
MANIFEST = Path("patches/approved/fast/gpui-kit-assets-owned-gpui-pre-optional-v1.json")
MANIFEST_SHA256 = "92ebad8bad9e8f72724d37c63aa3ae8adaa664346d0fd97b01f0fbe72aa15ceb"
if __name__ == "__main__":
    main(MANIFEST, MANIFEST_SHA256)
