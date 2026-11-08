#!/usr/bin/env python3
"""Replay the composed select popup RTL patch against exact package images."""
import argparse
import hashlib
import os
from pathlib import Path
import subprocess
import tempfile
import tomllib

PATCH = Path("patches/approved/gpui-component-select-popup-rtl-v1.patch")
PATCH_HASH = "d1f90bd4e273b2a293b4cd3416b2dfac34fb3be84d2c31a238549c834377e081"
IMAGES = {
    Path("vendor/gpui-component-0.7.3/src/list/list.rs"): ("5010322282b37829c974b105050ec8a8505df442e621724ea7c4fa3fa544a453", "b7cc17c9e01a804d42da1dc55154fe8bccece5db64cef7a78f296dc4d4034edd"),
    Path("vendor/gpui-component-0.7.3/src/searchable_list/adapter.rs"): ("f53fdfb62cd08c2f15945c868906dddecfd3af6ef8846a9715eee3ae970b2899", "e9f8e3c72eb0450a6aaef7b78b05ca16d7c843cf11578054399c1326ee9d80ce"),
    Path("vendor/gpui-component-0.7.3/src/searchable_list/item.rs"): ("4bd6b6973feb7444ace040468d58bf4b3b30d5d2c313c024d86ecde473066ae3", "649d172a451a0f288bd3a89034010e7231e4238fb35950b8ee49992dbe46d82f"),
}
PRESERVED = {
    "vendor/gpui-component-0.7.3/Cargo.toml": "192f4156aee8738567c748d6b1c5bc44c80e4ba5369a41e5349e370d83b35026",
    "vendor/gpui-component-0.7.3/.cargo-checksum.json": "d5b533efba197cc15bcb6756b935a786e846463c1ee431f876e8c334a7e638c6",
}


def digest(data):
    return hashlib.sha256(data).hexdigest()

def run(root, artifacts, mode):
    root, artifacts = root.resolve(), artifacts.resolve()
    patches = tomllib.loads((root / 'Cargo.toml').read_text())['patch']['crates-io']
    for name in ('gpui-component',):
        package = root / f'vendor/{name}-0.7.3'
        metadata = tomllib.loads((package / 'Cargo.toml').read_text())['package']
        if (metadata['name'], metadata['version'], patches[name]['path']) != (name, '0.7.3', f'vendor/{name}-0.7.3'):
            raise ValueError(f'{name} package selection mismatch')
    for path, expected in PRESERVED.items():
        if digest((root / path).read_bytes()) != expected:
            raise ValueError(f'upstream metadata changed: {path}')
    patch = artifacts / PATCH
    if digest(patch.read_bytes()) != PATCH_HASH:
        raise ValueError('select popup RTL patch hash mismatch')
    originals = {}
    for path in IMAGES:
        source = root / path
        if source.is_symlink():
            raise ValueError(f'source is a symlink: {path}')
        originals[path] = source.read_bytes()
    hashes = tuple(digest(originals[p]) for p in IMAGES)
    pre = tuple(v[0] for v in IMAGES.values())
    post = tuple(v[1] for v in IMAGES.values())
    if hashes not in (pre, post):
        raise ValueError('select popup RTL source images mismatch')
    reverse = hashes == post
    if mode == '--verify' and not reverse:
        raise ValueError('select popup RTL postimage required')
    if mode == '--apply' and reverse or mode == '--reverse' and not reverse:
        raise ValueError('select popup RTL source already has requested image')
    with tempfile.TemporaryDirectory(prefix='rustui-select-popup-rtl-') as folder:
        stage = Path(folder)
        for path, data in originals.items():
            target = stage / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        cmd = ['git', 'apply', '--whitespace=nowarn'] + (['--reverse'] if reverse else [])
        subprocess.run(cmd + ['--check', str(patch)], cwd=stage, check=True, capture_output=True)
        subprocess.run(cmd + [str(patch)], cwd=stage, check=True, capture_output=True)
        updated = {path: (stage / path).read_bytes() for path in IMAGES}
        if tuple(digest(updated[p]) for p in IMAGES) != (pre if reverse else post):
            raise ValueError('select popup RTL replay image mismatch')
        undo = ['git', 'apply', '--whitespace=nowarn'] + ([] if reverse else ['--reverse'])
        subprocess.run(undo + [str(patch)], cwd=stage, check=True, capture_output=True)
        if any((stage / p).read_bytes() != originals[p] for p in IMAGES):
            raise ValueError('select popup RTL reversal mismatch')
    if mode != '--verify':
        if any((root / p).read_bytes() != originals[p] for p in IMAGES):
            raise ValueError('calendar sources changed during staging')
        staged = {}
        replaced = []
        try:
            # Prepare every replacement before changing either source.
            for path in IMAGES:
                target = root / path
                fd, name = tempfile.mkstemp(prefix='.rustui-select-popup-rtl-', dir=target.parent)
                staged[path] = name
                os.fchmod(fd, target.stat().st_mode & 0o777)
                with os.fdopen(fd, 'wb') as output:
                    output.write(updated[path])
            if any((root / p).read_bytes() != originals[p] for p in IMAGES):
                raise ValueError('calendar sources changed during staging')
            for path in IMAGES:
                os.replace(staged[path], root / path)
                replaced.append(path)
        except Exception:
            for path in reversed(replaced):
                target = root / path
                fd, name = tempfile.mkstemp(prefix='.rustui-calendar-restore-', dir=target.parent)
                try:
                    os.fchmod(fd, target.stat().st_mode & 0o777)
                    with os.fdopen(fd, 'wb') as output:
                        output.write(originals[path])
                    os.replace(name, target)
                finally:
                    if os.path.exists(name):
                        os.unlink(name)
            raise
        finally:
            for name in staged.values():
                if os.path.exists(name):
                    os.unlink(name)
    print('select popup RTL patch replay verified')

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    for option in ('--verify', '--apply', '--reverse'):
        group.add_argument(option, action='store_const', dest='mode', const=option)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument('--artifacts', type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    run(args.root, args.artifacts, args.mode)
