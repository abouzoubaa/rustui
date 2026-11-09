#!/usr/bin/env python3
from pathlib import Path
from approved_single_source_patch import main
MANIFEST = Path('patches/approved/g10-parent-node-id-v1.json')
MANIFEST_SHA256 = 'f06daca5fe6579cce5ab9ed1f6e5ee41d738d227ed9b35e6419105ed71bb42e9'
if __name__ == "__main__":
    main(MANIFEST, MANIFEST_SHA256)
