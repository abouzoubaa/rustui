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

printf '%s\n' "Playground package replaces stale SVG references and preserves them if promotion fails."
