#!/usr/bin/env python3
"""Verify and replay the pinned gpui-base calendar date-limit patch."""
import argparse
import hashlib
import os
from pathlib import Path
import subprocess
import tempfile
import tomllib

SOURCE = Path('vendor/gpui-base-0.7.3/src/calendar.rs')
PATCH = Path('patches/approved/gpui-base-calendar-date-limits-v1.patch')
PRE = '23f747d81667c1f68a54ca827863d556e167fe500a25eec4049d1c90a7d3cd6e'
POST = '13c8d334e1eeb32b0960e84dad834e54c7d8737a1026356d2f5bc86bac189d6c'
PATCH_HASH = '9de49182541a35f6127215e232ab5053eb52a16ad77679620adc721e1ee45bc9'
PRESERVED = {
    'vendor/gpui-base-0.7.3/.cargo-checksum.json': '9e96d55b0071b60576bc6ac18c3e9c48e6d0de5ff848821af7623bd151f7c3ed',
    'vendor/gpui-base-0.7.3/LICENSE-APACHE': 'd1b0449e5478c574ba4f686c2656df7fe77d66821a61f8b6ed3378a58ed9a811',
}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def run(root, artifacts, mode):
    root, artifacts = root.resolve(), artifacts.resolve()
    package = tomllib.loads((root / 'vendor/gpui-base-0.7.3/Cargo.toml').read_text())['package']
    selected = tomllib.loads((root / 'Cargo.toml').read_text())['patch']['crates-io']['gpui-base']
    if (package.get('name'), package.get('version'), selected.get('path')) != ('gpui-base', '0.7.3', 'vendor/gpui-base-0.7.3'):
        raise ValueError('pinned local gpui-base package selection changed')
    for path, expected in PRESERVED.items():
        if digest((root / path).read_bytes()) != expected:
            raise ValueError(f'upstream provenance changed: {path}')
    patch = artifacts / PATCH
    if digest(patch.read_bytes()) != PATCH_HASH:
        raise ValueError('calendar patch hash mismatch')
    source = root / SOURCE
    old = source.read_bytes()
    current = digest(old)
    if current not in (PRE, POST):
        raise ValueError('calendar source is not a known preimage or postimage')
    reverse = current == POST
    if mode == '--verify' and not reverse:
        raise ValueError('calendar patch postimage required')
    with tempfile.TemporaryDirectory(prefix='rustui-calendar-patch-') as folder:
        stage = Path(folder)
        staged = stage / SOURCE
        staged.parent.mkdir(parents=True)
        staged.write_bytes(old)
        command = ['git', 'apply', '--unidiff-zero', '--whitespace=nowarn'] + (['--reverse'] if reverse else [])
        subprocess.run(command + ['--check', str(patch)], cwd=stage, check=True, capture_output=True)
        subprocess.run(command + [str(patch)], cwd=stage, check=True, capture_output=True)
        new = staged.read_bytes()
        if digest(new) != (PRE if reverse else POST):
            raise ValueError('calendar patch replay hash mismatch')
        undo = ['git', 'apply', '--unidiff-zero', '--whitespace=nowarn'] + ([] if reverse else ['--reverse'])
        subprocess.run(undo + [str(patch)], cwd=stage, check=True, capture_output=True)
        if staged.read_bytes() != old:
            raise ValueError('calendar patch reversal mismatch')
    if mode in ('--apply', '--reverse'):
        if (mode == '--apply') == reverse:
            raise ValueError('calendar source already at requested image')
        if source.read_bytes() != old:
            raise ValueError('calendar source changed during staging')
        fd, name = tempfile.mkstemp(prefix='.rustui-calendar-', dir=source.parent)
        try:
            os.fchmod(fd, source.stat().st_mode & 0o777)
            with os.fdopen(fd, 'wb') as output:
                output.write(new)
            if source.read_bytes() != old:
                raise ValueError('calendar source changed during staging')
            os.replace(name, source)
        finally:
            Path(name).unlink(missing_ok=True)
    print('gpui-base calendar date-limit patch verified')


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
