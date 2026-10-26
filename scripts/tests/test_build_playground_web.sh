#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
temporary_root=$(mktemp -d "${TMPDIR:-/tmp}/rustui-playground-web-package-test.XXXXXX")
trap 'rm -rf "$temporary_root"' EXIT HUP INT TERM

fixture="$temporary_root/repo"
fake_bin="$temporary_root/bin"
export FIXTURE_ROOT="$fixture"
export CARGO_HOME="$temporary_root/cargo-home"
mkdir -p \
    "$fixture/scripts" \
    "$fixture/apps/rustui-playground/web" \
    "$fixture/apps/rustui-playground/references" \
    "$fixture/crates/rustui/assets/fonts" \
    "$fixture/vendor/gpui-kit-assets-0.7.3" \
    "$fixture/docs/release/dependency-notices" \
    "$temporary_root/cargo-home/registry/src" \
    "$temporary_root/cargo-home/git/checkouts" \
    "$fake_bin"
cp "$repository_root/scripts/build-playground-web.sh" "$fixture/scripts/"
cp "$repository_root/scripts/manifest_playground_web.py" "$fixture/scripts/"
cp "$repository_root/scripts/collect_dependency_notices.py" "$fixture/scripts/"
cp "$repository_root/scripts/package_release_notices.py" "$fixture/scripts/"
cp -R "$repository_root/docs/release/dependency-notices/." \
    "$fixture/docs/release/dependency-notices/"
cp "$repository_root/apps/rustui-playground/web/index.html" \
    "$fixture/apps/rustui-playground/web/"
for asset in reference-host.html reference-host.js reference-client.js; do
    cp "$repository_root/apps/rustui-playground/web/$asset" "$fixture/apps/rustui-playground/web/$asset"
done
cp -R "$repository_root/apps/rustui-playground/references/." \
    "$fixture/apps/rustui-playground/references/"
cp "$repository_root"/crates/rustui/assets/fonts/*-LICENSE \
    "$fixture/crates/rustui/assets/fonts/"
cp "$repository_root/vendor/gpui-kit-assets-0.7.3/LICENSE-LUCIDE" \
    "$fixture/vendor/gpui-kit-assets-0.7.3/"
printf '%s\n' 'fixture workspace license' > "$fixture/LICENSE"
python3 - "$fixture" <<'PY_INPUTS'
import json, runpy, sys
from pathlib import Path
root = Path(sys.argv[1])
manifest = runpy.run_path(str(root / "scripts/manifest_playground_web.py"))
for relative in manifest["SOURCE_ROOTS"]:
    (root / relative).mkdir(parents=True, exist_ok=True)
for relative in manifest["SOURCE_INPUTS"]:
    path = root / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.touch(exist_ok=True)
(root / "apps/rustui-playground/Cargo.toml").write_text(
    '[package]\nname = "rustui-playground"\nversion = "0.1.0"\n', encoding="utf-8")
package_id = "path+" + (root / "apps/rustui-playground/Cargo.toml").resolve().as_uri() + "#rustui-playground@0.1.0"
metadata = {"packages": [{"id": package_id, "name": "rustui-playground", "version": "0.1.0",
                          "source": None, "manifest_path": str(root / "apps/rustui-playground/Cargo.toml")}],
            "workspace_members": [package_id]}
(root / "fixture-metadata.json").write_text(json.dumps(metadata), encoding="utf-8")
build = [{"reason": "compiler-artifact", "package_id": package_id},
         {"reason": "build-finished", "success": True}]
(root / "fixture-build.jsonl").write_text("".join(json.dumps(item) + "\n" for item in build), encoding="utf-8")
PY_INPUTS

cat > "$fixture/scripts/cargo-headstart" <<'EOF'
#!/bin/sh
set -eu
if [ "${1:-}" = metadata ]; then
    cat "$FIXTURE_ROOT/fixture-metadata.json"
    exit 0
fi
output="$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/rustui-playground.wasm"
mkdir -p "$(dirname -- "$output")"
printf 'fixture wasm' > "$output"
cat "$FIXTURE_ROOT/fixture-build.jsonl"
EOF
cat > "$fake_bin/wasm-bindgen" <<'EOF'
#!/bin/sh
set -eu
if [ "${1:-}" = --version ]; then
    printf '%s\n' 'wasm-bindgen 0.2.129'
    exit 0
fi
out_dir=
while [ "$#" -gt 0 ]; do
    if [ "$1" = --out-dir ]; then
        out_dir=$2
        shift 2
    else
        shift
    fi
done
mkdir -p "$out_dir"
printf '%s\n' 'export default async function init() {}' > "$out_dir/rustui_playground.js"
if [ "${OMIT_WASM_BINDGEN_ARTIFACT:-0}" != 1 ]; then
    printf 'fixture wasm' > "$out_dir/rustui_playground_bg.wasm"
fi
EOF
cat > "$fake_bin/rust-lld" <<'EOF'
#!/bin/sh
exit 0
EOF
chmod +x "$fixture/scripts/cargo-headstart" "$fake_bin/wasm-bindgen" "$fake_bin/rust-lld"

PATH="$fake_bin:$PATH" \
CARGO_HOME="$temporary_root/cargo-home" \
FIXTURE_ROOT="$fixture" \
CARGO_TARGET_DIR="$temporary_root/cargo-target" \
RUSTUI_HEADSTART_ROOT="$temporary_root/unused-headstart" \
    "$fixture/scripts/build-playground-web.sh" >/dev/null

bundle="$fixture/target/headstart/playground-web"
test -f "$bundle/index.html"
for asset in reference-host.html reference-host.js reference-client.js; do
    cmp "$fixture/apps/rustui-playground/web/$asset" "$bundle/$asset"
done
test -f "$bundle/rustui_playground.js"
test -f "$bundle/rustui_playground_bg.wasm"
test -f "$bundle/references/Form.svg"
cmp "$repository_root/vendor/gpui-kit-assets-0.7.3/LICENSE-LUCIDE" \
    "$bundle/licenses/icons/LICENSE-LUCIDE"
python3 - "$bundle/bundle-manifest.json" "$bundle" <<'PY_MANIFEST'
import hashlib, json, sys
from pathlib import Path
bundle = Path(sys.argv[2])
manifest = json.loads(Path(sys.argv[1]).read_text())
assert len(manifest["source"]["sha256"]) == 64
files = {item["path"]: item["sha256"] for item in manifest["bundle"]["files"]}
for name in ("index.html", "rustui_playground.js", "rustui_playground_bg.wasm", "reference-host.html", "reference-host.js", "reference-client.js"):
    assert hashlib.sha256((bundle / name).read_bytes()).hexdigest() == files[name]
notice_report_path = bundle / "licenses/release/dependency-notices/report.json"
notice_report = json.loads(notice_report_path.read_text())
assert hashlib.sha256(notice_report_path.read_bytes()).hexdigest() == files["licenses/release/dependency-notices/report.json"]
assert "metadata_sha256" not in notice_report["inputs"]
assert "build_json_sha256" not in notice_report["inputs"]
assert notice_report["inputs"]["supplement_manifest_sha256"]
assert notice_report["complete_license_or_compatibility_claim"] is False
assert all("file://" not in item["package_id"] for item in notice_report["packages"]), "absolute Cargo path ID remains"
assert "scripts/collect_dependency_notices.py" in {item["path"] for item in manifest["source"]["files"]}, "collector missing from source provenance"
assert "scripts/package_release_notices.py" in {item["path"] for item in manifest["source"]["files"]}, "packaging helper missing from source provenance"
assert "docs/release/dependency-notices/manifest.json" in {item["path"] for item in manifest["source"]["files"]}, "supplement manifest missing from source provenance"
for package in notice_report["packages"]:
    for notice in package["notices"]:
        packaged_path = "licenses/release/dependency-notices/" + notice["path"]
        assert packaged_path in files
        assert hashlib.sha256((bundle / packaged_path).read_bytes()).hexdigest() == files[packaged_path]
PY_MANIFEST
grep -Fq '<body>' "$bundle/index.html"
grep -Fq 'script nomodule' "$bundle/index.html"
grep -Fq 'does not support the JavaScript modules required by RustUI Playground' "$bundle/index.html"
grep -Fq '<div id="startup-status" role="status">' "$bundle/index.html"
grep -Fq 'Loading RustUI Playground' "$bundle/index.html"
! grep -Fq 'aria-busy' "$bundle/index.html"
for license in "$repository_root"/crates/rustui/assets/fonts/*-LICENSE; do
    name=$(basename -- "$license")
    cmp "$license" "$bundle/licenses/fonts/$name"
done

cat > "$fixture/apps/rustui-playground/web/index.html" <<'EOF'
<!doctype html><html><body><script
  defer
  type="module">
const { default: init } = await import("./rustui_playground.js");
await init();
</script></body></html>
EOF
env PATH="$fake_bin:$PATH" CARGO_TARGET_DIR="$temporary_root/cargo-target" RUSTUI_HEADSTART_ROOT="$temporary_root/unused-headstart" "$fixture/scripts/build-playground-web.sh" >/dev/null

cat > "$fixture/apps/rustui-playground/web/index.html" <<'EOF'
<!doctype html><html><body><SCRIPT defer TYPE = 'module'>
const { default: init } = await import("./rustui_playground.js");
await init();
</SCRIPT></body></html>
EOF
PATH="$fake_bin:$PATH" \
    CARGO_TARGET_DIR="$temporary_root/cargo-target" \
    RUSTUI_HEADSTART_ROOT="$temporary_root/unused-headstart" \
        "$fixture/scripts/build-playground-web.sh" >/dev/null
printf '%s\n' 'existing bundle sentinel' >"$bundle/preserved-on-failure.txt"

cat > "$fixture/apps/rustui-playground/web/index.html" <<'EOF'
<!doctype html><html><body><script data-type = 'module'>
const { default: init } = await import("./rustui_playground.js");
await init();
</script></body></html>
EOF
if PATH="$fake_bin:$PATH" \
    CARGO_TARGET_DIR="$temporary_root/cargo-target" \
    RUSTUI_HEADSTART_ROOT="$temporary_root/unused-headstart" \
        "$fixture/scripts/build-playground-web.sh" >"$temporary_root/data-type.out" 2>&1; then
    printf '%s\n' 'packager accepted a data-type attribute as a module script.' >&2
    exit 1
fi
grep -Fq 'Web browser shell does not load rustui_playground.js' "$temporary_root/data-type.out"
test "$(cat "$bundle/preserved-on-failure.txt")" = 'existing bundle sentinel'

cat > "$fixture/apps/rustui-playground/web/index.html" <<'EOF'
<!doctype html><html><body><script type="modulex">
const { default: init } = await import("./rustui_playground.js");
await init();
</script></body></html>
EOF
if PATH="$fake_bin:$PATH" \
    CARGO_TARGET_DIR="$temporary_root/cargo-target" \
    RUSTUI_HEADSTART_ROOT="$temporary_root/unused-headstart" \
        "$fixture/scripts/build-playground-web.sh" >"$temporary_root/module-prefix.out" 2>&1; then
    printf '%s\n' 'packager accepted a longer script type value as a module script.' >&2
    exit 1
fi
grep -Fq 'Web browser shell does not load rustui_playground.js' "$temporary_root/module-prefix.out"
test "$(cat "$bundle/preserved-on-failure.txt")" = 'existing bundle sentinel'

cat > "$fixture/apps/rustui-playground/web/index.html" <<'EOF'
<!doctype html><html><body>
<!-- Documentation example uses a ` character. -->
<script type="module">
const { default: init } = await import("./rustui_playground.js");
await init();
</script></body></html>
EOF
PATH="$fake_bin:$PATH" \
    CARGO_TARGET_DIR="$temporary_root/cargo-target" \
    RUSTUI_HEADSTART_ROOT="$temporary_root/unused-headstart" \
        "$fixture/scripts/build-playground-web.sh" >/dev/null

cat > "$fixture/apps/rustui-playground/web/index.html" <<'EOF'
<!doctype html><html><body><!-- compact-shell --> <script type="module">
const { default: init } = await import("./rustui_playground.js");
await init();
</script></body></html>
EOF
PATH="$fake_bin:$PATH" \
    CARGO_TARGET_DIR="$temporary_root/cargo-target" \
    RUSTUI_HEADSTART_ROOT="$temporary_root/unused-headstart" \
        "$fixture/scripts/build-playground-web.sh" >/dev/null

cat > "$fixture/apps/rustui-playground/web/index.html" <<'EOF'
<!doctype html><html><body><script type="module">
const commentText = "<!--";
const { default: init } = await import("./rustui_playground.js");
await init();
</script></body></html>
EOF
PATH="$fake_bin:$PATH" \
    CARGO_TARGET_DIR="$temporary_root/cargo-target" \
    RUSTUI_HEADSTART_ROOT="$temporary_root/unused-headstart" \
        "$fixture/scripts/build-playground-web.sh" >/dev/null
test -s "$bundle/rustui_playground.js"

printf '%s\n' 'existing bundle sentinel' > "$bundle/preserved-on-failure.txt"
cat > "$fixture/apps/rustui-playground/web/index.html" <<'EOF'
<!doctype html><html><body><script type="module">
const { default: init } = await import("./rustui_playground.js");
</script></body></html>
EOF
if PATH="$fake_bin:$PATH" \
    CARGO_TARGET_DIR="$temporary_root/cargo-target" \
    RUSTUI_HEADSTART_ROOT="$temporary_root/unused-headstart" \
        "$fixture/scripts/build-playground-web.sh" >"$temporary_root/missing-initializer.out" 2>&1; then
    printf '%s\n' 'packager accepted a generated module import without starting the Playground.' >&2
    exit 1
fi
grep -Fq 'does not initialize rustui_playground.js' "$temporary_root/missing-initializer.out"
test "$(cat "$bundle/preserved-on-failure.txt")" = 'existing bundle sentinel'
test -f "$bundle/rustui_playground_bg.wasm"
if find "$fixture/target/headstart" -maxdepth 1 -name '.playground-web.*' -print -quit | grep -q .; then
    printf '%s\n' 'packager left a staging directory after rejecting a shell without initialization.' >&2
    exit 1
fi

cat > "$fixture/apps/rustui-playground/web/index.html" <<'EOF'
<!doctype html><html><body><script type="module">
const { default: start } = await import("./rustui_playground.js");
await init();
</script></body></html>
EOF
if PATH="$fake_bin:$PATH" \
    CARGO_TARGET_DIR="$temporary_root/cargo-target" \
    RUSTUI_HEADSTART_ROOT="$temporary_root/unused-headstart" \
        "$fixture/scripts/build-playground-web.sh" >"$temporary_root/mismatched-initializer.out" 2>&1; then
    printf '%s\n' 'packager accepted an initializer name that does not match the imported default export.' >&2
    exit 1
fi
grep -Fq 'does not initialize rustui_playground.js' "$temporary_root/mismatched-initializer.out"
test "$(cat "$bundle/preserved-on-failure.txt")" = 'existing bundle sentinel'
test -f "$bundle/rustui_playground_bg.wasm"
if find "$fixture/target/headstart" -maxdepth 1 -name '.playground-web.*' -print -quit | grep -q .; then
    printf '%s\n' 'packager left a staging directory after rejecting a mismatched initializer.' >&2
    exit 1
fi

cat > "$fixture/apps/rustui-playground/web/index.html" <<'EOF'
<!doctype html><html><body>
<script type="module">
const { default: init } = await import("./rustui_playground.js");
</script>
<script type="module">
await init();
</script>
</body></html>
EOF
if PATH="$fake_bin:$PATH" \
    CARGO_TARGET_DIR="$temporary_root/cargo-target" \
    RUSTUI_HEADSTART_ROOT="$temporary_root/unused-headstart" \
        "$fixture/scripts/build-playground-web.sh" >"$temporary_root/separate-module.out" 2>&1; then
    printf '%s\n' 'packager accepted an initializer call outside the importing module script.' >&2
    exit 1
fi
grep -Fq 'does not initialize rustui_playground.js' "$temporary_root/separate-module.out"
test "$(cat "$bundle/preserved-on-failure.txt")" = 'existing bundle sentinel'

if PATH="$fake_bin:$PATH" \
    CARGO_TARGET_DIR="$temporary_root/cargo-target" \
    RUSTUI_HEADSTART_ROOT="$temporary_root/unused-headstart" \
    OMIT_WASM_BINDGEN_ARTIFACT=1 \
        "$fixture/scripts/build-playground-web.sh" >"$temporary_root/failure.out" 2>&1; then
    printf '%s\n' 'packager accepted a missing generated Wasm artifact.' >&2
    exit 1
fi
grep -Fq 'did not produce a non-empty rustui_playground_bg.wasm' "$temporary_root/failure.out"
test "$(cat "$bundle/preserved-on-failure.txt")" = 'existing bundle sentinel'
test -f "$bundle/rustui_playground_bg.wasm"
if find "$fixture/target/headstart" -maxdepth 1 -name '.playground-web.*' -print -quit | grep -q .; then
    printf '%s\n' 'packager left a staging directory after rejecting incomplete output.' >&2
    exit 1
fi

: > "$fixture/apps/rustui-playground/web/index.html"
if PATH="$fake_bin:$PATH" \
    CARGO_TARGET_DIR="$temporary_root/cargo-target" \
    RUSTUI_HEADSTART_ROOT="$temporary_root/unused-headstart" \
        "$fixture/scripts/build-playground-web.sh" >"$temporary_root/empty-shell.out" 2>&1; then
    printf '%s\n' 'packager accepted an empty browser shell.' >&2
    exit 1
fi
grep -Fq 'Web browser shell is empty' "$temporary_root/empty-shell.out"
test "$(cat "$bundle/preserved-on-failure.txt")" = 'existing bundle sentinel'
test -f "$bundle/rustui_playground_bg.wasm"
if find "$fixture/target/headstart" -maxdepth 1 -name '.playground-web.*' -print -quit | grep -q .; then
    printf '%s\n' 'packager left a staging directory after rejecting an empty browser shell.' >&2
    exit 1
fi

printf '%s\n' '<!doctype html><html><body></body></html>' \
    > "$fixture/apps/rustui-playground/web/index.html"
if PATH="$fake_bin:$PATH" \
    CARGO_TARGET_DIR="$temporary_root/cargo-target" \
    RUSTUI_HEADSTART_ROOT="$temporary_root/unused-headstart" \
        "$fixture/scripts/build-playground-web.sh" >"$temporary_root/missing-module.out" 2>&1; then
    printf '%s\n' 'packager accepted a browser shell without the generated module import.' >&2
    exit 1
fi
grep -Fq 'Web browser shell does not load rustui_playground.js' "$temporary_root/missing-module.out"
test "$(cat "$bundle/preserved-on-failure.txt")" = 'existing bundle sentinel'
test -f "$bundle/rustui_playground_bg.wasm"
if find "$fixture/target/headstart" -maxdepth 1 -name '.playground-web.*' -print -quit | grep -q .; then
    printf '%s\n' 'packager left a staging directory after rejecting a shell without the module import.' >&2
    exit 1
fi

cat > "$fixture/apps/rustui-playground/web/index.html" <<'EOF'
<!doctype html><html><body><script type="application/json">
const { default: init } = await import("./rustui_playground.js");
</script></body></html>
EOF
if PATH="$fake_bin:$PATH" \
    CARGO_TARGET_DIR="$temporary_root/cargo-target" \
    RUSTUI_HEADSTART_ROOT="$temporary_root/unused-headstart" \
        "$fixture/scripts/build-playground-web.sh" >"$temporary_root/non-module.out" 2>&1; then
    printf '%s\n' 'packager accepted a generated module import inside a non-executable script block.' >&2
    exit 1
fi
grep -Fq 'Web browser shell does not load rustui_playground.js' "$temporary_root/non-module.out"
test "$(cat "$bundle/preserved-on-failure.txt")" = 'existing bundle sentinel'
test -f "$bundle/rustui_playground_bg.wasm"
if find "$fixture/target/headstart" -maxdepth 1 -name '.playground-web.*' -print -quit | grep -q .; then
    printf '%s\n' 'packager left a staging directory after rejecting a non-module script block.' >&2
    exit 1
fi

cat > "$fixture/apps/rustui-playground/web/index.html" <<'EOF'
<!doctype html><html><body><script type="module">
const note = 'const { default: init } = await import("./rustui_playground.js");';
</script></body></html>
EOF
if PATH="$fake_bin:$PATH" \
    CARGO_TARGET_DIR="$temporary_root/cargo-target" \
    RUSTUI_HEADSTART_ROOT="$temporary_root/unused-headstart" \
        "$fixture/scripts/build-playground-web.sh" >"$temporary_root/string-module.out" 2>&1; then
    printf '%s\n' 'packager accepted a browser module import that only appears inside a string.' >&2
    exit 1
fi
grep -Fq 'Web browser shell does not load rustui_playground.js' "$temporary_root/string-module.out"
test "$(cat "$bundle/preserved-on-failure.txt")" = 'existing bundle sentinel'
test -f "$bundle/rustui_playground_bg.wasm"
if find "$fixture/target/headstart" -maxdepth 1 -name '.playground-web.*' -print -quit | grep -q .; then
    printf '%s\n' 'packager left a staging directory after rejecting a string-only module reference.' >&2
    exit 1
fi

printf '%s\n' '<!doctype html><html><body><script type="module">// import("./rustui_playground.js")</script></body></html>' \
    > "$fixture/apps/rustui-playground/web/index.html"
if PATH="$fake_bin:$PATH" \
    CARGO_TARGET_DIR="$temporary_root/cargo-target" \
    RUSTUI_HEADSTART_ROOT="$temporary_root/unused-headstart" \
        "$fixture/scripts/build-playground-web.sh" >"$temporary_root/commented-module.out" 2>&1; then
    printf '%s\n' 'packager accepted a commented-out browser module import.' >&2
    exit 1
fi
grep -Fq 'Web browser shell does not load rustui_playground.js' "$temporary_root/commented-module.out"
test "$(cat "$bundle/preserved-on-failure.txt")" = 'existing bundle sentinel'
test -f "$bundle/rustui_playground_bg.wasm"
if find "$fixture/target/headstart" -maxdepth 1 -name '.playground-web.*' -print -quit | grep -q .; then
    printf '%s\n' 'packager left a staging directory after rejecting a commented-out module import.' >&2
    exit 1
fi

cat > "$fixture/apps/rustui-playground/web/index.html" <<'EOF'
<!doctype html><html><body><script type="module">
/*
const { default: init } = await import("./rustui_playground.js");
*/
</script></body></html>
EOF
if PATH="$fake_bin:$PATH" \
    CARGO_TARGET_DIR="$temporary_root/cargo-target" \
    RUSTUI_HEADSTART_ROOT="$temporary_root/unused-headstart" \
        "$fixture/scripts/build-playground-web.sh" >"$temporary_root/block-commented-module.out" 2>&1; then
    printf '%s\n' 'packager accepted a block-commented browser module import.' >&2
    exit 1
fi
grep -Fq 'Web browser shell does not load rustui_playground.js' "$temporary_root/block-commented-module.out"
test "$(cat "$bundle/preserved-on-failure.txt")" = 'existing bundle sentinel'
test -f "$bundle/rustui_playground_bg.wasm"
if find "$fixture/target/headstart" -maxdepth 1 -name '.playground-web.*' -print -quit | grep -q .; then
    printf '%s\n' 'packager left a staging directory after rejecting a block-commented module import.' >&2
    exit 1
fi

cat > "$fixture/apps/rustui-playground/web/index.html" <<'EOF'
<!doctype html><html><body><script type="module">
const example = `
const { default: init } = await import("./rustui_playground.js");
await init();
`;
</script></body></html>
EOF
if PATH="$fake_bin:$PATH" \
    CARGO_TARGET_DIR="$temporary_root/cargo-target" \
    RUSTUI_HEADSTART_ROOT="$temporary_root/unused-headstart" \
        "$fixture/scripts/build-playground-web.sh" >"$temporary_root/template-module.out" 2>&1; then
    printf '%s\n' 'packager accepted a browser bootstrap that appears only inside a template string.' >&2
    exit 1
fi
grep -Fq 'Web browser shell does not load rustui_playground.js' "$temporary_root/template-module.out"
test "$(cat "$bundle/preserved-on-failure.txt")" = 'existing bundle sentinel'
test -f "$bundle/rustui_playground_bg.wasm"

cat > "$fixture/apps/rustui-playground/web/index.html" <<'EOF'
<!doctype html>
<!--
<script type="module">
const { default: init } = await import("./rustui_playground.js");
</script>
-->
<body></body>
EOF
if PATH="$fake_bin:$PATH" \
    CARGO_TARGET_DIR="$temporary_root/cargo-target" \
    RUSTUI_HEADSTART_ROOT="$temporary_root/unused-headstart" \
        "$fixture/scripts/build-playground-web.sh" >"$temporary_root/html-commented-module.out" 2>&1; then
    printf '%s\n' 'packager accepted an HTML-commented browser module import.' >&2
    exit 1
fi
grep -Fq 'Web browser shell does not load rustui_playground.js' "$temporary_root/html-commented-module.out"
test "$(cat "$bundle/preserved-on-failure.txt")" = 'existing bundle sentinel'
test -f "$bundle/rustui_playground_bg.wasm"
if find "$fixture/target/headstart" -maxdepth 1 -name '.playground-web.*' -print -quit | grep -q .; then
    printf '%s\n' 'packager left a staging directory after rejecting an HTML-commented module import.' >&2
    exit 1
fi

cat > "$fixture/apps/rustui-playground/web/index.html" <<'EOF'
<!doctype html><html><body><script type="module">
const example = `escaped \` backtick
const { default: init } = await import("./rustui_playground.js");
await init();
`;
</script></body></html>
EOF
if PATH="$fake_bin:$PATH" \
    CARGO_TARGET_DIR="$temporary_root/cargo-target" \
    RUSTUI_HEADSTART_ROOT="$temporary_root/unused-headstart" \
        "$fixture/scripts/build-playground-web.sh" >"$temporary_root/escaped-template-module.out" 2>&1; then
    printf '%s\n' 'packager accepted a browser bootstrap that appears only inside a template string with an escaped backtick.' >&2
    exit 1
fi
grep -Fq 'Web browser shell does not load rustui_playground.js' "$temporary_root/escaped-template-module.out"
test "$(cat "$bundle/preserved-on-failure.txt")" = 'existing bundle sentinel'
test -f "$bundle/rustui_playground_bg.wasm"
if find "$fixture/target/headstart" -maxdepth 1 -name '.playground-web.*' -print -quit | grep -q .; then
    printf '%s\n' 'packager left a staging directory after rejecting an escaped-template decoy.' >&2
    exit 1
fi

printf '%s\n' 'Web package validates generated entrypoints and browser module bootstrap, preserving the prior bundle on incomplete output.'
