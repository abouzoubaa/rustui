#!/usr/bin/env bash
set -euo pipefail
repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repository_root"
. scripts/pinned-toolchain.sh
export CARGO_TARGET_DIR="$HOME/.cache/rustui-target"
mkdir -p target/headstart/px4
# Build normal release dependencies once. Reference binaries are explicit local
# artifacts, never Cargo workspace targets or shipped application binaries.
pinned_cargo build -p rustui-lab --release --locked --offline --message-format=json > target/headstart/px4/build-artifacts.jsonl
xcrun swiftc -O apps/latency-ref/appkit.swift -framework AppKit -framework QuartzCore -o target/headstart/px4/latency-appkit
python3 - <<'PY'
import json,os,subprocess
from pathlib import Path
artifacts=[json.loads(line) for line in Path('target/headstart/px4/build-artifacts.jsonl').read_text().splitlines() if line.startswith('{')]
args=[]
for package,alias in [('rustui-gpui-fast#','gpui'),('rustui-gpui-fast-platform#','gpui_platform')]:
 found=[r for r in artifacts if r.get('reason')=='compiler-artifact' and package in r['package_id']]
 paths=[p for r in found for p in r['filenames'] if p.endswith('.rlib')]
 assert len(paths)==1,(package,paths)
 args+=['--extern',alias+'='+paths[0]]
subprocess.run([os.environ['RUSTC'],'--edition=2024','-C','opt-level=3','-C','lto=thin','-C','codegen-units=1','-L','dependency='+os.environ['CARGO_TARGET_DIR']+'/release/deps',*args,'apps/latency-ref/gpui.rs','-o','target/headstart/px4/latency-gpui'],check=True)
PY
