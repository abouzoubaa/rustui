#!/usr/bin/env python3
from pathlib import Path
from approved_single_source_patch import main
MANIFEST = Path('patches/approved/r15-button-disabled-focus-v1.json')
MANIFEST_SHA256 = '375adf70ed8ba0c12a200b762f781b33a60421a0d49c49c594d67547c60a36e6'
if __name__ == "__main__":
    main(MANIFEST, MANIFEST_SHA256)
