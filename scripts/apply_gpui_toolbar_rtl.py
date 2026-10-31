#!/usr/bin/env python3
"""Reproduce the pinned GPUI toolbar right-to-left patch."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import tomllib

MANIFEST = Path('patches/approved/gpui-toolbar-rtl-v1.json')
MANIFEST_HASH = '62c4630c2033810b3103cf53eb16433a982c2402ad801a72b13006db4917dd20'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def run(root, artifacts, mode):
    root, artifacts = root.resolve(), artifacts.resolve()
    raw = (artifacts / MANIFEST).read_bytes()
    if digest(raw) != MANIFEST_HASH:
        raise ValueError('toolbar patch metadata hash mismatch')
    meta = json.loads(raw)
    selected = tomllib.loads((root / 'Cargo.toml').read_text())['patch']['crates-io']
    for package in ('gpui-base', 'gpui-component'):
        path = f'vendor/{package}-0.7.3'
        identity = tomllib.loads((root / path / 'Cargo.toml').read_text())['package']
        if (identity.get('name'), identity.get('version'), selected[package].get('path')) != (package, '0.7.3', path):
            raise ValueError(f'local package selection changed: {package}')
    for path, expected in meta['preserved'].items():
        if digest((root / path).read_bytes()) != expected:
            raise ValueError(f'upstream provenance changed: {path}')
    patch = artifacts / meta['patch']
    if digest(patch.read_bytes()) != meta['patch_sha256']:
        raise ValueError('toolbar patch hash mismatch')
    old = {path: (root / path).read_bytes() for path in meta['pre']}
    hashes = {path: digest(data) for path, data in old.items()}
    if hashes == meta['pre']:
        reverse = False
    elif hashes == meta['post']:
        reverse = True
    else:
        raise ValueError('toolbar sources are unknown or partially patched')
    if mode == '--verify' and not reverse:
        raise ValueError('toolbar patch postimage required')
    with tempfile.TemporaryDirectory(prefix='rustui-toolbar-rtl-') as folder:
        stage = Path(folder)
        for path, data in old.items():
            dest = stage / path
            dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_bytes(data)
        command = ['git', 'apply', '--unidiff-zero', '--whitespace=nowarn'] + (['--reverse'] if reverse else [])
        subprocess.run(command + ['--check', str(patch)], cwd=stage, check=True, capture_output=True)
        subprocess.run(command + [str(patch)], cwd=stage, check=True, capture_output=True)
        new = {path: (stage / path).read_bytes() for path in old}
        if {path: digest(data) for path, data in new.items()} != meta['pre' if reverse else 'post']:
            raise ValueError('toolbar replay hash mismatch')
        undo = ['git', 'apply', '--unidiff-zero', '--whitespace=nowarn'] + ([] if reverse else ['--reverse'])
        subprocess.run(undo + [str(patch)], cwd=stage, check=True, capture_output=True)
        if any((stage / path).read_bytes() != data for path, data in old.items()):
            raise ValueError('toolbar patch reversal mismatch')
    if mode in ('--apply', '--reverse'):
        if (mode == '--apply') == reverse:
            raise ValueError('toolbar sources already at requested image')
        if any((root / path).read_bytes() != data for path, data in old.items()):
            raise ValueError('toolbar source changed during staging')
        written = []
        try:
            for path, data in new.items():
                target = root / path
                fd, name = tempfile.mkstemp(prefix='.rustui-toolbar-', dir=target.parent)
                try:
                    os.fchmod(fd, target.stat().st_mode & 0o777)
                    with os.fdopen(fd, 'wb') as output:
                        output.write(data)
                    os.replace(name, target)
                finally:
                    Path(name).unlink(missing_ok=True)
                written.append(path)
        except OSError:
            for path in written:
                (root / path).write_bytes(old[path])
            raise
    print('GPUI toolbar RTL patch verified')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument('--artifacts', type=Path)
    group = parser.add_mutually_exclusive_group(required=True)
    for mode in ('--check', '--apply', '--verify', '--reverse'):
        group.add_argument(mode, action='store_true')
    args = parser.parse_args()
    mode = next(m for m in ('--check', '--apply', '--verify', '--reverse') if getattr(args, m[2:]))
    try:
        run(args.root, args.artifacts or args.root, mode)
    except (ValueError, OSError, subprocess.CalledProcessError, KeyError) as error:
        parser.exit(1, f'error: {error}\n')


if __name__ == '__main__':
    main()
