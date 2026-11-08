#!/usr/bin/env python3
from pathlib import Path
from approved_single_source_patch import main
MANIFEST = Path('patches/approved/r15-tree-disabled-focus-v1.json')
MANIFEST_SHA256 = '24856d85196fa7a05fd9f1da885c5dbdf3e9beb8af298e1267bac352d7cec693'
if __name__ == "__main__":
    main(MANIFEST, MANIFEST_SHA256)
