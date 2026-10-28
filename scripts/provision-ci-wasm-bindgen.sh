#!/bin/bash
set -euo pipefail

# The trusted CI runner is macOS ARM64. Keep the tool in the existing ignored
# Headstart tools directory, without replacing a tool already owned by the user.
root="$(cd "$(dirname "$0")/.." && pwd)"
version=0.2.129
archive="wasm-bindgen-${version}-aarch64-apple-darwin.tar.gz"
digest=81d4a23d56b3c3eb8187658329116d50e0b228a93b343825fb71f70179051cd1
tool="$root/target/headstart/tools/bin/wasm-bindgen"

if [ "$(uname -s)" != Darwin ] || [ "$(uname -m)" != arm64 ]; then
  echo 'CI wasm-bindgen provisioning requires macOS ARM64.' >&2
  exit 1
fi
if [ -e "$tool" ]; then
  if [ -x "$tool" ] && [ "$("$tool" --version)" = "wasm-bindgen $version" ]; then
    exit 0
  fi
  echo "Existing wasm-bindgen is incompatible; preserving $tool" >&2
  exit 1
fi

mkdir -p "$(dirname "$tool")"
tmp="$(mktemp -d "$root/target/headstart/tools/.bindgen.XXXXXXXX")"
trap 'rm -rf "$tmp"' EXIT
curl -fsSL --retry 2 "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/$version/$archive" -o "$tmp/$archive"
actual="$(shasum -a 256 "$tmp/$archive" | cut -d ' ' -f 1)"
if [ "$actual" != "$digest" ]; then
  echo 'wasm-bindgen archive SHA-256 mismatch.' >&2
  exit 1
fi
tar -xOzf "$tmp/$archive" "wasm-bindgen-${version}-aarch64-apple-darwin/wasm-bindgen" > "$tmp/wasm-bindgen"
chmod 755 "$tmp/wasm-bindgen"
if [ "$("$tmp/wasm-bindgen" --version)" != "wasm-bindgen $version" ]; then
  echo 'Downloaded wasm-bindgen reports an unexpected version.' >&2
  exit 1
fi
# A hard link makes installation fail rather than overwrite a concurrent owner.
ln "$tmp/wasm-bindgen" "$tool"
echo "Provisioned wasm-bindgen $version from the SHA-256 verified upstream release."
