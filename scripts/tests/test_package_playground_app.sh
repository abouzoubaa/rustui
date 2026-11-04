#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
temporary_root=$(mktemp -d "${TMPDIR:-/tmp}/rustui-playground-package-test.XXXXXX")
trap 'rm -rf "$temporary_root"' EXIT HUP INT TERM

target_root="$temporary_root/target"
references_source="$temporary_root/reference assets"
notice_source="$temporary_root/notice collection"
binary="$target_root/debug/rustui-playground"
bundle="$target_root/debug/RustUI Playground.app"

mkdir -p "$target_root/debug" "$references_source/Nested Folder" "$notice_source/notices"
printf '%s\n' '#!/bin/sh' 'exit 0' > "$binary"
chmod 755 "$binary"
printf '%s\n' '<svg />' > "$references_source/Nested Folder/Example.svg"
printf '%s\n' 'verbatim dependency notice' > "$notice_source/notices/LICENSE"
python3 - "$notice_source" <<'PY'
import hashlib, json, sys
from pathlib import Path
root = Path(sys.argv[1])
notice = root / "notices/LICENSE"
report = {"schema_version": 1, "status": "gaps",
          "complete_license_or_compatibility_claim": False,
          "inputs": {"metadata_sha256": "1" * 64, "build_json_sha256": "2" * 64},
          "packages": [{"name": "fixture", "version": "1.0.0", "source": None,
                        "package_id": "fixture-id", "manifest_path": "fixture/Cargo.toml",
                        "manifest_sha256": "3" * 64,
                        "notices": [{"path": "notices/LICENSE",
                                     "sha256": hashlib.sha256(notice.read_bytes()).hexdigest()}]}],
          "unresolved_gaps": [{"name": "gap", "version": "1", "package_id": "gap-id",
                               "reason": "no captured notice files"}]}
(root / "report.json").write_text(json.dumps(report), encoding="utf-8")
(root / "REPORT.txt").write_text("fixture report\n", encoding="utf-8")
PY

CARGO_TARGET_DIR="$target_root" \
RUSTUI_RELEASE_NOTICES_DIR="$notice_source" \
RUSTUI_PLAYGROUND_REFERENCES_DIR="$references_source" \
    "$repository_root/apps/rustui-playground/package-app.sh" >/dev/null

test -x "$bundle/Contents/MacOS/rustui-playground"
test -f "$bundle/Contents/Info.plist"
for resource in assets/link-fixture.html assets/link-disabled-fixture.html \
    assets/playground-sample.png themes/macos27.json; do
    cmp "$repository_root/$resource" "$bundle/Contents/Resources/$resource"
done
cmp "$repository_root/vendor/gpui-kit-assets-0.7.3/LICENSE-LUCIDE" \
    "$bundle/Contents/Resources/licenses/icons/LICENSE-LUCIDE"
cmp "$notice_source/notices/LICENSE" \
    "$bundle/Contents/Resources/licenses/release/dependency-notices/notices/LICENSE"
for font_license in FiraMono-LICENSE Inter-LICENSE NotoSans-LICENSE; do
    cmp "$repository_root/crates/rustui/assets/fonts/$font_license" \
        "$bundle/Contents/Resources/licenses/release/fonts/$font_license"
done
headstart_binary="$target_root/headstart/build/package hash/out/rustui-playground"
mkdir -p "$(dirname -- "$headstart_binary")"
printf '%s\n' '#!/bin/sh' '# headstart artifact' 'exit 0' > "$headstart_binary"
chmod 755 "$headstart_binary"
CARGO_TARGET_DIR="$target_root" \
RUSTUI_PLAYGROUND_BINARY="$headstart_binary" \
RUSTUI_RELEASE_NOTICES_DIR="$notice_source" \
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

# A validation bundle can reuse Cargo artifacts while leaving the default bundle intact.
fresh_bundle="$temporary_root/isolated output/RustUI Playground.app"
printf '%s\n' 'existing bundle sentinel' > "$bundle/sentinel"
CARGO_TARGET_DIR="$target_root" \
RUSTUI_PLAYGROUND_BINARY="$headstart_binary" \
RUSTUI_PLAYGROUND_BUNDLE_PATH="$fresh_bundle" \
RUSTUI_PLAYGROUND_REFERENCES_DIR="$references_source" \
    "$repository_root/apps/rustui-playground/package-app.sh" >/dev/null
cmp "$headstart_binary" "$fresh_bundle/Contents/MacOS/rustui-playground"
test -z "$(find "$fresh_bundle" -name '.rustui-package-owner.*' -print -quit)"
test "$(cat "$bundle/sentinel")" = 'existing bundle sentinel'
if CARGO_TARGET_DIR="$target_root" \
    RUSTUI_PLAYGROUND_BINARY="$headstart_binary" \
    RUSTUI_PLAYGROUND_BUNDLE_PATH="$fresh_bundle" \
    RUSTUI_PLAYGROUND_REFERENCES_DIR="$references_source" \
    "$repository_root/apps/rustui-playground/package-app.sh" >/dev/null 2>&1; then
    printf '%s\n' 'package unexpectedly reused an explicit existing bundle.' >&2
    exit 1
fi
cmp "$headstart_binary" "$fresh_bundle/Contents/MacOS/rustui-playground"
test "$(cat "$bundle/sentinel")" = 'existing bundle sentinel'
failed_bundle="$temporary_root/failed output/RustUI Playground.app"
if CARGO_TARGET_DIR="$target_root" \
    RUSTUI_PLAYGROUND_BINARY="$headstart_binary" \
    RUSTUI_PLAYGROUND_BUNDLE_PATH="$failed_bundle" \
    RUSTUI_PLAYGROUND_REFERENCES_DIR="$references_source" \
    RUSTUI_RELEASE_NOTICES_DIR="$temporary_root/missing-notices" \
        "$repository_root/apps/rustui-playground/package-app.sh" >/dev/null 2>&1; then
    printf '%s\n' 'package unexpectedly succeeded with missing release notices.' >&2
    exit 1
fi
if [ -e "$failed_bundle" ]; then
    printf '%s\n' 'failed fresh bundle was left behind.' >&2
    exit 1
fi
CARGO_TARGET_DIR="$target_root" \
RUSTUI_PLAYGROUND_BINARY="$headstart_binary" \
RUSTUI_PLAYGROUND_BUNDLE_PATH="$failed_bundle" \
RUSTUI_PLAYGROUND_REFERENCES_DIR="$references_source" \
    "$repository_root/apps/rustui-playground/package-app.sh" >/dev/null
cmp "$headstart_binary" "$failed_bundle/Contents/MacOS/rustui-playground"
test "$(cat "$bundle/sentinel")" = 'existing bundle sentinel'
unsafe_root="$temporary_root/unsafe-target"
protected="$temporary_root/protected-app"
mkdir -p "$unsafe_root/debug" "$protected/Contents/MacOS" "$protected/Contents/Resources"
printf '%s\n' 'protected executable' > "$protected/Contents/MacOS/rustui-playground"
ln -s "$protected" "$unsafe_root/debug/RustUI Playground.app"
if CARGO_TARGET_DIR="$unsafe_root" \
    RUSTUI_PLAYGROUND_BINARY="$headstart_binary" \
    RUSTUI_PLAYGROUND_REFERENCES_DIR="$references_source" \
    "$repository_root/apps/rustui-playground/package-app.sh" >/dev/null 2>&1; then
    printf '%s\n' 'package accepted a symlinked default bundle.' >&2
    exit 1
fi
test "$(cat "$protected/Contents/MacOS/rustui-playground")" = 'protected executable'

unsafe_contents="$temporary_root/unsafe-contents"
unsafe_bundle="$unsafe_contents/debug/RustUI Playground.app"
mkdir -p "$unsafe_bundle/Contents/MacOS"
ln -s "$protected/Contents/Resources" "$unsafe_bundle/Contents/Resources"
if CARGO_TARGET_DIR="$unsafe_contents" \
    RUSTUI_PLAYGROUND_BINARY="$headstart_binary" \
    RUSTUI_PLAYGROUND_REFERENCES_DIR="$references_source" \
    "$repository_root/apps/rustui-playground/package-app.sh" >/dev/null 2>&1; then
    printf '%s\n' 'package accepted a symlinked Resources directory.' >&2
    exit 1
fi
test ! -e "$protected/Contents/Resources/assets"

unsafe_file="$temporary_root/unsafe-file"
unsafe_bundle="$unsafe_file/debug/RustUI Playground.app"
mkdir -p "$unsafe_bundle/Contents/Resources/assets"
printf '%s\n' 'protected resource' > "$protected/resource"
ln -s "$protected/resource" "$unsafe_bundle/Contents/Resources/assets/link-fixture.html"
if CARGO_TARGET_DIR="$unsafe_file" \
    RUSTUI_PLAYGROUND_BINARY="$headstart_binary" \
    RUSTUI_PLAYGROUND_REFERENCES_DIR="$references_source" \
    "$repository_root/apps/rustui-playground/package-app.sh" >/dev/null 2>&1; then
    printf '%s\n' 'package accepted a symlinked resource file.' >&2
    exit 1
fi
test "$(cat "$protected/resource")" = 'protected resource'

hardlink_root="$temporary_root/hardlink-target"
hardlink_bundle="$hardlink_root/debug/RustUI Playground.app"
mkdir -p "$hardlink_bundle/Contents/Resources/assets" "$hardlink_bundle/Contents/MacOS"
printf '%s\n' 'protected hardlinked resource' > "$protected/hardlink-resource"
printf '%s\n' 'protected hardlinked executable' > "$protected/hardlink-executable"
printf '%s\n' 'protected hardlinked plist' > "$protected/hardlink-plist"
ln "$protected/hardlink-resource" "$hardlink_bundle/Contents/Resources/assets/link-fixture.html"
ln "$protected/hardlink-executable" "$hardlink_bundle/Contents/MacOS/rustui-playground.next"
ln "$protected/hardlink-executable" "$hardlink_bundle/Contents/MacOS/rustui-playground"
ln "$protected/hardlink-plist" "$hardlink_bundle/Contents/Info.plist"
CARGO_TARGET_DIR="$hardlink_root" \
    RUSTUI_PLAYGROUND_BINARY="$headstart_binary" \
    RUSTUI_PLAYGROUND_REFERENCES_DIR="$references_source" \
    "$repository_root/apps/rustui-playground/package-app.sh" >/dev/null
test "$(cat "$protected/hardlink-resource")" = 'protected hardlinked resource'
test "$(cat "$protected/hardlink-executable")" = 'protected hardlinked executable'
test "$(cat "$protected/hardlink-plist")" = 'protected hardlinked plist'
cmp "$headstart_binary" "$hardlink_bundle/Contents/MacOS/rustui-playground"
cmp "$repository_root/assets/link-fixture.html" "$hardlink_bundle/Contents/Resources/assets/link-fixture.html"

copy_failure_root="$temporary_root/copy-failure"
copy_failure_bundle="$copy_failure_root/debug/RustUI Playground.app"
copy_shim="$temporary_root/copy-shim"
mkdir -p "$copy_failure_bundle/Contents/Resources/assets" "$copy_shim"
printf '%s\n' 'existing resource' > "$copy_failure_bundle/Contents/Resources/assets/link-fixture.html"
cat > "$copy_shim/cp" <<'EOF'
#!/bin/sh
exit 73
EOF
chmod 755 "$copy_shim/cp"
if PATH="$copy_shim:$PATH" CARGO_TARGET_DIR="$copy_failure_root" \
    RUSTUI_PLAYGROUND_BINARY="$headstart_binary" \
    RUSTUI_PLAYGROUND_REFERENCES_DIR="$references_source" \
    "$repository_root/apps/rustui-playground/package-app.sh" >/dev/null 2>&1; then
    printf '%s\n' 'package accepted a failed resource copy.' >&2
    exit 1
fi
test "$(cat "$copy_failure_bundle/Contents/Resources/assets/link-fixture.html")" = 'existing resource'
test -z "$(find "$copy_failure_bundle" -name '*.tmp.*' -print -quit)"

directory_root="$temporary_root/directory-output"
directory_bundle="$directory_root/debug/RustUI Playground.app"
mkdir -p "$directory_bundle/Contents/Info.plist"
if CARGO_TARGET_DIR="$directory_root" \
    RUSTUI_PLAYGROUND_BINARY="$headstart_binary" \
    RUSTUI_PLAYGROUND_REFERENCES_DIR="$references_source" \
    "$repository_root/apps/rustui-playground/package-app.sh" >/dev/null 2>&1; then
    printf '%s\n' 'package accepted a directory at the Info.plist output.' >&2
    exit 1
fi
test -z "$(find "$directory_bundle/Contents/Info.plist" -mindepth 1 -print -quit)"
test ! -e "$directory_bundle/Contents/Resources"

fifo_root="$temporary_root/fifo-output"
fifo_bundle="$fifo_root/debug/RustUI Playground.app"
mkdir -p "$fifo_bundle/Contents/Resources/assets"
mkfifo "$fifo_bundle/Contents/Resources/assets/link-fixture.html"
if CARGO_TARGET_DIR="$fifo_root" \
    RUSTUI_PLAYGROUND_BINARY="$headstart_binary" \
    RUSTUI_PLAYGROUND_REFERENCES_DIR="$references_source" \
    "$repository_root/apps/rustui-playground/package-app.sh" >/dev/null 2>&1; then
    printf '%s\n' 'package accepted a FIFO at a resource output.' >&2
    exit 1
fi
test -p "$fifo_bundle/Contents/Resources/assets/link-fixture.html"

printf '%s\n' 'Playground packaging reuse, rollback and output path safety passed.'
