#!/bin/sh
set -eu

app_root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repository_root=$(CDPATH= cd -- "$app_root/../.." && pwd)
target_root="${CARGO_TARGET_DIR:-$repository_root/target/headstart}"
binary="${RUSTUI_PLAYGROUND_BINARY:-$target_root/debug/rustui-playground}"
bundle="${RUSTUI_PLAYGROUND_BUNDLE_PATH:-$target_root/debug/RustUI Playground.app}"
references_dir="${RUSTUI_PLAYGROUND_REFERENCES_DIR:-$app_root/references}"
replace_existing=0
for argument in "$@"; do
    case "$argument" in
        --replace) replace_existing=1 ;;
        *)
            printf '%s\n' "unknown argument: $argument" >&2
            exit 2
            ;;
    esac
done
if [ -e "$bundle" ] || [ -L "$bundle" ]; then
    if [ "$replace_existing" -ne 1 ]; then
        printf '%s\n' "Playground bundle already exists; pass --replace to replace it: $bundle" >&2
        exit 1
    fi
    if [ -L "$bundle" ]; then
        printf '%s\n' "refusing symlinked Playground bundle path: $bundle" >&2
        exit 1
    fi
    if [ ! -d "$bundle" ]; then
        printf '%s\n' "refusing to replace non-directory Playground bundle path: $bundle" >&2
        exit 1
    fi
fi
resources="$bundle/Contents/Resources"
references_bundle="$resources/references"
references_staging=""
references_previous="$resources/references.previous.$$"
references_moved=""
output_staging=""
replacement_backup=""
replacement_moved=""
release_notices_source="${RUSTUI_RELEASE_NOTICES_DIR:-}"
release_notices_required="${RUSTUI_RELEASE_NOTICES_REQUIRED:-0}"
case "$release_notices_required" in
    0|1) ;;
    *)
        printf '%s\n' 'RUSTUI_RELEASE_NOTICES_REQUIRED must be 0 or 1' >&2
        exit 1
        ;;
esac
if [ "$release_notices_required" = 1 ]; then
    if [ -z "$release_notices_source" ]; then
        printf '%s\n' 'release packaging requires RUSTUI_RELEASE_NOTICES_DIR' >&2
        exit 1
    fi
    python3 "$repository_root/scripts/package_release_notices.py" \
        --validate-only \
        --require-no-gaps \
        --collection "$release_notices_source" \
        --font-source "$repository_root/crates/rustui/assets/fonts" \
        --repo-root "$repository_root" \
        --destination "$resources/licenses/release"
fi
fresh_bundle_marker=""

cleanup() {
    status=$?
    trap - EXIT HUP INT TERM
    if [ -n "$output_staging" ]; then
        rm -f "$output_staging"
    fi
    if [ -n "$references_staging" ]; then
        rm -rf "$references_staging"
    fi
    if [ -n "$references_moved" ]; then
        if [ ! -e "$references_bundle" ]; then
            mv "$references_previous" "$references_bundle"
        else
            rm -rf "$references_previous"
        fi
    fi
    if [ -n "$fresh_bundle_marker" ] && [ ! -L "$bundle" ] && \
        [ -f "$fresh_bundle_marker" ] && [ ! -L "$fresh_bundle_marker" ]; then
        if [ "$status" -ne 0 ]; then
            rm -rf "$bundle"
        else
            rm -f "$fresh_bundle_marker"
        fi
    fi
    if [ -n "$replacement_moved" ]; then
        if [ "$status" -eq 0 ]; then
            if ! rm -rf "$replacement_backup"; then
                printf '%s\n' "could not remove replaced Playground bundle backup: $replacement_backup" >&2
                status=1
            fi
        else
            if [ -e "$bundle" ] || [ -L "$bundle" ]; then
                rm -rf "$bundle"
            fi
            if ! mv "$replacement_backup" "$bundle"; then
                printf '%s\n' "could not restore prior Playground bundle from: $replacement_backup" >&2
                status=1
            fi
        fi
    fi
    exit "$status"
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM

copy_output() {
    output_staging=$(mktemp "$2.tmp.XXXXXX")
    cp -p "$1" "$output_staging"
    if [ "${3:-}" = executable ]; then
        chmod 755 "$output_staging"
    fi
    mv -f "$output_staging" "$2"
    output_staging=""
}

if [ ! -f "$binary" ] || [ ! -x "$binary" ]; then
    printf '%s\n' "missing executable: $binary" >&2
    printf '%s\n' "build rustui-playground; set RUSTUI_PLAYGROUND_BINARY to its Cargo artifact path when using a nonstandard layout" >&2
    exit 1
fi
if [ ! -d "$references_dir" ]; then
    printf '%s\n' "missing SVG references directory: $references_dir" >&2
    exit 1
fi
for resource in assets/link-fixture.html assets/link-disabled-fixture.html \
    assets/playground-sample.png themes/macos27.json; do
    if [ ! -f "$repository_root/$resource" ]; then
        printf '%s\n' "missing Playground resource: $repository_root/$resource" >&2
        exit 1
    fi
done
if [ -e "$references_previous" ]; then
    printf '%s\n' "refusing existing Playground backup path: $references_previous" >&2
    exit 1
fi

# None of the output paths may redirect writes outside the selected bundle.
for destination in \
    "$bundle" "$bundle/Contents" "$bundle/Contents/MacOS" \
    "$bundle/Contents/MacOS/rustui-playground" \
    "$bundle/Contents/Info.plist" "$resources" \
    "$resources/assets" "$resources/assets/link-fixture.html" \
    "$resources/assets/link-disabled-fixture.html" \
    "$resources/assets/playground-sample.png" \
    "$resources/themes" "$resources/themes/macos27.json" \
    "$resources/licenses" "$resources/licenses/icons" \
    "$resources/licenses/icons/LICENSE-LUCIDE" "$resources/licenses/release" \
    "$references_bundle" "$references_previous"; do
    if [ -L "$destination" ]; then
        printf '%s\n' "refusing symlinked Playground bundle path: $destination" >&2
        exit 1
    fi
done
for destination in \
    "$bundle/Contents/MacOS/rustui-playground" "$bundle/Contents/Info.plist" \
    "$resources/assets/link-fixture.html" \
    "$resources/assets/link-disabled-fixture.html" \
    "$resources/assets/playground-sample.png" \
    "$resources/themes/macos27.json" \
    "$resources/licenses/icons/LICENSE-LUCIDE"; do
    if [ -e "$destination" ] && [ ! -f "$destination" ]; then
        printf '%s\n' "refusing non-file Playground output: $destination" >&2
        exit 1
    fi
done
if [ -z "$release_notices_source" ] && [ -e "$resources/licenses/release" ]; then
  printf '%s\n' 'refusing to reuse release notices without RUSTUI_RELEASE_NOTICES_DIR' >&2
  exit 1
fi

# Replacing is explicit and happens only after the preflight checks above.
if [ "$replace_existing" -eq 1 ] && \
    { [ -e "$bundle" ] || [ -L "$bundle" ]; }; then
    replacement_backup="$bundle.previous.$$"
    if [ -e "$replacement_backup" ] || [ -L "$replacement_backup" ]; then
        printf '%s\n' "refusing existing Playground replacement backup path: $replacement_backup" >&2
        exit 1
    fi
    if ! mv "$bundle" "$replacement_backup"; then
        printf '%s\n' "could not move existing Playground bundle for replacement: $bundle" >&2
        exit 1
    fi
    replacement_moved=1
fi
if [ -n "${RUSTUI_PLAYGROUND_BUNDLE_PATH:-}" ] && [ ! -e "$bundle" ]; then
    mkdir -p "$(dirname -- "$bundle")"
    if ! mkdir "$bundle"; then
        printf '%s\n' "cannot create fresh Playground bundle: $bundle" >&2
        exit 1
    fi
    if ! fresh_bundle_marker=$(mktemp "$bundle/.rustui-package-owner.XXXXXX"); then
        rmdir "$bundle" 2>/dev/null || :
        exit 1
    fi
fi
mkdir -p "$bundle/Contents/MacOS" "$resources"
mkdir -p "$resources/assets" "$resources/themes"
copy_output "$repository_root/assets/link-fixture.html" "$resources/assets/link-fixture.html"
copy_output "$repository_root/assets/link-disabled-fixture.html" "$resources/assets/link-disabled-fixture.html"
copy_output "$repository_root/assets/playground-sample.png" "$resources/assets/playground-sample.png"
copy_output "$repository_root/themes/macos27.json" "$resources/themes/macos27.json"
mkdir -p "$resources/licenses/icons"
copy_output "$repository_root/vendor/gpui-kit-assets-0.7.3/LICENSE-LUCIDE" \
    "$resources/licenses/icons/LICENSE-LUCIDE"
if [ -n "$release_notices_source" ]; then
    if [ "$release_notices_required" = 1 ]; then
        python3 "$repository_root/scripts/package_release_notices.py" \
            --require-no-gaps \
            --collection "$release_notices_source" \
            --font-source "$repository_root/crates/rustui/assets/fonts" \
            --repo-root "$repository_root" \
            --destination "$resources/licenses/release"
    else
        python3 "$repository_root/scripts/package_release_notices.py" \
            --collection "$release_notices_source" \
            --font-source "$repository_root/crates/rustui/assets/fonts" \
            --repo-root "$repository_root" \
            --destination "$resources/licenses/release"
    fi
fi
references_staging=$(mktemp -d "$resources/.references.next.XXXXXX")
cp -R "$references_dir/." "$references_staging/"
if [ -e "$references_bundle" ] || [ -L "$references_bundle" ]; then
    mv "$references_bundle" "$references_previous"
    references_moved=1
fi
mv "$references_staging" "$references_bundle"
references_staging=""
rm -rf "$references_previous"
references_moved=""
copy_output "$app_root/macos/Info.plist" "$bundle/Contents/Info.plist"
copy_output "$binary" "$bundle/Contents/MacOS/rustui-playground" executable
printf '%s\n' "$bundle"
