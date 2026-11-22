#!/usr/bin/env bash
set -euo pipefail
repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repository_root"
. scripts/pinned-toolchain.sh
export CARGO_TARGET_DIR="$HOME/.cache/rustui-target"
mkdir -p target/headstart/g5-1
CARGO_PROFILE_RELEASE_DEBUG=line-tables-only CARGO_PROFILE_RELEASE_STRIP=none pinned_cargo build -p rustui-lab --release --locked --offline --message-format=json > target/headstart/g5-1/build-artifacts.jsonl
xcrun swiftc -O apps/latency-ref/metal.swift -framework AppKit -framework Metal -framework QuartzCore -o target/headstart/g5-1/latency-metal
mkdir -p target/headstart/g5-1/appkit-build
cp apps/latency-ref/appkit_display.swift target/headstart/g5-1/appkit-build/main.swift
xcrun swiftc -O apps/latency-ref/pixel_capture.swift target/headstart/g5-1/appkit-build/main.swift -framework AppKit -framework ScreenCaptureKit -framework CoreMedia -framework CoreVideo -framework QuartzCore -o target/headstart/g5-1/latency-appkit-display
xcrun swiftc -O -emit-library -module-name G5Pixels apps/latency-ref/pixel_capture.swift -framework AppKit -framework ScreenCaptureKit -framework CoreMedia -framework CoreVideo -framework QuartzCore -Xlinker -install_name -Xlinker @rpath/libG5Pixels.dylib -o target/headstart/g5-1/libG5Pixels.dylib
xcrun clang -O2 -fobjc-arc -DG5_REFRESH_AUTOSTART -dynamiclib apps/latency-ref/refresh.m -framework AppKit -framework QuartzCore -o target/headstart/g5-1/libG5Refresh.dylib
python3 - <<'PY'
import json,os,subprocess
from pathlib import Path
p=Path('target/headstart/g5-1');r=[json.loads(x) for x in (p/'build-artifacts.jsonl').read_text().splitlines() if x.startswith('{')];args=[]
for package,alias in [('rustui-gpui-fast#','gpui'),('rustui-gpui-fast-platform#','gpui_platform'),('#libc@0.2.190','libc')]:
 a=[a for a in r if a.get('reason')=='compiler-artifact' and package in a['package_id'] and a['profile']['opt_level']=='3']
 paths=[v for a in a for v in a['filenames'] if v.endswith('.rlib')];assert len(paths)==1,(package,paths);args+=['--extern',alias+'='+paths[0]]
subprocess.run(['xcrun','clang','-O2','-fobjc-arc','-c','apps/latency-ref/refresh.m','-o',str(p/'refresh.o')],check=True)
subprocess.run([os.environ['RUSTC'],'--edition=2024','-C','opt-level=3','-C','debuginfo=line-tables-only','-C','lto=thin','-C','codegen-units=1','-L','dependency='+os.environ['CARGO_TARGET_DIR']+'/release/deps','-C','link-arg='+str(p/'refresh.o'),'-l','framework=AppKit','-l','framework=QuartzCore','-L','native='+str(p.resolve()),'-l','dylib=G5Pixels','-C','link-arg=-Wl,-rpath,'+str(p.resolve()),*args,'apps/latency-ref/g5_gpui.rs','-o',str(p/'latency-gpui')],check=True)
PY
python3 - <<'PYBUNDLE'
from pathlib import Path
import hashlib,plistlib,shutil
root=Path('target/headstart/g5-1');bundle=root/'G5 Bare.app'/'Contents'
(bundle/'MacOS').mkdir(parents=True,exist_ok=True)
source=root/'latency-gpui';dest=bundle/'MacOS/latency-gpui';shutil.copy2(source,dest)
(bundle/'Info.plist').write_bytes(plistlib.dumps({'CFBundleIdentifier':'dev.rustui.latency.g5.bare','CFBundleName':'G5 Bare','CFBundleExecutable':'latency-gpui','CFBundlePackageType':'APPL','CFBundleVersion':'1','NSHighResolutionCapable':True}))
assert hashlib.sha256(source.read_bytes()).digest()==hashlib.sha256(dest.read_bytes()).digest()
PYBUNDLE
