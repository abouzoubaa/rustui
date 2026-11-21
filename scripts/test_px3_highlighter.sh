#!/usr/bin/env bash
set -euo pipefail
repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repository_root"
. scripts/pinned-toolchain.sh
export CARGO_TARGET_DIR="$HOME/.cache/rustui-target"
# The owned Kit package is intentionally excluded from the workspace. Exercise
# its real private oracle tests in a disposable source copy, without changing
# the owned manifest, its lockfile, or the release workspace.
python3 - <<'PY'
from pathlib import Path
import re,shutil
root=Path.cwd()
source=root/'crates/rustui-gpui-component'
fixture=root/'target/headstart/px3/kit-oracle'
if fixture.exists(): shutil.rmtree(fixture)
shutil.copytree(source,fixture)
# Other upstream unit modules reference files outside the published package.
# Exclude those unrelated tests in the copy; leave the entire real highlighter
# (including its production code and PX-3 oracle module) byte-for-byte intact.
for file in (fixture/'src').rglob('*.rs'):
    if file.parent == fixture/'src/highlighter': continue
    text=file.read_text()
    text=text.replace('#[cfg(test)]', '#[cfg(any())]')
    text=re.sub(r'#\[cfg\(all\(test,[^\n]+\)\)\]', '#[cfg(any())]', text)
    file.write_text(text)
manifest=(fixture/'Cargo.toml').read_text()
manifest=re.sub(r'path = "(\.\./[^"]+)"',lambda m:'path = "'+str((source/m[1]).resolve())+'"',manifest)
patches=(root/'Cargo.toml').read_text().split('[patch.crates-io]\n',1)[1].split('\n[profile.release]',1)[0]
patches=re.sub(r'path = "([^"]+)"',lambda m:'path = "'+str(root/m[1])+'"',patches)
(fixture/'Cargo.toml').write_text(manifest+'\n[workspace]\n\n[patch.crates-io]\n'+patches)
shutil.copy2(root/'Cargo.lock',fixture/'Cargo.lock')
PY
pinned_cargo test --manifest-path target/headstart/px3/kit-oracle/Cargo.toml \
  --lib --no-default-features --features gpui-fast,tree-sitter-rust,gpui-fast/test-support \
  --offline px3_ -- --test-threads=1
