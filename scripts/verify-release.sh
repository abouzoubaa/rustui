#!/usr/bin/env bash
# Source/build checks only. Native/browser/AX/performance acceptance is separate.
set -euo pipefail
repository_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repository_root"

case "${1:-}" in
  source)
    python3 -m unittest discover -s scripts/tests -p 'test_*.py'
    sh scripts/tests/test_package_playground_app.sh
    sh scripts/tests/test_build_playground_web.sh
    python3 scripts/validate_locales.py
    python3 scripts/validate_themes.py --plans-only
    python3 scripts/verify_theme_adapter_coverage.py
    ;;
  macos)
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
    ./scripts/cargo-headstart build -p rustui-playground --release --locked --offline \
      --message-format=json > "$evidence/native-build.jsonl"
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
    RUSTUI_PLAYGROUND_BINARY="$binary" apps/rustui-playground/package-app.sh
    ;;
  web)
    RUSTUI_HEADSTART_ROOT="${RUSTUI_WASM_HEADSTART_ROOT:-$repository_root/target/headstart/wasm-root}" \
    CARGO_TARGET_DIR="${RUSTUI_WASM_TARGET_DIR:-$repository_root/target/headstart/x1a-wasm}" \
      ./scripts/build-playground-web.sh
    ;;
  *)
    echo 'Usage: ./scripts/verify-release.sh source|macos|web' >&2
    exit 2
    ;;
esac
