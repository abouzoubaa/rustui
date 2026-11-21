#!/usr/bin/env python3
"""Capture two actual G0 hover states. Run separately from performance timing."""
import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=Path.home()/'.cache/rustui-target/release/rustui-lab')
    parser.add_argument('--output', type=Path, default=ROOT/'target/headstart/verification'/('px2-hover-'+datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')))
    args = parser.parse_args()
    assert args.binary.is_file(), 'build rustui-lab with the pinned toolchain first'
    folder = args.output.resolve()
    folder.mkdir(parents=True)
    helper = folder/'hover-pixels'
    with (folder/'build.log').open('w') as build:
        subprocess.run(['xcrun','swiftc','-parse-as-library',str(ROOT/'scripts/g0_hover_pixels.swift'),'-o',str(helper)],stdout=build,stderr=build,check=True)
    log = folder/'app.log'
    with log.open('w') as output:
        app = subprocess.Popen([str(args.binary.resolve()),'--g0-scene','hover','--duration','4'],
            env=dict(os.environ,RUSTUI_DAMAGE='0',RUSTUI_G0_LATENCY='1',RUSTUI_G0_HOVER_PIXELS='1',RUSTUI_FRAME_LOG='buffered',RUSTUI_FRAME_PROFILE='1'),stdout=output,stderr=output)
        try:
            with (folder/'capture.stdout').open('w') as out, (folder/'capture.stderr').open('w') as err:
                subprocess.run([str(helper),str(app.pid),str(folder),str(log)],stdout=out,stderr=err,check=True,timeout=20)
            assert app.wait(timeout=10) == 0, 'Lab failed'
        finally:
            if app.poll() is None:
                app.terminate()
                app.wait()
    proof = json.loads((folder/'proof.json').read_text())
    assert proof['verified'], proof
    markers, gpu = [], []
    for line in log.read_text().splitlines():
        if line.startswith('RUSTUI_G0_LOG '):
            markers.append(json.loads(line.removeprefix('RUSTUI_G0_LOG ')))
        if line.startswith('RUSTUI_FRAME_LOG '):
            r = json.loads(line.removeprefix('RUSTUI_FRAME_LOG '))
            if r['event'] == 'gpu':
                gpu.append(r)
    assert [m['event'] for m in markers] == ['start','end'] and markers[-1]['valid']
    assert gpu and all(r['mode'] in ('full','direct') for r in gpu), 'pixel proof must run Direct'
    print(json.dumps(proof, indent=2))
    print(f'Captured pixels: {folder}')


if __name__ == '__main__':
    main()
