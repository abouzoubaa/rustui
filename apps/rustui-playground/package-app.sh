#!/bin/sh
set -eu

app_root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repository_root=$(CDPATH= cd -- "$app_root/../.." && pwd)
target_root="${CARGO_TARGET_DIR:-$repository_root/target/headstart}"
binary="$target_root/debug/rustui-playground"
bundle="$target_root/debug/RustUI Playground.app"

if [ ! -x "$binary" ]; then
    printf '%s\n' "missing executable: $binary" >&2
    printf '%s\n' "run ./scripts/cargo-headstart build -p rustui-playground" >&2
    exit 1
fi

mkdir -p "$bundle/Contents/MacOS"
cp "$app_root/macos/Info.plist" "$bundle/Contents/Info.plist"
cp "$binary" "$bundle/Contents/MacOS/rustui-playground.next"
chmod 755 "$bundle/Contents/MacOS/rustui-playground.next"
mv -f "$bundle/Contents/MacOS/rustui-playground.next" "$bundle/Contents/MacOS/rustui-playground"
printf '%s\n' "$bundle"
