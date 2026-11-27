#!/usr/bin/env bash
# Exercise the excluded, patched Apple renderer against the repository's GPUI packages.
set -euo pipefail
cd "$(dirname "$0")/.."
. scripts/pinned-toolchain.sh
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/.cache/rustui-target}"
fixture=target/headstart/g3-metal-proofs
mkdir -p "$fixture/src"
cp Cargo.lock "$fixture/Cargo.lock"
python3 - "$fixture" <<'PY'
import json
from pathlib import Path
import sys
import tomllib

root = Path.cwd()
manifest = tomllib.loads((root / "Cargo.toml").read_text())
patches = manifest["patch"]["crates-io"]
paths = {name: json.dumps(str(root / package["path"])) for name, package in patches.items()}
lines = [
    "[workspace]", "[package]", 'name = "rustui-g3-metal-proofs"',
    'version = "0.0.0"', 'edition = "2024"', "[dependencies]",
    'gpui_apple = { package = "gpui-fast-apple", path = ' + paths["gpui-fast-apple"] +
    ', features = ["runtime_shaders", "test-support"] }', "[patch.crates-io]",
]
for name, path in paths.items():
    lines.append(f'{name} = {{ path = {path} }}')
(Path(sys.argv[1]) / "Cargo.toml").write_text("\n".join(lines) + "\n")
PY
cat > "$fixture/src/main.rs" <<'RS'
use gpui_apple::metal_renderer::MetalRenderer;
fn main() {
    MetalRenderer::test_damage_shader_pixels().unwrap();
    MetalRenderer::test_damage_planner();
    MetalRenderer::test_damage_frames().unwrap();
    MetalRenderer::test_damage_partials().unwrap();
    MetalRenderer::test_damage_verify().unwrap();
    MetalRenderer::test_damage_cpu_budget().unwrap();
    MetalRenderer::test_damage_direct_policy().unwrap();
    MetalRenderer::test_damage_backoff().unwrap();
    MetalRenderer::test_damage_encoding_metrics();
    println!("Nine native Metal proofs passed");
    MetalRenderer::test_scroll_shift_frames().unwrap();
}
RS
RUSTUI_DAMAGE=0 RUSTUI_DAMAGE_SCROLL_SHIFT=0 pinned_cargo run --offline --manifest-path "$fixture/Cargo.toml"
