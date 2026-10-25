#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
temporary_root=$(mktemp -d "${TMPDIR:-/tmp}/rustui-playground-package-test.XXXXXX")
trap 'rm -rf "$temporary_root"' EXIT HUP INT TERM

target_root="$temporary_root/target"
references_source="$temporary_root/reference assets"
binary="$target_root/debug/rustui-playground"
bundle="$target_root/debug/RustUI Playground.app"

mkdir -p "$target_root/debug" "$references_source/Nested Folder"
printf '%s\n' '#!/bin/sh' 'exit 0' > "$binary"
chmod 755 "$binary"
printf '%s\n' '<svg />' > "$references_source/Nested Folder/Example.svg"

CARGO_TARGET_DIR="$target_root" \
RUSTUI_PLAYGROUND_REFERENCES_DIR="$references_source" \
    "$repository_root/apps/rustui-playground/package-app.sh" >/dev/null

test -x "$bundle/Contents/MacOS/rustui-playground"
test -f "$bundle/Contents/Info.plist"
cmp "$repository_root/vendor/gpui-kit-assets-0.7.3/LICENSE-LUCIDE" \
    "$bundle/Contents/Resources/licenses/icons/LICENSE-LUCIDE"
headstart_binary="$target_root/headstart/build/package hash/out/rustui-playground"
mkdir -p "$(dirname -- "$headstart_binary")"
printf '%s\n' '#!/bin/sh' '# headstart artifact' 'exit 0' > "$headstart_binary"
chmod 755 "$headstart_binary"
CARGO_TARGET_DIR="$target_root" \
RUSTUI_PLAYGROUND_BINARY="$headstart_binary" \
RUSTUI_PLAYGROUND_REFERENCES_DIR="$references_source" \
    "$repository_root/apps/rustui-playground/package-app.sh" >/dev/null
cmp "$headstart_binary" "$bundle/Contents/MacOS/rustui-playground"
if CARGO_TARGET_DIR="$target_root" \
    RUSTUI_PLAYGROUND_BINARY="$(dirname -- "$headstart_binary")" \
    RUSTUI_PLAYGROUND_REFERENCES_DIR="$references_source" \
        "$repository_root/apps/rustui-playground/package-app.sh" >/dev/null 2>&1; then
    printf '%s\n' "package accepted a directory as an executable" >&2
    exit 1
fi
cmp "$headstart_binary" "$bundle/Contents/MacOS/rustui-playground"
cmp \
    "$references_source/Nested Folder/Example.svg" \
    "$bundle/Contents/Resources/references/Nested Folder/Example.svg"

rm -rf "$references_source/Nested Folder"
mkdir -p "$references_source/Updated Folder"
printf '%s\n' '<svg id="updated" />' > "$references_source/Updated Folder/Updated.svg"

CARGO_TARGET_DIR="$target_root" \
RUSTUI_PLAYGROUND_REFERENCES_DIR="$references_source" \
    "$repository_root/apps/rustui-playground/package-app.sh" >/dev/null

test -x "$bundle/Contents/MacOS/rustui-playground"
test -f "$bundle/Contents/Info.plist"
test ! -e "$bundle/Contents/Resources/references/Nested Folder/Example.svg"
cmp \
    "$references_source/Updated Folder/Updated.svg" \
    "$bundle/Contents/Resources/references/Updated Folder/Updated.svg"

mv_shim="$temporary_root/mv-shim"
mkdir -p "$mv_shim"
cat > "$mv_shim/mv" <<'EOF'
#!/bin/sh
case "$1" in
    */.references.next.*) exit 73 ;;
esac
exec /bin/mv "$@"
EOF
chmod 755 "$mv_shim/mv"
mkdir -p "$references_source/Failed Update"
printf '%s\n' '<svg id="must-not-publish" />' > "$references_source/Failed Update/Failed.svg"

if PATH="$mv_shim:$PATH" \
    CARGO_TARGET_DIR="$target_root" \
    RUSTUI_PLAYGROUND_REFERENCES_DIR="$references_source" \
        "$repository_root/apps/rustui-playground/package-app.sh" >/dev/null 2>&1; then
    printf '%s\n' "package unexpectedly succeeded when staged references could not be promoted." >&2
    exit 1
fi

cmp \
    "$references_source/Updated Folder/Updated.svg" \
    "$bundle/Contents/Resources/references/Updated Folder/Updated.svg"
test ! -e "$bundle/Contents/Resources/references/Failed Update/Failed.svg"
test -z "$(find "$bundle/Contents/Resources" -maxdepth 1 -name '.references.next.*' -print -quit)"

printf '%s\n' "Playground package replaces stale SVG references and preserves them if promotion fails."
