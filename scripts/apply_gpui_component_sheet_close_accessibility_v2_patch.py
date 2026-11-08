#!/usr/bin/env python3
from pathlib import Path
from approved_single_source_patch import main

if __name__ == "__main__":
    main(Path("patches/approved/gpui-component-sheet-close-accessibility-v2.json"), "3a0412d037e3bdb2177c8fa9fcc3bbce5dd01fc5235d48703d28ac3e3823cceb")
