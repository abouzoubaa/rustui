#!/usr/bin/env python3
"""Replay the composed calendar multi-month heading patch against exact package images."""
import argparse
import hashlib
import os
from pathlib import Path
import subprocess
import tempfile
import tomllib

PATCH = Path('patches/approved/gpui-calendar-multi-month-heading-v1.patch')
PATCH_HASH = '0e7f82434cf793f152b1b180a73d1d4c42c97850201f27489c03ecffb8c3ae92'
IMAGES = {
    Path('vendor/gpui-base-0.7.3/src/calendar.rs'): ('17b393d5b86465908ccfc4db78e42905fafd3146f86f1599ba62bc401c1f7318', '109a9ae748c940fb5985f9a2c3bd74878eb43831a990d06e5f908c3e2b23d29d'),
}
PRESERVED = {
    'vendor/gpui-base-0.7.3/.cargo-checksum.json': '9e96d55b0071b60576bc6ac18c3e9c48e6d0de5ff848821af7623bd151f7c3ed',
    'vendor/gpui-component-0.7.3/.cargo-checksum.json': 'd5b533efba197cc15bcb6756b935a786e846463c1ee431f876e8c334a7e638c6',
}

def digest(data):
    return hashlib.sha256(data).hexdigest()

def run(root, artifacts, mode):
    root, artifacts = root.resolve(), artifacts.resolve()
    patches = tomllib.loads((root / 'Cargo.toml').read_text())['patch']['crates-io']
    for name in ('gpui-base', 'gpui-component'):
        package = root / f'vendor/{name}-0.7.3'
        metadata = tomllib.loads((package / 'Cargo.toml').read_text())['package']
        if (metadata['name'], metadata['version'], patches[name]['path']) != (name, '0.7.3', f'vendor/{name}-0.7.3'):
            raise ValueError(f'{name} package selection mismatch')
    for path, expected in PRESERVED.items():
        if digest((root / path).read_bytes()) != expected:
            raise ValueError(f'upstream metadata changed: {path}')
    patch = artifacts / PATCH
    if digest(patch.read_bytes()) != PATCH_HASH:
        raise ValueError('calendar multi-month heading patch hash mismatch')
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
        raise ValueError('calendar multi-month heading source images mismatch')
    reverse = hashes == post
    if mode == '--verify' and not reverse:
        raise ValueError('calendar multi-month heading postimage required')
    if mode == '--apply' and reverse or mode == '--reverse' and not reverse:
        raise ValueError('calendar multi-month heading source already has requested image')
    with tempfile.TemporaryDirectory(prefix='rustui-calendar-heading-') as folder:
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
            raise ValueError('calendar multi-month heading replay image mismatch')
        undo = ['git', 'apply', '--whitespace=nowarn'] + ([] if reverse else ['--reverse'])
        subprocess.run(undo + [str(patch)], cwd=stage, check=True, capture_output=True)
        if any((stage / p).read_bytes() != originals[p] for p in IMAGES):
            raise ValueError('calendar multi-month heading reversal mismatch')
    if mode != '--verify':
        if any((root / p).read_bytes() != originals[p] for p in IMAGES):
            raise ValueError('calendar sources changed during staging')
        staged = {}
        replaced = []
        try:
            # Prepare every replacement before changing either source.
            for path in IMAGES:
                target = root / path
                fd, name = tempfile.mkstemp(prefix='.rustui-calendar-heading-', dir=target.parent)
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
    print('calendar multi-month heading patch replay verified')

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    for option in ('--verify', '--apply', '--reverse'):
        group.add_argument(option, action='store_const', dest='mode', const=option)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument('--artifacts', type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    run(args.root, args.artifacts, args.mode)
