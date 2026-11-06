#!/bin/sh
set -eu

app_root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repository_root=$(CDPATH= cd -- "$app_root/../.." && pwd)
target_root="${CARGO_TARGET_DIR:-$repository_root/target/headstart}"
binary="$target_root/debug/rustui-playground"
bundle="$target_root/debug/RustUI Playground.app"
references_dir="${RUSTUI_PLAYGROUND_REFERENCES_DIR:-$app_root/references}"
resources="$bundle/Contents/Resources"
references_bundle="$resources/references"
references_staging=""
references_previous="$resources/references.previous.$$"

cleanup() {
    status=$?
    trap - EXIT HUP INT TERM
    if [ -n "$references_staging" ]; then
        rm -rf "$references_staging"
    fi
    if [ -e "$references_previous" ]; then
        if [ ! -e "$references_bundle" ]; then
            mv "$references_previous" "$references_bundle"
        else
            rm -rf "$references_previous"
        fi
    fi
    exit "$status"
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM

if [ ! -x "$binary" ]; then
    printf '%s\n' "missing executable: $binary" >&2
    printf '%s\n' "run ./scripts/cargo-headstart build -p rustui-playground" >&2
    exit 1
fi
if [ ! -d "$references_dir" ]; then
    printf '%s\n' "missing SVG references directory: $references_dir" >&2
    exit 1
fi

mkdir -p "$bundle/Contents/MacOS" "$resources"
references_staging=$(mktemp -d "$resources/.references.next.XXXXXX")
cp -R "$references_dir/." "$references_staging/"
if [ -e "$references_bundle" ] || [ -L "$references_bundle" ]; then
    mv "$references_bundle" "$references_previous"
fi
mv "$references_staging" "$references_bundle"
references_staging=""
rm -rf "$references_previous"
cp "$app_root/macos/Info.plist" "$bundle/Contents/Info.plist"
cp "$binary" "$bundle/Contents/MacOS/rustui-playground.next"
chmod 755 "$bundle/Contents/MacOS/rustui-playground.next"
mv -f "$bundle/Contents/MacOS/rustui-playground.next" "$bundle/Contents/MacOS/rustui-playground"
printf '%s\n' "$bundle"
