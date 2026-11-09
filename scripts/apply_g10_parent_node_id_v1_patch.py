#!/usr/bin/env python3
from pathlib import Path
from g10_owned_source_patch import main
MANIFEST = Path('patches/approved/g10-parent-node-id-v1.json')
MANIFEST_SHA256 = '6f3935331ff339940be611f1660828d2b1a166bcd1828a6531ddaabc4817f271'
if __name__ == "__main__":
    main(MANIFEST, MANIFEST_SHA256)
