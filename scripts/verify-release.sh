#!/usr/bin/env bash
# Source/build checks only. Native/browser/AX/performance acceptance is separate.
set -euo pipefail
repository_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repository_root"

verify_tree_accessibility_series() {
  python3 scripts/apply_gpui_tree_accessibility_series.py --verify
  python3 scripts/apply_gpui_tree_aria_disabled_patch.py --check
}

# Exercise the actual dependency-free Web edit planner on the host toolchain.
# GPUI/Wasm compilation and browser acceptance remain separate checks.
verify_web_ime_edit_plan() (
  binary_dir="$repository_root/target/headstart/verification/web-ime-edit-plan"
  mkdir -p "$binary_dir"
  test_binary="$binary_dir/ime_edit_tests"
  trap 'rm -f "$test_binary"' EXIT
  rustup run 1.97.1 rustc --edition 2024 --test \
    vendor/gpui-pre-web-0.3.9/src/ime_edit.rs -o "$test_binary"
  "$test_binary"
)

case "${1:-}" in
  source)
    verify_tree_accessibility_series
    python3 scripts/apply_gpui_web_semantic_adapter.py --verify
  python3 scripts/apply_gpui_base_button_disabled_patch.py --verify
  python3 scripts/apply_gpui_base_disabled_semantic_roots.py --verify
    python3 -m unittest discover -s scripts/tests -p 'test_*.py'
    node --test scripts/tests/test_gpui_web_semantic_dom.mjs
    sh scripts/tests/test_package_playground_app.sh
    sh scripts/tests/test_build_playground_web.sh
    sh scripts/tests/test_verify_ci_checkout.sh
    python3 scripts/validate_locales.py
    python3 scripts/validate_themes.py --plans-only
    python3 scripts/verify_theme_adapter_coverage.py
    ;;
  macos)
    verify_tree_accessibility_series
    python3 scripts/apply_gpui_web_semantic_adapter.py --verify
    if [[ "$(uname -s)" != Darwin ]]; then
      echo 'The macos release check requires a macOS host.' >&2
      exit 2
    fi
    rustup run 1.97.1 cargo fmt --all -- --check
    python3 scripts/verify_dependency_boundary.py
    ./scripts/cargo-headstart check --workspace --all-targets --profile headstart --locked --offline
    ./scripts/cargo-headstart test --workspace --lib --profile headstart --locked --offline
    ./scripts/cargo-headstart test --workspace --test '*' --profile headstart --locked --offline
    ./scripts/cargo-headstart test -p rustui-macros --doc --profile headstart --locked --offline
    evidence="$repository_root/target/headstart/evidence/release-checks"
    mkdir -p "$evidence"
    ./scripts/cargo-headstart metadata --format-version 1 --locked --offline > "$evidence/native-metadata.json"
    ./scripts/cargo-headstart build -p rustui-playground --release --locked --offline \
      --message-format=json > "$evidence/native-build.jsonl"
    notices="$repository_root/target/headstart/evidence/native-dependency-notices-checked-$(date -u +%Y%m%dT%H%M%SZ)-$$"
    cargo_home="${CARGO_HOME:-$HOME/.cargo}"
    python3 scripts/collect_dependency_notices.py \
      --metadata "$evidence/native-metadata.json" \
      --build-json "$evidence/native-build.jsonl" \
      --repo-root "$repository_root" \
      --source-root "$cargo_home/registry/src" \
      --source-root "$cargo_home/git/checkouts" \
      --supplement-manifest docs/release/dependency-notices/manifest.json \
      --supplement-root docs/release/dependency-notices \
      --output "$notices"
    binary="$(python3 - "$evidence/native-build.jsonl" <<'PY'
import json, sys
from pathlib import Path
executables = set()
for line in Path(sys.argv[1]).read_text().splitlines():
    message = json.loads(line)
    if (message.get('reason') == 'compiler-artifact'
            and message.get('target', {}).get('name') == 'rustui-playground'
            and message.get('executable')):
        executables.add(message['executable'])
if len(executables) != 1:
    sys.exit('Expected exactly one Playground executable in Cargo artifact evidence')
print(executables.pop())
PY
)"
    RUSTUI_PLAYGROUND_BINARY="$binary" \
    RUSTUI_RELEASE_NOTICES_DIR="$notices" \
      apps/rustui-playground/package-app.sh
    ;;
  web)
    verify_tree_accessibility_series
    verify_web_ime_edit_plan
    python3 scripts/apply_gpui_web_semantic_adapter.py --verify
    RUSTUI_HEADSTART_ROOT="${RUSTUI_WASM_HEADSTART_ROOT:-$repository_root/target/headstart/wasm-root}" \
    CARGO_TARGET_DIR="${RUSTUI_WASM_TARGET_DIR:-$repository_root/target/headstart/x1a-wasm}" \
      ./scripts/build-playground-web.sh
    ;;
  *)
    echo 'Usage: ./scripts/verify-release.sh source|macos|web' >&2
    exit 2
    ;;
esac
