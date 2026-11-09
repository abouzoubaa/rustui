#!/usr/bin/env python3
from pathlib import Path
from g10_owned_source_patch import main
MANIFEST = Path('patches/approved/g10-labelled-by-tree-v1.json')
MANIFEST_SHA256 = '60bb68cfbf2dbdfd886c3f7ddca1ddfce12996e01e4eacb24cc57a3d2f2a9934'
if __name__ == "__main__":
    main(MANIFEST, MANIFEST_SHA256)
