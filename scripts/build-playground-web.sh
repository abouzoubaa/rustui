#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -gt 1 ] || { [ "$#" -eq 1 ] && [ "$1" != "--check-bindgen" ]; }; then
  echo "Usage: $0 [--check-bindgen]" >&2
  exit 2
fi

REPO_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/target/headstart/x1a-wasm}"
export RUSTUI_HEADSTART_ROOT="${RUSTUI_HEADSTART_ROOT:-$REPO_ROOT/target/headstart/wasm-root}"
export RUSTUI_REMAP_BUILD_PATHS="${RUSTUI_REMAP_BUILD_PATHS:-1}"
# The grammar C sources use the headers shipped by tree-sitter-language.
# Apple Clang does not include a WebAssembly backend; prefer an installed LLVM.
if [ -z "${CC_wasm32_unknown_unknown:-}" ] && command -v brew >/dev/null 2>&1; then
  LLVM_CLANG="$(brew --prefix llvm@22 2>/dev/null || true)/bin/clang"
  if [ -x "$LLVM_CLANG" ]; then
    export CC_wasm32_unknown_unknown="$LLVM_CLANG"
  fi
fi
export CFLAGS_wasm32_unknown_unknown="${CFLAGS_wasm32_unknown_unknown:-} -I$REPO_ROOT/vendor/tree-sitter-language-0.1.9/wasm/include"

if [ -x "$REPO_ROOT/target/headstart/tools/bin/wasm-bindgen" ]; then
  export PATH="$REPO_ROOT/target/headstart/tools/bin:$PATH"
fi

if ! command -v rust-lld >/dev/null 2>&1 && command -v rustup >/dev/null 2>&1; then
  HOST_RUSTC="$(rustup which rustc --toolchain 1.97.1)"
  HOST_TRIPLE="$("$HOST_RUSTC" -vV | sed -n 's/^host: //p')"
  HOST_LLD_DIR="$(dirname -- "$HOST_RUSTC")/../lib/rustlib/$HOST_TRIPLE/bin"
  if [ -x "$HOST_LLD_DIR/rust-lld" ]; then
    export PATH="$HOST_LLD_DIR:$PATH"
  fi
fi

WASM_BINDGEN_VERSION="0.2.129"
if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "wasm-bindgen CLI $WASM_BINDGEN_VERSION is required; install the version matching Cargo.lock." >&2
  exit 127
fi
if [ "$(wasm-bindgen --version)" != "wasm-bindgen $WASM_BINDGEN_VERSION" ]; then
  echo "wasm-bindgen CLI $WASM_BINDGEN_VERSION is required; install the version matching Cargo.lock." >&2
  exit 1
fi
if [ "${1:-}" = "--check-bindgen" ]; then
  echo "wasm-bindgen $WASM_BINDGEN_VERSION available"
  exit 0
fi

HEADSTART_HOST="$(rustc -vV | sed -n 's/^host: //p')"
HEADSTART_RUSTC="$RUSTUI_HEADSTART_ROOT/rustc/build/$HEADSTART_HOST/stage1/bin/rustc"
if [ -x "$HEADSTART_RUSTC" ]; then
  WASM_LIBDIR="$("$HEADSTART_RUSTC" --print target-libdir --target wasm32-unknown-unknown)"
  if ! compgen -G "$WASM_LIBDIR/libstd-*.rlib" >/dev/null; then
    echo "Wasm standard library missing from $WASM_LIBDIR." >&2
    echo "Restore the matching wasm32-unknown-unknown standard library in this Headstart toolchain before building." >&2
    exit 1
  fi
fi

# An explicit, simple name permits parallel source-bound validation bundles while
# keeping all generated output under the persistent Headstart directory.
WEB_OUTPUT_NAME="${RUSTUI_WEB_OUTPUT_NAME:-playground-web}"
if [[ ! "$WEB_OUTPUT_NAME" =~ ^playground-web(-[A-Za-z0-9_-]+)?$ ]]; then
  echo "Invalid RUSTUI_WEB_OUTPUT_NAME: use playground-web or playground-web-<name>." >&2
  exit 2
fi
OUT_DIR="$REPO_ROOT/target/headstart/$WEB_OUTPUT_NAME"
STAGING_DIR=""
PREVIOUS_DIR=""
SOURCE_SNAPSHOT_DIR="$(mktemp -d "${TMPDIR:-/tmp}/rustui-playground-web-source.XXXXXX")"
SOURCE_SNAPSHOT_FILE="$SOURCE_SNAPSHOT_DIR/source.json"
cleanup() {
  if [[ -n "$STAGING_DIR" ]]; then
    rm -rf -- "$STAGING_DIR"
  fi
  if [[ -n "$PREVIOUS_DIR" && ( -e "$PREVIOUS_DIR" || -L "$PREVIOUS_DIR" ) ]]; then
    if [[ ! -e "$OUT_DIR" && ! -L "$OUT_DIR" ]]; then
      mv -- "$PREVIOUS_DIR" "$OUT_DIR"
    else
      rm -rf -- "$PREVIOUS_DIR"
    fi
  fi
  if [[ -n "$SOURCE_SNAPSHOT_DIR" ]]; then
    rm -rf -- "$SOURCE_SNAPSHOT_DIR"
  fi
}
trap cleanup EXIT
python3 "$REPO_ROOT/scripts/manifest_playground_web.py" \
  --root "$REPO_ROOT" \
  --output "$SOURCE_SNAPSHOT_FILE"

EVIDENCE_DIR="$REPO_ROOT/target/headstart/evidence/release-checks"
mkdir -p "$EVIDENCE_DIR"
"$REPO_ROOT/scripts/cargo-headstart" metadata --format-version 1 --locked --offline \
  > "$EVIDENCE_DIR/web-metadata.json"
"$REPO_ROOT/scripts/cargo-headstart" build -p rustui-playground \
  --target wasm32-unknown-unknown --locked --offline --release --message-format=json \
  > "$EVIDENCE_DIR/web-build.jsonl"
NOTICES_BASE="$REPO_ROOT/target/headstart/evidence/web-dependency-notices-checked-$(date -u +%Y%m%dT%H%M%SZ)-$$"
NOTICES_DIR="$NOTICES_BASE"
notices_suffix=0
while [[ -e "$NOTICES_DIR" || -L "$NOTICES_DIR" ]]; do
  notices_suffix=$((notices_suffix + 1))
  NOTICES_DIR="$NOTICES_BASE-$notices_suffix"
done
CARGO_HOME_FOR_NOTICES="${CARGO_HOME:-$HOME/.cargo}"
python3 "$REPO_ROOT/scripts/collect_dependency_notices.py" \
  --metadata "$EVIDENCE_DIR/web-metadata.json" \
  --build-json "$EVIDENCE_DIR/web-build.jsonl" \
  --repo-root "$REPO_ROOT" \
  --source-root "$CARGO_HOME_FOR_NOTICES/registry/src" \
  --source-root "$CARGO_HOME_FOR_NOTICES/git/checkouts" \
  --supplement-manifest "$REPO_ROOT/docs/release/dependency-notices/manifest.json" \
  --supplement-root "$REPO_ROOT/docs/release/dependency-notices" \
  --output "$NOTICES_DIR"

mkdir -p "$(dirname -- "$OUT_DIR")"
STAGING_DIR="$(mktemp -d "$(dirname -- "$OUT_DIR")/.playground-web.XXXXXX")"
wasm-bindgen \
  --target web \
  --out-dir "$STAGING_DIR" \
  --out-name rustui_playground \
  "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/rustui-playground.wasm"
for artifact in rustui_playground.js rustui_playground_bg.wasm; do
  if [[ ! -s "$STAGING_DIR/$artifact" ]]; then
    echo "wasm-bindgen did not produce a non-empty $artifact; preserving the existing Web bundle." >&2
    exit 1
  fi
done
python3 - "$STAGING_DIR" "$REPO_ROOT" <<'PY'
from pathlib import Path
import sys

stage, root = Path(sys.argv[1]), sys.argv[2].encode()
for name in ("rustui_playground.js", "rustui_playground_bg.wasm"):
    if root in (stage / name).read_bytes():
        raise SystemExit(f"{name} embeds the checkout path; preserving existing Web bundle")
PY
cp "$REPO_ROOT/apps/rustui-playground/web/index.html" "$STAGING_DIR/index.html"
for asset in reference-host.html reference-host.js reference-client.js; do
    cp "$REPO_ROOT/apps/rustui-playground/web/$asset" "$STAGING_DIR/$asset"
    if [[ ! -s "$STAGING_DIR/$asset" ]]; then
        echo "Web reference host asset is empty: $asset; preserving existing Web bundle." >&2
        exit 1
    fi
done
if [[ ! -s "$STAGING_DIR/index.html" ]]; then
  echo "Web browser shell is empty; preserving the existing Web bundle." >&2
  exit 1
fi
MODULE_INIT="$(awk '
  function is_module_script(line, lower, single_quote, start, end, tag) {
    if (pending_script_tag != "") {
      line = pending_script_tag " " line
      pending_script_tag = ""
    }
    while (1) {
      start = match(tolower(line), /<script([[:space:]>]|$)/)
      if (!start) return 0
      line = substr(line, start)
      end = index(line, ">")
      if (!end) {
        pending_script_tag = line
        return 0
      }
      tag = substr(line, 1, end)
      lower = tolower(tag)
      single_quote = sprintf("%c", 39)
      if (lower ~ ("<script[[:space:]]([^>]*[[:space:]])?type[[:space:]]*=[[:space:]]*(\"module\"|" single_quote "module" single_quote "|module)([[:space:]/>])")) return 1
      line = substr(line, end + 1)
    }
  }
  function update_template_state(line, i, char, escaped) {
    escaped = 0
    for (i = 1; i <= length(line); i++) {
      char = substr(line, i, 1)
      if (char == "\\") {
        escaped = !escaped
        continue
      }
      if (char == "`" && !escaped) in_template = !in_template
      escaped = 0
    }
  }
  function strip_html_comments(line, result, start, end) {
    result = ""
    while (1) {
      if (in_html_comment) {
        end = index(line, "-->")
        if (!end) return result
        line = substr(line, end + 3)
        in_html_comment = 0
      }
      start = index(line, "<!--")
      if (!start) return result line
      result = result substr(line, 1, start - 1)
      line = substr(line, start + 4)
      in_html_comment = 1
    }
  }
  !in_module_script { $0 = strip_html_comments($0) }
  !in_module_script && is_module_script($0) { in_module_script = 1 }
  in_module_script { update_template_state($0) }
  /\/\*/ { in_block_comment = 1 }
  in_module_script && !in_block_comment && !in_template && /^[[:space:]]*const[[:space:]]+\{[[:space:]]*default:[[:space:]]*[[:alpha:]_$][[:alnum:]_$]*[[:space:]]*\}[[:space:]]*=[[:space:]]*await[[:space:]]+import\("\.\/rustui_playground\.js"\);[[:space:]]*$/ {
    imported = $0
    sub(/^[[:space:]]*const[[:space:]]+\{[[:space:]]*default:[[:space:]]*/, "", imported)
    sub(/[[:space:]]*\}[[:space:]]*=.*/, "", imported)
    print imported
    found = 1
  }
  /\*\// { in_block_comment = 0 }
  /<\/script[[:space:]]*>/ { in_module_script = 0 }
  END { exit !found }
' "$STAGING_DIR/index.html")" || {
  echo "Web browser shell does not load rustui_playground.js; preserving the existing Web bundle." >&2
  exit 1
}
if ! awk -v module_init="$MODULE_INIT" '
  function is_module_script(line, lower, single_quote, start, end, tag) {
    if (pending_script_tag != "") {
      line = pending_script_tag " " line
      pending_script_tag = ""
    }
    while (1) {
      start = match(tolower(line), /<script([[:space:]>]|$)/)
      if (!start) return 0
      line = substr(line, start)
      end = index(line, ">")
      if (!end) {
        pending_script_tag = line
        return 0
      }
      tag = substr(line, 1, end)
      lower = tolower(tag)
      single_quote = sprintf("%c", 39)
      if (lower ~ ("<script[[:space:]]([^>]*[[:space:]])?type[[:space:]]*=[[:space:]]*(\"module\"|" single_quote "module" single_quote "|module)([[:space:]/>])")) return 1
      line = substr(line, end + 1)
    }
  }
  function update_template_state(line, i, char, escaped) {
    escaped = 0
    for (i = 1; i <= length(line); i++) {
      char = substr(line, i, 1)
      if (char == "\\") {
        escaped = !escaped
        continue
      }
      if (char == "`" && !escaped) in_template = !in_template
      escaped = 0
    }
  }
  function strip_html_comments(line, result, start, end) {
    result = ""
    while (1) {
      if (in_html_comment) {
        end = index(line, "-->")
        if (!end) return result
        line = substr(line, end + 3)
        in_html_comment = 0
      }
      start = index(line, "<!--")
      if (!start) return result line
      result = result substr(line, 1, start - 1)
      line = substr(line, start + 4)
      in_html_comment = 1
    }
  }
  !in_module_script { $0 = strip_html_comments($0) }
  !in_module_script && is_module_script($0) { in_module_script = 1; imported = 0 }
  in_module_script { update_template_state($0) }
  /\/\*/ { in_block_comment = 1 }
  in_module_script && !in_block_comment && !in_template && /^[[:space:]]*const[[:space:]]+\{[[:space:]]*default:[[:space:]]*[[:alpha:]_$][[:alnum:]_$]*[[:space:]]*\}[[:space:]]*=[[:space:]]*await[[:space:]]+import\("\.\/rustui_playground\.js"\);[[:space:]]*$/ {
    binding = $0
    sub(/^[[:space:]]*const[[:space:]]+\{[[:space:]]*default:[[:space:]]*/, "", binding)
    sub(/[[:space:]]*\}[[:space:]]*=.*/, "", binding)
    if (binding == module_init) imported = 1
  }
  in_module_script && imported && !in_block_comment && !in_template && /^[[:space:]]*await[[:space:]]+[[:alpha:]_$][[:alnum:]_$]*\(\);[[:space:]]*$/ {
    called = $0
    sub(/^[[:space:]]*await[[:space:]]+/, "", called)
    sub(/\(\);[[:space:]]*$/, "", called)
    if (called == module_init) found = 1
  }
  /\*\// { in_block_comment = 0 }
  /<\/script[[:space:]]*>/ { in_module_script = 0 }
  END { exit !found }
' "$STAGING_DIR/index.html"; then
  echo "Web browser shell does not initialize rustui_playground.js; preserving the existing Web bundle." >&2
  exit 1
fi
mkdir "$STAGING_DIR/references"
cp -R "$REPO_ROOT/apps/rustui-playground/references/." "$STAGING_DIR/references/"
mkdir "$STAGING_DIR/assets"
for asset in playground-sample.png link-fixture.html link-disabled-fixture.html; do
  cp "$REPO_ROOT/assets/$asset" "$STAGING_DIR/assets/$asset"
done
cp "$REPO_ROOT/themes/macos27.json" "$STAGING_DIR/assets/macos27.json"
mkdir -p "$STAGING_DIR/licenses/fonts"
cp "$REPO_ROOT"/crates/rustui/assets/fonts/*-LICENSE "$STAGING_DIR/licenses/fonts/"
mkdir -p "$STAGING_DIR/licenses/icons"
cp "$REPO_ROOT/vendor/gpui-kit-assets-0.7.3/LICENSE-LUCIDE" \
  "$STAGING_DIR/licenses/icons/LICENSE-LUCIDE"
python3 "$REPO_ROOT/scripts/package_release_notices.py" \
  --collection "$NOTICES_DIR" \
  --font-source "$REPO_ROOT/crates/rustui/assets/fonts" \
  --repo-root "$REPO_ROOT" \
  --destination "$STAGING_DIR/licenses/release"
python3 "$REPO_ROOT/scripts/manifest_playground_web.py" \
  --root "$REPO_ROOT" \
  --bundle-dir "$STAGING_DIR" \
  --source-baseline "$SOURCE_SNAPSHOT_FILE" \
  --output "$STAGING_DIR/bundle-manifest.json"
if [[ -e "$OUT_DIR" || -L "$OUT_DIR" ]]; then
  PREVIOUS_DIR="$(mktemp -d "$(dirname -- "$OUT_DIR")/.playground-web-previous.XXXXXX")"
  rmdir -- "$PREVIOUS_DIR"
  mv -- "$OUT_DIR" "$PREVIOUS_DIR"
fi
mv -- "$STAGING_DIR" "$OUT_DIR"
STAGING_DIR=""
if [[ -n "$PREVIOUS_DIR" ]]; then
  rm -rf -- "$PREVIOUS_DIR"
  PREVIOUS_DIR=""
fi
printf 'Web bundle: %s\nServe it with: python3 -m http.server 8000 --directory %s\n' \
  "$OUT_DIR" "$OUT_DIR"
