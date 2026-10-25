#!/bin/sh
set -eu

app_root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repository_root=$(CDPATH= cd -- "$app_root/../.." && pwd)
target_root="${CARGO_TARGET_DIR:-$repository_root/target/headstart}"
binary="${RUSTUI_PLAYGROUND_BINARY:-$target_root/debug/rustui-playground}"
bundle="$target_root/debug/RustUI Playground.app"
references_dir="${RUSTUI_PLAYGROUND_REFERENCES_DIR:-$app_root/references}"
resources="$bundle/Contents/Resources"
references_bundle="$resources/references"
references_staging=""
references_previous="$resources/references.previous.$$"
release_notices_source="${RUSTUI_RELEASE_NOTICES_DIR:-}"

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

if [ ! -f "$binary" ] || [ ! -x "$binary" ]; then
    printf '%s\n' "missing executable: $binary" >&2
    printf '%s\n' "build rustui-playground; set RUSTUI_PLAYGROUND_BINARY to its Cargo artifact path when using a nonstandard layout" >&2
    exit 1
fi
if [ ! -d "$references_dir" ]; then
    printf '%s\n' "missing SVG references directory: $references_dir" >&2
    exit 1
fi

mkdir -p "$bundle/Contents/MacOS" "$resources"
mkdir -p "$resources/licenses/icons"
cp "$repository_root/vendor/gpui-kit-assets-0.7.3/LICENSE-LUCIDE" \
    "$resources/licenses/icons/LICENSE-LUCIDE"
if [ -n "$release_notices_source" ]; then
    python3 "$repository_root/scripts/package_release_notices.py" \
        --collection "$release_notices_source" \
        --font-source "$repository_root/crates/rustui/assets/fonts" \
        --repo-root "$repository_root" \
        --destination "$resources/licenses/release"
fi
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
