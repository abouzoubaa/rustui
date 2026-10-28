#!/usr/bin/env python3
"""Apply or verify the exact gpui-component Web syntax runtime patch."""
import argparse
import hashlib
from pathlib import Path
import subprocess
import tomllib

PATCH = Path("patches/approved/gpui-component-web-syntax-runtime-v3.patch")
PACKAGE = Path("vendor/gpui-component-0.7.3")
PATCH_HASH = "c93c5a3c0c26cc755cdd225a50207c9de6bfee6a712b7fb5dbb79bdee547f24b"
CHECKSUM_HASH = "d5b533efba197cc15bcb6756b935a786e846463c1ee431f876e8c334a7e638c6"
HASHES = {'src/highlighter/registry.rs': ['237d83124bb55031b8d16293bd6e64e590b41dce617a393f39b98c856e32ab31', '1c40eabb10a5bb9b35679f20e84ff11aafadc1f3d6cf4d0a415ca1cfad86acca'], 'src/highlighter/highlighter.rs': ['ffea07b48800410e7a093d7615fc1cacf8481374d3ce155d601930bd8bcf234b', '62b126fb587498bdcf5ae17b0a054d6811a3f14ada2068aabf25d6a52648bb38'], 'src/diff/state.rs': ['a24a4e5846501ec65ca2319c8d26be53c8aa5384058495254c38dddab132c9e3', '4ad6e94eb04cc58946575c183d2f3ecf7d51d1635083074a43941ae0d9dd7ad4'], 'src/input/syntax_context.rs': ['7fb9a64557191822c559943091b451e316ba9b32e0cddb59ebfc04562b440acd', 'f9684895428cf5ad594889fdfa5c20d40cb1bab08854e09115d39709386b745e']}

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    actions = parser.add_mutually_exclusive_group(required=True)
    actions.add_argument("--apply", action="store_true")
    actions.add_argument("--reverse", action="store_true")
    actions.add_argument("--check", action="store_true")
    args = parser.parse_args()
    root = args.root.resolve()
    package = root / PACKAGE
    metadata = tomllib.loads((package / "Cargo.toml").read_text())["package"]
    if metadata.get("name") != "gpui-component" or metadata.get("version") != "0.7.3":
        raise ValueError("expected gpui-component 0.7.3")
    if digest(root / PATCH) != PATCH_HASH or digest(package / ".cargo-checksum.json") != CHECKSUM_HASH:
        raise ValueError("patch or original package checksum mismatch")
    states = [digest(package / name) for name in HASHES]
    before = [pair[0] for pair in HASHES.values()]
    after = [pair[1] for pair in HASHES.values()]
    if args.check:
        if states != after:
            raise ValueError("patched source hash mismatch")
    else:
        expected, result = (after, before) if args.reverse else (before, after)
        if states != result:
            if states != expected:
                raise ValueError("unexpected source hash")
            cmd = ["patch", "--batch", "--fuzz=0", "-p1", "-i", str(root / PATCH)]
            if args.reverse:
                cmd.insert(2, "-R")
            run = subprocess.run(cmd, cwd=root, capture_output=True, text=True)
            if run.returncode or any(x in (run.stdout + run.stderr).lower() for x in ("offset", "fuzz", "failed")):
                raise ValueError("exact patch failed: " + run.stdout + run.stderr)
            if [digest(package / name) for name in HASHES] != result:
                raise ValueError("post-patch source hash mismatch")
    print("verified gpui-component Web syntax runtime")

if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, tomllib.TOMLDecodeError) as error:
        raise SystemExit(f"error: {error}")
