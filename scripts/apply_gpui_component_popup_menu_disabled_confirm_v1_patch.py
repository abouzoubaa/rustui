#!/usr/bin/env python3
from pathlib import Path
from approved_single_source_patch import main

if __name__ == "__main__":
    main(Path("patches/approved/gpui-component-popup-menu-disabled-confirm-v1.json"), "3c681bbf8430e476a49d63444ac3dcf54b97463d76754e2840342a82f496238c")
