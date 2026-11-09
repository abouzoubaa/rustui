#!/usr/bin/env python3
from pathlib import Path
from approved_single_source_patch import main
MANIFEST = Path('patches/approved/g10-labelled-by-tree-v1.json')
MANIFEST_SHA256 = '3b56d56b3c8abb9acd57b477985804308948c03201cacccc480628b002e9416b'
if __name__ == "__main__":
    main(MANIFEST, MANIFEST_SHA256)
