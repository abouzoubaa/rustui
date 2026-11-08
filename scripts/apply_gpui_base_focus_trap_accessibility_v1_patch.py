#!/usr/bin/env python3
from pathlib import Path
from approved_single_source_patch import main

if __name__ == "__main__":
    main(Path("patches/approved/gpui-base-focus-trap-accessibility-v1.json"), "a12ae831caedc11201462b1cd560ac4de36197057254651ee72f1da36920dee4")
