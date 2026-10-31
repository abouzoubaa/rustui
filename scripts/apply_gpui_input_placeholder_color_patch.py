#!/usr/bin/env python3
"""Apply or verify the pinned GPUI input placeholder-color patch."""
import argparse
import hashlib
from pathlib import Path
import subprocess

PATCH = "patches/approved/gpui-input-placeholder-color-v1.patch"
PATCH_HASH = "59e6e2cc615fa5fd5abf975e997642a507ea3bea447168a022a3ebc4f5afee89"
IMAGES = {'vendor/gpui-base-0.7.3/src/input/editor/highlighting.rs': ['a8ddf22a7e48437da874e5f4f665b0826660a39ea364a564bfdf77e90f218155', '79c38573136ed2745af6db9dd7bd74579ef6c86462a217c851a83fc7e3c9bc8b'], 'vendor/gpui-base-0.7.3/src/input/base/element.rs': ['cdb211ebe96ecfd1625e1b7aa29c8d6eee3f28e012bf9be2edd9b50ed6f7527d', '40770fd8b48df5394d406c34eae857765003b9d05f75435597a7daba8ef23ea5'], 'vendor/gpui-component-0.7.3/src/input/input.rs': ['a07a59963b06006a37f8ed0a2180dd732957b5d1a77d41d4a5cc9d00779c17a1', '0a50716a86b34d44817ec54a4328915d40df2de7054a373ad5d3a3890fd2dfc3'], 'vendor/gpui-base-0.7.3/.cargo-checksum.json': ['9e96d55b0071b60576bc6ac18c3e9c48e6d0de5ff848821af7623bd151f7c3ed', '9e96d55b0071b60576bc6ac18c3e9c48e6d0de5ff848821af7623bd151f7c3ed'], 'vendor/gpui-component-0.7.3/.cargo-checksum.json': ['d5b533efba197cc15bcb6756b935a786e846463c1ee431f876e8c334a7e638c6', 'd5b533efba197cc15bcb6756b935a786e846463c1ee431f876e8c334a7e638c6']}

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    action = parser.add_mutually_exclusive_group(required=True)
    action.add_argument("--apply", action="store_true")
    action.add_argument("--reverse", action="store_true")
    action.add_argument("--check", action="store_true")
    args = parser.parse_args()
    root = args.root.resolve()
    patch = root / PATCH
    if digest(patch) != PATCH_HASH:
        raise SystemExit("placeholder patch hash mismatch")
    expected = 1 if args.check or args.reverse else 0
    for name, images in IMAGES.items():
        if digest(root / name) != images[expected]:
            raise SystemExit(f"unexpected source or checksum image: {name}")
    if not args.check:
        command = ["patch", "--batch", "--fuzz=0", "-p1", "-i", str(patch)]
        if args.reverse:
            command.append("--reverse")
        subprocess.run(command + ["--dry-run"], cwd=root, check=True)
        subprocess.run(command, cwd=root, check=True)
        for name, images in IMAGES.items():
            if digest(root / name) != images[1 - expected]:
                raise SystemExit(f"postimage mismatch: {name}")
    print("verified pinned placeholder patch images")

if __name__ == "__main__":
    main()
