# GPUI Kit 0.8.x Upgrade Plan

## Scope and method

Assessment only. No repository code, vendor, patch, or lockfile files were changed for this plan. The target is the latest 0.8.x release visible on the upstream GPUI Kit releases page: **v0.8.1**, commit `99576db58ec8103417c41a092e891b4022d24e7e`. Its release manifest pins **gpui-pre 0.3.10**. Sources: [GPUI Kit v0.8.1 release notes](https://github.com/longbridge/gpui-kit/releases/tag/v0.8.1), [tag Cargo.toml](https://raw.githubusercontent.com/longbridge/gpui-kit/v0.8.1/Cargo.toml).

Commands below were run from `/Users/Abium/Developer/rustui`, except target source and lock resolution commands explicitly run under `/tmp`.

## 1. Current state on `origin/main`

Commands: `git rev-parse origin/main`; `find vendor -maxdepth 1 -type d -name 'gpui*'`; Python read of each matching vendor `Cargo.toml` package name/version; `cat .cargo/config.toml`.

Observed baseline: `origin/main` = `a0cd3e2da0a5df731daaece78dec9bf5185d9e4a`; `.cargo/config.toml` replaces `crates-io` with `vendored-sources` at `vendor/`.

| Family | Vendored versions today |
|---|---|
| Kit | `gpui-base`, `gpui-component`, `gpui-component-macros`, `gpui-kit-assets` = **0.7.3** |
| GPUI snapshot | `gpui-pre`, `gpui-pre-apple`, `gpui-pre-collections`, `gpui-pre-derive-refineable`, `gpui-pre-http-client`, `gpui-pre-linux`, `gpui-pre-macos`, `gpui-pre-macros`, `gpui-pre-platform`, `gpui-pre-refineable`, `gpui-pre-scheduler`, `gpui-pre-shared-string`, `gpui-pre-sum-tree`, `gpui-pre-util`, `gpui-pre-web`, `gpui-pre-wgpu`, `gpui-pre-windows`, `gpui-pre-ztracing`, `gpui-pre-ztracing-macro` = **0.3.9** |
| GPUI HTTP dependency | `gpui-pre-reqwest` = **0.12.15** |

Owned mirrors present on main, as mapped by `scripts/verify_owned_gpui_sources.py`:

| Owned crate | Vendor source |
|---|---|
| `crates/rustui-gpui` | `vendor/gpui-pre-0.3.9` |
| `crates/rustui-gpui-platform` | `vendor/gpui-pre-platform-0.3.9` |
| `crates/rustui-gpui-macos` | `vendor/gpui-pre-macos-0.3.9` |
| `crates/rustui-gpui-apple` | `vendor/gpui-pre-apple-0.3.9` |
| `crates/rustui-gpui-base` | `vendor/gpui-base-0.7.3` |
| `crates/rustui-gpui-component` | `vendor/gpui-component-0.7.3` |
| `crates/rustui-gpui-kit-assets` | `vendor/gpui-kit-assets-0.7.3` |
| `crates/rustui-gpui-linux` | `vendor/gpui-pre-linux-0.3.9` |
| `crates/rustui-gpui-web` | `vendor/gpui-pre-web-0.3.9` |
| `crates/rustui-gpui-wgpu` | `vendor/gpui-pre-wgpu-0.3.9` |
| `crates/rustui-gpui-windows` | `vendor/gpui-pre-windows-0.3.9` |
| `crates/rustui-block` | `vendor/block-0.1.6` |
| `crates/rustui-cocoa` | `vendor/cocoa-0.26.1` |
| `crates/rustui-cocoa-foundation` | `vendor/cocoa-foundation-0.2.1` |
| `crates/rustui-metal` | `vendor/metal-0.33.0` |
| `crates/rustui-objc-foundation` | `vendor/objc-foundation-0.1.1` |

## 2. Target and disposable lock resolution

The v0.8.1 release manifest pins `gpui-pre`, `gpui-pre-platform`, `gpui-pre-web`, `gpui-pre-macros`, `gpui-pre-reqwest-client`, and `gpui-pre-sum-tree` to **=0.3.10**; the tagged Cargo.toml also lists `gpui-pre-reqwest =0.12.15`. The snapshot packages published for this GPUI Kit release are version 0.3.10, including platform packages `gpui-pre-macos`, `gpui-pre-apple`, `gpui-pre-linux`, `gpui-pre-windows`, `gpui-pre-wgpu`, and `gpui-pre-ios`. GPUI Kit packages `gpui-base`, `gpui-component`, `gpui-component-macros`, and `gpui-kit-assets` move from **0.7.3 to 0.8.1**. GPUI Fast is an optional alternative backend at **0.1.12** (`gpui-fast`, `gpui-fast-platform`, `gpui-fast-web`); it is not required to target Kit 0.8.1.

Crate version changes for a full source snapshot refresh: all 0.3.9 GPUI snapshot crates above → 0.3.10; `gpui-base`, `gpui-component`, `gpui-component-macros`, `gpui-kit-assets` 0.7.3 → 0.8.1. `gpui-pre-reqwest` remains 0.12.15. `gpui-pre-ios` is a new platform package not present in the current vendor inventory.

Disposable resolution command: in `/tmp/a2-plan-lockprobe`, create a probe manifest with the five GPUI dependencies used by RustUI, copy in the current `Cargo.lock`, then run `cargo metadata --format-version 1` (no `--locked`). Probe features were GPUI `test-support`, platform `font-kit` + `runtime_shaders`, base default features off, component default features off + `tree-sitter-rust`, and assets. Exit: **0**. Cargo reported “Locking 114 packages to latest Rust 1.97.1 compatible versions.” A comparison of the resulting probe lock to repository `Cargo.lock` found 27 shared registry-package version changes, 66 target-only registry packages, and 26 old-only packages. Among these, 12 are GPUI-family entries (including the GPUI Kit packages and snapshot platform crates); thus the disposable GPUI closure indicates:

- 15 existing third-party package versions change: `naga 29.0.4→30.0.1`, `taffy 0.13.0→0.15.1`, `tree-sitter 0.27.1→0.26.13`, `wgpu`, `wgpu-core`, `wgpu-core-deps-apple`, `wgpu-core-deps-emscripten`, `wgpu-core-deps-wasm`, `wgpu-core-deps-windows-linux-android`, `wgpu-hal`, `wgpu-naga-bridge`, `wgpu-types` all `29.0.4→30.0.1`; `windows-registry 0.4.0→0.6.1`; `zbus 5.19.0→6.2.1`; `zbus_macros 5.19.0→6.2.1`.
- 54 third-party packages are newly present in that closure: `accesskit_windows`, `aes`, `block`, `block-padding`, `calloop`, `cbc`, `cipher`, `cocoa`, `cocoa-foundation`, `core-graphics-helmer-fork`, `cosmic-text`, `deranged`, `dispatch`, `dunce`, `generic-array`, `hkdf`, `hmac`, `inout`, `mac-notification-sys`, `md-5`, `metal`, `naga-types`, `notify-rust`, `ntapi`, `num`, `num-bigint-dig`, `num-conv`, `num-iter`, `objc-foundation`, `objc2-ui-kit`, `objc_exception`, `objc_id`, `oo7`, `pbkdf2`, `powerfmt`, `quick-xml`, `screencapturekit`, `screencapturekit-sys`, `serde_bytes`, `sha1_smol`, `sysinfo`, `tao-core-video-sys`, `tauri-winrt-notification`, `time`, `time-core`, `wasm_thread`, `windows-capture`, `windows-version`, `x11`, `xcb`, `zbus_utils`, `zed-scap`, `zeroize_derive`, `zruntime`.

These counts are a resolved isolated GPUI dependency closure, not yet the exact full-workspace lock diff: the integrated lock must be resolved again after RustUI, Playground, lab, platform-target and patch feature choices are ported. Do not copy the probe lock into the repo.

## 3. Approved patch compatibility matrix

Source acquisition: `git clone --depth 1 --branch v0.8.1 https://github.com/longbridge/gpui-kit.git /tmp/a2-plan-gpui-kit-v0.8.1`; downloaded exact 0.8.1 / 0.3.10 published source archives into `/tmp/a2-plan-crates`. Disposable source tree `/tmp/a2-plan-patchcheck` placed those sources at the versioned paths used in patch headers; each approved patch was checked with `git apply --check <absolute-patch-path>` from that disposable tree. No patch was applied to the repository. `APPLIES` means all patch hunks passed that check against the downloaded target source; `REBASE` means check failed and names the first failing file(s). This is only the syntactic result, not a behavior verdict. The matrix below preserves the first-pass counts (29 APPLIES / 40 REBASE / 2 UNKNOWN); the intent review that follows reclassifies all 71 patches. The two third-party UNKNOWN apply checks were later repeated against pristine downloads and passed.

Target source package contents are used, but patch stacks are not simulated together; patches that intentionally build on an earlier patch may need a stacked disposable check before implementation. Use the table as the first rebase gate, not as a promotion decision.

| Patch | Target file(s) | Status against target |
|---|---|---|
| `a11y-validation-grid-v1.patch` | `vendor/gpui-component-0.7.3/src/input/input.rs`<br>`vendor/gpui-pre-0.3.9/src/elements/div.rs`<br>`vendor/gpui-pre-0.3.9/src/window/a11y/debug.rs` | REBASE: vendor/gpui-component-0.7.3/src/input/input.rs |
| `block-0.1.6-stack-symbol-v1.patch` | `vendor/block-0.1.6/src/lib.rs` | UNKNOWN: Unchanged third-party pin; staged source is current already-patched local source |
| `cosmic-text-initial-bidi-span-v1.patch` | `vendor/cosmic-text-0.19.0/src/shape.rs` | UNKNOWN: Unchanged third-party pin; staged source is current already-patched local source |
| `g10-labelled-by-tree-v1.patch` | `vendor/gpui-pre-0.3.9/src/window/a11y/debug.rs`<br>`crates/rustui-gpui/src/window/a11y/debug.rs` | REBASE: vendor/gpui-pre-0.3.9/src/window/a11y/debug.rs; crates/rustui-gpui/src/window/a11y/debug.rs |
| `g10-parent-node-id-v1.patch` | `vendor/gpui-pre-0.3.9/src/window/a11y.rs`<br>`crates/rustui-gpui/src/window/a11y.rs` | APPLIES: — |
| `gpui-base-button-disabled-accessibility-v1.patch` | `vendor/gpui-base-0.7.3/src/button.rs` | REBASE: vendor/gpui-base-0.7.3/src/button.rs |
| `gpui-base-calendar-date-limits-v1.patch` | `vendor/gpui-base-0.7.3/src/calendar.rs` | REBASE: vendor/gpui-base-0.7.3/src/calendar.rs |
| `gpui-base-disabled-semantic-roots-v1.patch` | `vendor/gpui-base-0.7.3/src/select.rs`<br>`vendor/gpui-base-0.7.3/src/date_picker.rs`<br>`vendor/gpui-base-0.7.3/src/input/base/mod.rs` | REBASE: vendor/gpui-base-0.7.3/src/date_picker.rs |
| `gpui-base-focus-trap-accessibility-v1.patch` | `vendor/gpui-base-0.7.3/src/focus_trap.rs` | APPLIES: — |
| `gpui-base-resize-first-move-v1.patch` | `vendor/gpui-base-0.7.3/src/resizable/panel.rs` | REBASE: vendor/gpui-base-0.7.3/src/resizable/panel.rs |
| `gpui-base-resize-first-move-v2.patch` | `vendor/gpui-base-0.7.3/src/resizable/panel.rs` | REBASE: vendor/gpui-base-0.7.3/src/resizable/panel.rs |
| `gpui-base-resize-first-move-v3.patch` | `vendor/gpui-base-0.7.3/src/resizable/panel.rs` | REBASE: vendor/gpui-base-0.7.3/src/resizable/panel.rs |
| `gpui-base-slider-endpoint-v1.patch` | `vendor/gpui-base-0.7.3/src/slider.rs` | REBASE: vendor/gpui-base-0.7.3/src/slider.rs |
| `gpui-base-slider-first-move-v1.patch` | `vendor/gpui-base-0.7.3/src/slider.rs` | APPLIES: — |
| `gpui-base-slider-set-value-v2.patch` | `vendor/gpui-base-0.7.3/src/slider.rs` | REBASE: vendor/gpui-base-0.7.3/src/slider.rs |
| `gpui-base-slider-snap-position-v1.patch` | `vendor/gpui-base-0.7.3/src/slider.rs` | APPLIES: — |
| `gpui-base-slider-zero-track-v1.patch` | `vendor/gpui-base-0.7.3/src/slider.rs` | APPLIES: — |
| `gpui-bidi-input-hit-v1.patch` | `vendor/gpui-pre-0.3.9/src/text_system/line_layout.rs`<br>`vendor/gpui-pre-0.3.9/src/text_system/line.rs`<br>`vendor/gpui-pre-0.3.9/src/platform.rs`<br>`vendor/gpui-pre-web-0.3.9/src/run_replacements.rs`<br>`vendor/gpui-pre-wgpu-0.3.9/src/cosmic_text_system.rs`<br>`vendor/gpui-pre-apple-0.3.9/src/text_system.rs`<br>`vendor/gpui-pre-windows-0.3.9/src/direct_write.rs`<br>`vendor/gpui-base-0.7.3/src/text/inline.rs` | REBASE: vendor/gpui-pre-0.3.9/src/text_system/line_layout.rs |
| `gpui-bidi-wrapped-rows-v1.patch` | `vendor/gpui-pre-0.3.9/src/platform.rs`<br>`vendor/gpui-pre-0.3.9/src/text_system/line.rs`<br>`vendor/gpui-pre-0.3.9/src/text_system/line_layout.rs`<br>`vendor/gpui-pre-web-0.3.9/src/text_system.rs`<br>`vendor/gpui-pre-wgpu-0.3.9/src/cosmic_text_system.rs` | REBASE: vendor/gpui-pre-0.3.9/src/platform.rs |
| `gpui-calendar-keyboard-v1.patch` | `vendor/gpui-base-0.7.3/src/calendar.rs`<br>`vendor/gpui-component-0.7.3/src/time/date_picker.rs` | APPLIES: — |
| `gpui-calendar-multi-month-heading-v1.patch` | `vendor/gpui-base-0.7.3/src/calendar.rs` | REBASE: vendor/gpui-base-0.7.3/src/calendar.rs |
| `gpui-calendar-rtl-v1.patch` | `vendor/gpui-base-0.7.3/src/calendar.rs`<br>`vendor/gpui-component-0.7.3/src/time/calendar.rs`<br>`vendor/gpui-component-0.7.3/src/time/date_picker.rs` | REBASE: vendor/gpui-base-0.7.3/src/calendar.rs; vendor/gpui-component-0.7.3/src/time/date_picker.rs |
| `gpui-component-calendar-fr-ar-v1.patch` | `vendor/gpui-component-0.7.3/locales/ui.yml` | REBASE: vendor/gpui-component-0.7.3/locales/ui.yml |
| `gpui-component-date-picker-accessibility-v1.patch` | `vendor/gpui-component-0.7.3/src/time/date_picker.rs` | APPLIES: — |
| `gpui-component-date-picker-localization-v2.patch` | `vendor/gpui-component-0.7.3/src/time/date_picker.rs` | APPLIES: — |
| `gpui-component-date-picker-rtl-alignment-v1.patch` | `vendor/gpui-component-0.7.3/src/time/date_picker.rs` | REBASE: vendor/gpui-component-0.7.3/src/time/date_picker.rs |
| `gpui-component-popup-menu-disabled-confirm-v1.patch` | `vendor/gpui-component-0.7.3/src/menu/popup_menu.rs` | REBASE: vendor/gpui-component-0.7.3/src/menu/popup_menu.rs |
| `gpui-component-select-committed-title-v1.patch` | `vendor/gpui-component-0.7.3/src/select.rs` | REBASE: vendor/gpui-component-0.7.3/src/select.rs |
| `gpui-component-select-popup-rtl-v1.patch` | `vendor/gpui-component-0.7.3/src/list/list.rs`<br>`vendor/gpui-component-0.7.3/src/searchable_list/adapter.rs`<br>`vendor/gpui-component-0.7.3/src/searchable_list/item.rs` | REBASE: vendor/gpui-component-0.7.3/src/list/list.rs |
| `gpui-component-select-rtl-alignment-v1.patch` | `vendor/gpui-component-0.7.3/src/select.rs` | APPLIES: — |
| `gpui-component-select-rtl-alignment-v2.patch` | `vendor/gpui-component-0.7.3/src/select.rs` | APPLIES: — |
| `gpui-component-select-rtl-alignment-v3.patch` | `vendor/gpui-component-0.7.3/src/select.rs` | APPLIES: — |
| `gpui-component-select-rtl-auto-width-v4.patch` | `vendor/gpui-component-0.7.3/src/select.rs` | APPLIES: — |
| `gpui-component-select-rtl-auto-width-v6.patch` | `vendor/gpui-component-0.7.3/src/select.rs` | APPLIES: — |
| `gpui-component-sheet-close-accessibility-v2.patch` | `vendor/gpui-component-0.7.3/src/sheet.rs` | APPLIES: — |
| `gpui-component-slider-accessibility-label-v2.patch` | `vendor/gpui-component-0.7.3/src/slider.rs` | REBASE: vendor/gpui-component-0.7.3/src/slider.rs |
| `gpui-component-tracked-dialog-focus-rebase-v2.patch` | `vendor/gpui-component-0.7.3/src/dialog/dialog.rs`<br>`vendor/gpui-component-0.7.3/src/lib.rs`<br>`vendor/gpui-component-0.7.3/src/root.rs`<br>`vendor/gpui-component-0.7.3/src/window_ext.rs` | REBASE: vendor/gpui-component-0.7.3/src/dialog/dialog.rs; vendor/gpui-component-0.7.3/src/root.rs |
| `gpui-component-tree-accessible-name-v1.patch` | `vendor/gpui-base-0.7.3/src/tree.rs`<br>`vendor/gpui-component-0.7.3/src/tree.rs` | REBASE: vendor/gpui-component-0.7.3/src/tree.rs |
| `gpui-component-web-syntax-runtime-v1.patch` | `vendor/gpui-component-0.7.3/src/highlighter/registry.rs`<br>`vendor/gpui-component-0.7.3/src/highlighter/highlighter.rs`<br>`vendor/gpui-component-0.7.3/src/diff/state.rs` | REBASE: vendor/gpui-component-0.7.3/src/diff/state.rs |
| `gpui-component-web-syntax-runtime-v2.patch` | `vendor/gpui-component-0.7.3/src/highlighter/registry.rs`<br>`vendor/gpui-component-0.7.3/src/highlighter/highlighter.rs`<br>`vendor/gpui-component-0.7.3/src/diff/state.rs` | REBASE: vendor/gpui-component-0.7.3/src/diff/state.rs |
| `gpui-component-web-syntax-runtime-v3.patch` | `vendor/gpui-component-0.7.3/src/highlighter/registry.rs`<br>`vendor/gpui-component-0.7.3/src/highlighter/highlighter.rs`<br>`vendor/gpui-component-0.7.3/src/diff/state.rs`<br>`vendor/gpui-component-0.7.3/src/input/syntax_context.rs` | REBASE: vendor/gpui-component-0.7.3/src/diff/state.rs |
| `gpui-component-web-syntax-v1.patch` | `vendor/gpui-component-0.7.3/Cargo.toml` | APPLIES: — |
| `gpui-input-placeholder-color-v1.patch` | `vendor/gpui-base-0.7.3/src/input/editor/highlighting.rs`<br>`vendor/gpui-base-0.7.3/src/input/base/element.rs`<br>`vendor/gpui-component-0.7.3/src/input/input.rs` | APPLIES: — |
| `gpui-macos-anchored-popup-v1.patch` | `vendor/gpui-pre-macos-0.3.9/src/platform.rs`<br>`vendor/gpui-pre-macos-0.3.9/src/window.rs` | REBASE: vendor/gpui-pre-macos-0.3.9/src/platform.rs; vendor/gpui-pre-macos-0.3.9/src/window.rs |
| `gpui-macos-anchored-popup-v2.patch` | `vendor/gpui-pre-macos-0.3.9/src/platform.rs`<br>`vendor/gpui-pre-macos-0.3.9/src/window.rs` | REBASE: vendor/gpui-pre-macos-0.3.9/src/platform.rs; vendor/gpui-pre-macos-0.3.9/src/window.rs |
| `gpui-macos-anchored-popup-v3.patch` | `vendor/gpui-pre-macos-0.3.9/src/platform.rs`<br>`vendor/gpui-pre-macos-0.3.9/src/window.rs` | REBASE: vendor/gpui-pre-macos-0.3.9/src/platform.rs; vendor/gpui-pre-macos-0.3.9/src/window.rs |
| `gpui-macos-anchored-popup-v4.patch` | `vendor/gpui-pre-macos-0.3.9/src/platform.rs`<br>`vendor/gpui-pre-macos-0.3.9/src/window.rs` | REBASE: vendor/gpui-pre-macos-0.3.9/src/platform.rs; vendor/gpui-pre-macos-0.3.9/src/window.rs |
| `gpui-macos-focused-element-forwarder-v1.patch` | `vendor/gpui-pre-macos-0.3.9/src/window.rs` | REBASE: vendor/gpui-pre-macos-0.3.9/src/window.rs |
| `gpui-pre-test-a11y-activation-v1.patch` | `vendor/gpui-pre-0.3.9/src/platform/test/window.rs` | APPLIES: — |
| `gpui-pre-web-accessible-name-v1.patch` | `vendor/gpui-pre-0.3.9/src/window.rs`<br>`vendor/gpui-pre-web-0.3.9/src/window.rs`<br>`vendor/gpui-pre-web-0.3.9/src/ime_mirror.rs` | APPLIES: — |
| `gpui-pre-web-enter-composition-v1.patch` | `vendor/gpui-pre-web-0.3.9/src/events.rs` | REBASE: vendor/gpui-pre-web-0.3.9/src/events.rs |
| `gpui-pre-web-keyboard-focus-activation-v3.patch` | `vendor/gpui-pre-web-0.3.9/src/window.rs`<br>`vendor/gpui-pre-web-0.3.9/src/events.rs` | APPLIES: — |
| `gpui-pre-web-noopener-v1.patch` | `vendor/gpui-pre-web-0.3.9/src/platform.rs` | APPLIES: — |
| `gpui-primary-mousedown-click-v2.patch` | `vendor/gpui-pre-0.3.9/src/elements/div.rs` | APPLIES: — |
| `gpui-selectable-list-semantics-v2.patch` | `vendor/gpui-component-0.7.3/src/list/list.rs` | APPLIES: — |
| `gpui-toolbar-rtl-v1.patch` | `vendor/gpui-base-0.7.3/src/toolbar.rs`<br>`vendor/gpui-component-0.7.3/src/toolbar.rs` | REBASE: vendor/gpui-base-0.7.3/src/toolbar.rs |
| `gpui-tree-aria-disabled-v1.patch` | `vendor/gpui-pre-0.3.9/src/elements/div.rs`<br>`vendor/gpui-base-0.7.3/src/tree.rs`<br>`vendor/gpui-base-0.7.3/src/switch.rs` | REBASE: vendor/gpui-base-0.7.3/src/switch.rs |
| `gpui-tree-aria-level-v1.patch` | `vendor/gpui-base-0.7.3/src/tree.rs` | REBASE: vendor/gpui-base-0.7.3/src/tree.rs |
| `gpui-tree-disabled-keyboard-v2.patch` | `vendor/gpui-base-0.7.3/src/tree.rs` | APPLIES: — |
| `gpui-web-combobox-enter-v1.patch` | `vendor/gpui-pre-web-0.3.9/src/a11y_dom.js` | REBASE: vendor/gpui-pre-web-0.3.9/src/a11y_dom.js |
| `gpui-web-semantic-adapter-v9.patch` | `vendor/gpui-base-0.7.3/src/input/base/state.rs`<br>`vendor/gpui-component-0.7.3/src/input/input.rs`<br>`vendor/gpui-component-0.7.3/src/input/state.rs`<br>`vendor/gpui-pre-0.3.9/src/input.rs`<br>`vendor/gpui-pre-0.3.9/src/platform.rs`<br>`vendor/gpui-pre-0.3.9/src/window.rs`<br>`vendor/gpui-pre-0.3.9/src/window/a11y.rs`<br>`vendor/gpui-pre-web-0.3.9/src/a11y.rs`<br>`vendor/gpui-pre-web-0.3.9/src/a11y_dom.js`<br>`vendor/gpui-pre-web-0.3.9/src/a11y_dom.rs`<br>`vendor/gpui-pre-web-0.3.9/src/events.rs`<br>`vendor/gpui-pre-web-0.3.9/src/gpui_web.rs`<br>`vendor/gpui-pre-web-0.3.9/src/ime_edit.rs`<br>`vendor/gpui-pre-web-0.3.9/src/ime_mirror.rs`<br>`vendor/gpui-pre-web-0.3.9/src/window.rs` | REBASE: vendor/gpui-base-0.7.3/src/input/base/state.rs; vendor/gpui-pre-0.3.9/src/platform.rs; vendor/gpui-pre-0.3.9/src/window.rs; vendor/gpui-pre-web-0.3.9/src/events.rs; vendor/gpui-pre-web-0.3.9/src/ime_mirror.rs; vendor/gpui-pre-web-0.3.9/src/window.rs |
| `r15-breadcrumb-disabled-v1.patch` | `vendor/gpui-component-0.7.3/src/breadcrumb.rs` | APPLIES: — |
| `r15-button-disabled-focus-v1.patch` | `vendor/gpui-component-0.7.3/src/button/button.rs` | REBASE: vendor/gpui-component-0.7.3/src/button/button.rs |
| `r15-sidebar-disabled-v1.patch` | `vendor/gpui-component-0.7.3/src/sidebar/menu.rs` | APPLIES: — |
| `r15-test-tree-disabled-v1.patch` | `vendor/gpui-pre-0.3.9/src/window/a11y/debug.rs` | REBASE: vendor/gpui-pre-0.3.9/src/window/a11y/debug.rs |
| `r15-tree-disabled-focus-v1.patch` | `vendor/gpui-base-0.7.3/src/tree.rs` | APPLIES: — |
| `r15b-dropdown-menu-refresh-v1.patch` | `vendor/gpui-component-0.7.3/src/menu/dropdown_menu.rs` | APPLIES: — |
| `r15b-menu-item-disabled-v1.patch` | `vendor/gpui-component-0.7.3/src/menu/menu_item.rs` | APPLIES: — |
| `r15b-popup-menu-recovery-v1.patch` | `vendor/gpui-component-0.7.3/src/menu/popup_menu.rs` | APPLIES: — |
| `t13-modal-focus-recovery-v1.patch` | `vendor/gpui-component-0.7.3/src/root.rs`<br>`vendor/gpui-pre-0.3.9/src/key_dispatch.rs`<br>`vendor/gpui-pre-0.3.9/src/tab_stop.rs`<br>`vendor/gpui-pre-0.3.9/src/window.rs` | REBASE: vendor/gpui-component-0.7.3/src/root.rs |
| `t17-inert-focus-scope-v1.patch` | `vendor/gpui-pre-0.3.9/src/tab_stop.rs`<br>`vendor/gpui-pre-0.3.9/src/window.rs` | REBASE: vendor/gpui-pre-0.3.9/src/tab_stop.rs; vendor/gpui-pre-0.3.9/src/window.rs |

Counts across 71 approved `.patch` files: **OBSOLETE 0 / APPLIES 29 / REBASE 40 / UNKNOWN 2**.

Anchored popup check: downloaded `gpui-pre-macos-0.3.10/src/platform.rs` still imports `PopupNotSupportedError` and returns it when `WindowKind::AnchoredPopup(_)` is passed (lines 751–752); `src/window.rs` groups `AnchoredPopup` with unsupported window kinds. Therefore **no**, target gpui-pre does not support macOS anchored popups, and the v0.8.1 GPUI Fast platform crate contains no `AnchoredPopup` implementation. Patches `gpui-macos-anchored-popup-v1..v4` are not obsolete. Proposed `gpui-macos-anchored-popup-v5-coordinate-fix` is likewise not upstream-fixed; it needs a fresh preimage/rebase against 0.3.10.



### Intent review (behavioral reclassification)

The matrix above records whether each patch text applies to the target source. It does not establish that the behavior is still missing. This second pass checks each patch purpose against the GPUI Kit v0.8.0 and v0.8.1 release-note PR lists and the corresponding upstream source diff from Kit 0.7.3 / gpui-pre 0.3.9 to Kit 0.8.1 / gpui-pre 0.3.10. Source archives and the tag checkout are disposable under `/tmp/a2-intent-sources` and `/tmp/a2-plan-gpui-kit-v0.8.1`; repository vendor, owned mirrors, patches, and lockfiles were not changed.

Commands: `git -C /tmp/a2-plan-gpui-kit-v0.8.1 show v0.8.1:Cargo.toml`; `curl -L https://crates.io/api/v1/crates/<crate>/<version>/download` for exact old/target archives; Python `pathlib.Path.read_bytes()` comparisons for every patch target path between the old and target source directories (including the vendor counterpart for owned-mirror hunks); and `git -C /tmp/a2-intent-thirdparty apply --check <repo>/patches/approved/<patch>.patch` for pristine `block 0.1.6` and `cosmic-text 0.19.0`. Output: 71 patch rows, 83 GPUI target-file diffs, 42 byte-identical GPUI target files, and five GPUI target paths absent from 0.8.1; the two third-party source files were downloaded pristine and both `git apply --check` commands exited 0. The two UNKNOWN cases are the five relocated/removed paths, not an unavailable old source image. Release-note evidence: [v0.8.0](https://github.com/longbridge/gpui-kit/releases/tag/v0.8.0), [v0.8.1](https://github.com/longbridge/gpui-kit/releases/tag/v0.8.1).

| Patch | Purpose | Intent and upstream comparison |
|---|---|---|
| `a11y-validation-grid-v1.patch` | Expose input validation descriptions and editable-grid readonly/sort metadata. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-component-0.7.3/src/input/input.rs`. |
| `block-0.1.6-stack-symbol-v1.patch` | Repair the block runtime stack-symbol linkage. | **APPLIES** — Pristine crate source downloaded to `/tmp/a2-intent-thirdparty`; 0.1.6/0.19.0 source is unchanged for this upgrade and `git apply --check` passes. Existing GPUI-side linkage/shaping behavior is not supplied by v0.8.1. |
| `cosmic-text-initial-bidi-span-v1.patch` | Preserve initial bidirectional span geometry in text shaping. | **APPLIES** — Pristine crate source downloaded to `/tmp/a2-intent-thirdparty`; 0.1.6/0.19.0 source is unchanged for this upgrade and `git apply --check` passes. Existing GPUI-side linkage/shaping behavior is not supplied by v0.8.1. |
| `g10-labelled-by-tree-v1.patch` | Expose labelled-by IDs in test-window accessibility-tree output. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-pre-0.3.9/src/window/a11y/debug.rs; crates/rustui-gpui/src/window/a11y/debug.rs`. |
| `g10-parent-node-id-v1.patch` | Expose the existing AccessKit parent node ID to accessibility callbacks. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `gpui-base-button-disabled-accessibility-v1.patch` | Report disabled state on the Base Button accessibility node. | **OBSOLETE** — [#3531](https://github.com/longbridge/gpui-kit/pull/3531): `gpui-base/src/button.rs` now applies `A11yDisabledExt::a11y_disabled(disabled)` to the button node. RustUI regression required: `components::disabled_tests::button::accessibility`. |
| `gpui-base-calendar-date-limits-v1.patch` | Clamp calendar month/year calculations to representable date limits. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-base-0.7.3/src/calendar.rs`. |
| `gpui-base-disabled-semantic-roots-v1.patch` | Report disabled state on Select, DatePicker, and Input semantic roots. | **OBSOLETE** — [#3531](https://github.com/longbridge/gpui-kit/pull/3531): `select.rs`, `date_picker.rs`, and input semantic roots now apply `a11y_disabled(disabled)`; the patch’s three target roots are covered. RustUI regression required: `components::disabled_tests::combo_box::accessibility`, `components::disabled_tests::date_picker::accessibility`, and `components::input::disabled_input_accessibility_node_is_marked_disabled`. |
| `gpui-base-focus-trap-accessibility-v1.patch` | Forward accessibility role, properties, and synthetic children through FocusTrap. | **OBSOLETE** — [#3443](https://github.com/longbridge/gpui-kit/pull/3443): `gpui-base/src/focus_trap.rs` forwards the wrapped element role and accessibility properties. RustUI regression required: `components::overlay::focus_recovery_tests::focus_trap_forwards_dialog_role_and_label_in_open_dialog` and `...::focus_trap_forwards_dialog_role_and_label_in_open_sheet`. |
| `gpui-base-resize-first-move-v1.patch` | Apply pointer position immediately when resizing begins. | **PARTIAL** — [#3629](https://github.com/longbridge/gpui-kit/pull/3629): Target `gpui-base/src/resizable/panel.rs` reads `resizing_panel_ix` inside mouse callbacks; the proposal still adds immediate initial-pointer sizing. |
| `gpui-base-resize-first-move-v2.patch` | Apply pointer position immediately and read active resize state in mouse handlers. | **PARTIAL** — [#3629](https://github.com/longbridge/gpui-kit/pull/3629): Target `gpui-base/src/resizable/panel.rs` reads `resizing_panel_ix` inside mouse callbacks; the proposal still adds immediate initial-pointer sizing. |
| `gpui-base-resize-first-move-v3.patch` | Apply pointer position immediately and finish resize on left-button release. | **PARTIAL** — [#3629](https://github.com/longbridge/gpui-kit/pull/3629): Target `gpui-base/src/resizable/panel.rs` reads `resizing_panel_ix` inside mouse callbacks; immediate initial-pointer sizing and the explicit left-button release behavior remain. |
| `gpui-base-slider-endpoint-v1.patch` | Clamp snapped slider values to configured minimum and maximum. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-base-0.7.3/src/slider.rs`. |
| `gpui-base-slider-first-move-v1.patch` | Apply pointer position immediately when slider or thumb dragging begins. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `gpui-base-slider-set-value-v2.patch` | Handle slider SetValue, Increment, and Decrement accessibility actions consistently. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-base-0.7.3/src/slider.rs`. |
| `gpui-base-slider-snap-position-v1.patch` | Recompute thumb geometry after slider range or value updates. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `gpui-base-slider-zero-track-v1.patch` | Ignore invalid or zero-sized slider track geometry safely. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `gpui-bidi-input-hit-v1.patch` | Use shaped bidirectional clusters for caret and pointer hit testing. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-pre-0.3.9/src/text_system/line_layout.rs`. |
| `gpui-bidi-wrapped-rows-v1.patch` | Use row-shaped geometry for wrapped bidirectional text painting and positions. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-pre-0.3.9/src/platform.rs`. |
| `gpui-calendar-keyboard-v1.patch` | Add a separate uncommitted keyboard cursor to calendar navigation. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `gpui-calendar-multi-month-heading-v1.patch` | Position month headings correctly for multi-month calendars. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-base-0.7.3/src/calendar.rs`. |
| `gpui-calendar-rtl-v1.patch` | Order calendar cells, navigation, and date-picker layout for RTL. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-base-0.7.3/src/calendar.rs; vendor/gpui-component-0.7.3/src/time/date_picker.rs`. |
| `gpui-component-calendar-fr-ar-v1.patch` | Add French and Arabic calendar message catalog entries. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-component-0.7.3/locales/ui.yml`. |
| `gpui-component-date-picker-accessibility-v1.patch` | Allow a supplied accessible name on the DatePicker control. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `gpui-component-date-picker-localization-v2.patch` | Resolve DatePicker labels and month text from RustUI localization. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `gpui-component-date-picker-rtl-alignment-v1.patch` | Align DatePicker popup placement and content for RTL. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-component-0.7.3/src/time/date_picker.rs`. |
| `gpui-component-popup-menu-disabled-confirm-v1.patch` | Prevent disabled popup-menu selections from invoking actions or dismissing. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-component-0.7.3/src/menu/popup_menu.rs`. |
| `gpui-component-select-committed-title-v1.patch` | Keep the Select trigger title on the committed selection while filtering. | **OBSOLETE** — [#3446](https://github.com/longbridge/gpui-kit/pull/3446): `gpui-component/src/select.rs::accessibility_value` and `display_title` read `state.selection.first()` (the committed selection), not the transient list cursor. RustUI regression required: `picker_trigger_uses_committed_selection_while_filter_cursor_moves` (add/retain in the Picker component ticket). |
| `gpui-component-select-popup-rtl-v1.patch` | Apply RTL order and alignment to Select popup rows. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-component-0.7.3/src/list/list.rs`. |
| `gpui-component-select-rtl-alignment-v1.patch` | Align the Select popup to the logical start of an RTL trigger. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `gpui-component-select-rtl-alignment-v2.patch` | Correct Select popup RTL alignment after the first alignment revision. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `gpui-component-select-rtl-alignment-v3.patch` | Correct Select popup RTL alignment after the second alignment revision. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `gpui-component-select-rtl-auto-width-v4.patch` | Measure all option labels to fit an automatic Select popup width. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `gpui-component-select-rtl-auto-width-v6.patch` | Cache option-label measurements for automatic Select popup width. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `gpui-component-sheet-close-accessibility-v2.patch` | Expose a configurable accessible name for the Sheet close button. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `gpui-component-slider-accessibility-label-v2.patch` | Forward an optional accessible label to the component Slider node. | **OBSOLETE** — [#3494](https://github.com/longbridge/gpui-kit/pull/3494): `gpui-component/src/slider.rs` stores `accessibility_label` and forwards it to the Base Slider `aria_label`; #3524 is a settings-field label PR, not the slider change. RustUI regression required: `components::input::slider::rustui_slider_exposes_owner_accessibility_label`. |
| `gpui-component-tracked-dialog-focus-rebase-v2.patch` | Close a specific stacked dialog and restore its successor focus safely. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-component-0.7.3/src/dialog/dialog.rs; vendor/gpui-component-0.7.3/src/root.rs`. |
| `gpui-component-tree-accessible-name-v1.patch` | Expose an accessible name on the Tree root. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-component-0.7.3/src/tree.rs`. |
| `gpui-component-web-syntax-runtime-v1.patch` | Use the runtime syntax registry instead of a fixed syntax feature path. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-component-0.7.3/src/diff/state.rs`. |
| `gpui-component-web-syntax-runtime-v2.patch` | Refine runtime syntax registration for highlighter state. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-component-0.7.3/src/diff/state.rs`. |
| `gpui-component-web-syntax-runtime-v3.patch` | Route diff and input syntax selection through the runtime registry. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-component-0.7.3/src/diff/state.rs`. |
| `gpui-component-web-syntax-v1.patch` | Adjust syntax dependency features for the target package configuration. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `gpui-input-placeholder-color-v1.patch` | Apply configured placeholder color to input and editor placeholders. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `gpui-macos-anchored-popup-v1.patch` | Implement native macOS anchored-popup window creation. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-pre-macos-0.3.9/src/platform.rs; vendor/gpui-pre-macos-0.3.9/src/window.rs`. |
| `gpui-macos-anchored-popup-v2.patch` | Refine native macOS anchored-popup placement and lifecycle. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-pre-macos-0.3.9/src/platform.rs; vendor/gpui-pre-macos-0.3.9/src/window.rs`. |
| `gpui-macos-anchored-popup-v3.patch` | Refine native macOS anchored-popup focus and dismissal behavior. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-pre-macos-0.3.9/src/platform.rs; vendor/gpui-pre-macos-0.3.9/src/window.rs`. |
| `gpui-macos-anchored-popup-v4.patch` | Refine native macOS anchored-popup coordinate conversion. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-pre-macos-0.3.9/src/platform.rs; vendor/gpui-pre-macos-0.3.9/src/window.rs`. |
| `gpui-macos-focused-element-forwarder-v1.patch` | Forward focused-element information from the macOS platform window. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-pre-macos-0.3.9/src/window.rs`. |
| `gpui-pre-test-a11y-activation-v1.patch` | Enable accessibility tree construction in GPUI test windows. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `gpui-pre-web-accessible-name-v1.patch` | Expose accessible names through the platform accessibility bridge. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `gpui-pre-web-enter-composition-v1.patch` | Preserve Enter and composition event handling in the platform event bridge. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-pre-web-0.3.9/src/events.rs`. |
| `gpui-pre-web-keyboard-focus-activation-v3.patch` | Synchronize keyboard activation and focus state in the platform window. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `gpui-pre-web-noopener-v1.patch` | Use safe opener behavior when creating external browser windows. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `gpui-primary-mousedown-click-v2.patch` | Advertise Click for elements whose primary mouse-down listener activates them. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `gpui-selectable-list-semantics-v2.patch` | Expose listbox, option, and selected-state semantics for selectable lists. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `gpui-toolbar-rtl-v1.patch` | Mirror toolbar ordering and alignment for RTL layouts. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-base-0.7.3/src/toolbar.rs`. |
| `gpui-tree-aria-disabled-v1.patch` | Expose disabled state on Tree rows and preserve disabled semantics support. | **PARTIAL** — [#3531](https://github.com/longbridge/gpui-kit/pull/3531): Target adds the reusable disabled accessibility extension and covers Switch; `gpui-base/src/tree.rs` still does not mark the Tree row node disabled, so retain only that Tree-row delta. |
| `gpui-tree-aria-level-v1.patch` | Expose one-based aria-level values on Tree rows. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-base-0.7.3/src/tree.rs`. |
| `gpui-tree-disabled-keyboard-v2.patch` | Prevent disabled Tree folders from expanding through keyboard actions. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `gpui-web-combobox-enter-v1.patch` | Keep Enter behavior consistent for focused combobox options. | **UNKNOWN** — The target removed `src/a11y_dom.js`; behavior moved into the event/keyboard path and the same-node ownership contract was not established from the target diff. Resolve the new path before rebase/removal. |
| `gpui-web-semantic-adapter-v9.patch` | Forward semantic input roles, values, descriptions, and state through the platform adapter. | **UNKNOWN** — Four target files in the patch were removed or substantially relocated across GPUI core, Base, and platform sources; without mapping the new semantic adapter end to end, equivalence and remaining behavior cannot be decided. Resolve the relocated implementation before classification. |
| `r15-breadcrumb-disabled-v1.patch` | Preserve disabled Breadcrumb item semantics and interaction behavior. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `r15-button-disabled-focus-v1.patch` | Move focus onward when a focused Button becomes disabled. | **REBASE** — [#3606](https://github.com/longbridge/gpui-kit/pull/3606) preserves the focused Button binding in GPUI Base so forward/reverse Tab can leave it after disable. It does not move focus immediately. The Kit 0.8 candidate keeps that upstream Base implementation and adds only `window.focus_next(cx)` in the focused disabled Component Button render path. |
| `r15-sidebar-disabled-v1.patch` | Preserve disabled Sidebar item semantics and interaction behavior. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `r15-test-tree-disabled-v1.patch` | Expose disabled state and bounds in the GPUI test accessibility tree. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-pre-0.3.9/src/window/a11y/debug.rs`. |
| `r15-tree-disabled-focus-v1.patch` | Move focus past a Tree group when it becomes disabled. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `r15b-dropdown-menu-refresh-v1.patch` | Refresh live dropdown item descriptors while preserving menu state and focus. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `r15b-menu-item-disabled-v1.patch` | Expose disabled state on popup-menu item accessibility nodes. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `r15b-popup-menu-recovery-v1.patch` | Move selection to an enabled popup-menu item after live disable or rebuild. | **APPLIES** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared, and the patch applies cleanly to the v0.8.1/0.3.10 source image. |
| `t13-modal-focus-recovery-v1.patch` | Restore focus to the next modal or owner after modal removal. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-component-0.7.3/src/root.rs`. |
| `t17-inert-focus-scope-v1.patch` | Exclude inert focus scopes from keyboard traversal and restore traversal after changes. | **REBASE** — Corresponding target file diff was inspected; no upstream implementation of this behavior appeared. The syntactic matrix above records the first conflict: `vendor/gpui-pre-0.3.9/src/tab_stop.rs; vendor/gpui-pre-0.3.9/src/window.rs`. |

**Intent counts (71 patches): OBSOLETE 5 / PARTIAL 4 / APPLIES 30 / REBASE 30 / UNKNOWN 2.** `APPLIES` means still needed and cleanly applicable; `REBASE` means still needed but the target hunk conflicts; `PARTIAL` means upstream covers only a named subset; `OBSOLETE` means the target source implements the same behavior. The earlier syntactic counts (29 APPLIES / 40 REBASE / 2 UNKNOWN) remain in the original matrix as apply-check results only. Of the 42 rows in those two groups, 40 were resolved as still-needed REBASE and two relocated-source cases remain UNKNOWN; all earlier clean-apply rows were reviewed for duplicate behavior.

Candidate verification:

- `gpui-component-slider-accessibility-label-v2` is OBSOLETE via #3494, not #3524. #3524 names settings boolean fields after their setting title. The target Slider forwards its stored label to Base `aria_label`.
- `r15-button-disabled-focus-v1` is REBASE via #3606: GPUI Base retains the existing focused binding for Tab traversal; the candidate Component hunk adds immediate `focus_next` on disable. `gpui-base-button-disabled-accessibility-v1` and `gpui-base-disabled-semantic-roots-v1` are OBSOLETE via #3531.
- `gpui-base-focus-trap-accessibility-v1` is OBSOLETE via #3443. #3642 addresses repeated Tab-stop search termination, a separate behavior.
- `gpui-component-select-committed-title-v1` is OBSOLETE via #3446: the target trigger title reads the committed selection. Keep a RustUI regression for filtering without changing the displayed committed value.
- No patch that creates a cell-to-header relation is made obsolete by #3667. That PR adds DataTable roles and `TableDelegate::cell_label`; target code still has no `labelled_by` link from a RustUI grid cell to its header. #3758/#3773 postdate v0.8.1 and are outside this target comparison.
- Tree selection/toggling on click (#3567) does not implement disabled keyboard expansion or row disabled semantics. Popup/menu changes in #3307 do not guard disabled confirmation or add disabled state to popup-menu accessibility nodes.
- #3446 covers the committed Select selection used by the trigger title; the exact `selection.first()` implementation in the target is why the old patch is OBSOLETE.
- Resize patches are PARTIAL via #3629: the target reads active drag state during mouse callbacks, while immediate first-pointer sizing/release behavior remains absent. Tree disabled semantics are PARTIAL via #3531: Switch is covered; Tree rows are not.

Required RustUI regressions for retiring OBSOLETE patches (the Select trigger regression is proposed for the component ticket). Keep the Button recovery regression for the PARTIAL patch:

| Retired patch | RustUI test that must pass |
|---|---|
| `gpui-base-button-disabled-accessibility-v1.patch` | `components::disabled_tests::button::accessibility` |
| `gpui-base-disabled-semantic-roots-v1.patch` | `components::disabled_tests::combo_box::accessibility`, `components::disabled_tests::date_picker::accessibility`, and `components::input::disabled_input_accessibility_node_is_marked_disabled` |
| `gpui-base-focus-trap-accessibility-v1.patch` | `components::overlay::focus_recovery_tests::focus_trap_forwards_dialog_role_and_label_in_open_dialog` and `...::focus_trap_forwards_dialog_role_and_label_in_open_sheet` |
| `gpui-component-slider-accessibility-label-v2.patch` | `components::input::slider::rustui_slider_exposes_owner_accessibility_label` |
| `gpui-component-select-committed-title-v1.patch` | Add/retain `picker_trigger_uses_committed_selection_while_filter_cursor_moves` in the Picker component ticket. |

### Parked for later stages

These patches remain unchanged and active in `patches/approved/` during A2-1 through A2-6. A2-7 moves them unchanged, with their manifests, into `patches/parked/`.

| Patch | Reason |
|---|---|
| `gpui-pre-web-enter-composition-v1.patch` | Its only target is the deferred platform event adapter. |
| `gpui-pre-web-keyboard-focus-activation-v3.patch` | Its targets are confined to the deferred platform window and event adapter. |
| `gpui-pre-web-noopener-v1.patch` | Its only target is deferred-platform window creation. |
| `gpui-web-combobox-enter-v1.patch` | UNKNOWN in the intent review; its target source path was removed in 0.8.1 and belongs to the deferred platform adapter. |
| `gpui-web-semantic-adapter-v9.patch` | UNKNOWN in the intent review; its cross-crate adapter paths were reorganized, and the owner directed this unresolved platform patch to remain parked. |

## 4. RustUI API impact

Release notes v0.8.1 “Breaking Changes” and “Upstream GPUI compatibility” describe these changes ([release notes](https://github.com/longbridge/gpui-kit/releases/tag/v0.8.1)):

| Upstream change | Grep evidence under `crates/rustui/src` | Port note |
|---|---|---|
| `Modifiers::secondary()` / `secondary_key()` now take `&App` | 0 direct occurrences of either qualified API | Search full dependency wrappers too; not a direct RustUI callsite in this tree. |
| `Keystroke::parse` and `KeyBinding::new` key strings become static; OS-aware loading/parsing is needed for dynamic shortcut strings | 44 `Keystroke::parse(` occurrences in 21 RustUI source/test files; 25 `KeyBinding::new(` occurrences in 3 files | The public keyboard shortcut path parses caller-provided `chord` dynamically twice in `interaction/keyboard.rs`; this is the clearest production blocker. Literal/test parse calls can remain static. Audit each binding string before port. |
| `Keystroke::unparse` and `KeybindingKeystroke::unparse` now take `OperatingSystem` | 0 direct qualified occurrences | Check wrapper callsites after target compilation. |
| `PlatformInput::ScrollWheel` renamed to `Scroll`; scroll event gains `source` | 0 direct `PlatformInput::ScrollWheel` / `PlatformInput::Scroll(` occurrences; four `ScrollWheelEvent` test event constructions in `components/data/table.rs` | RustUI tests instantiate the event directly; inspect the target event struct for required `source` fields even without direct PlatformInput construction. |

Direct RustUI affected API callsite count by grep: **69 total** (44 parse + 25 bindings; no overlapping matches). The two dynamic chord parses are a subset of the 44. This is a grep count, includes unit tests, and should be reduced to an AST/compile-verified migration checklist during A2-4.

## 5. Tooling and consumer impact

- `scripts/verify_owned_gpui_sources.py`: update every source map path/version; add/remove mirror entries for new or retired GPUI packages; keep `Cargo.toml` as the only ignored file and preserve byte identity on all other files. The 0.8.x kit changes package names/manifest metadata in at least the base mirror (`rustui-gpui-base` is locally renamed); decide whether to retain that package rename or align it consistently before updating verifier expectations.
- Approved patch apply scripts, JSON manifests and script tests: target directories, exact pre/post hashes, and any package manifest paths must change to the 0.8.1/0.3.10 source images. Rebase patches in `patches/proposed/`, update manifest pre/post hashes from pristine and patched target trees, regenerate/verify mirror equivalence, then promote only after each stack test passes. Do not hand-edit owned or vendor files; apply scripts remain the only mechanism.
- `scripts/verify-release.sh` and patch inventory lists: update path/version-specific references and prove every promoted patch script/test is listed. `scripts/check.sh` also checks package/notices and GPUI source integrity; retain a passing old-source baseline until a coherent activation branch is ready.
- Notices: package versions and dependency closure change materially. Re-run the existing notice collection pipeline against the final integrated lock/source tree, inspect licenses for all additions, regenerate report/notice artifacts using its approved process, and verify every package resolves to a notice or explicit reviewed gap. Current GPUI Kit crates are Apache-2.0; do not infer the transitive tree is license-complete from upstream's lockfile.
- T11 Git consumer: `examples/external-consumer` depends only on RustUI and has no consumer `[patch]`; preserve that contract. Test a fresh Git consumer against the integration branch after owned path crates, source maps, package versions and bundled sources are coherent. The lock probe does not prove this consumer builds.
- Existing releases package notices and source files from the repository. Any changes in mirror/vendor pair count or dependency lock affect source download size and notice contents; re-run external-consumer, source verifier and notice tests before merge.

## 6. Risks and rollback

Before source migration, create and push an annotated archive tag at the verified `origin/main` commit (record the full commit in the ticket) so it permanently identifies the pre-upgrade tree. Do this only after checking the tag name is unused and the branch is clean. Keep the 0.7.3/0.3.9 source and approved patches available until all target patches, source verifier, workspace checks, release verifier, notices and external consumer pass. If any stage fails before activation, abandon the isolated target worktree; if the activation branch has begun, revert its single scoped activation commit or reset that disposable branch to the recorded archive base. Never rewrite `main`, overwrite an existing archive tag, or discard unrelated working tree state.

## 7. Proposed ticket split (each ≤2 hours; keep default CI green)

Prepare target files and patch rebases as inert proposals first. Do not replace the active dependency graph until a complete candidate passes on one integration branch; each preparation PR should affect only proposal tooling/data and leave current sources/build graph intact.

1. **A2-1 — Snapshot and rebase harness (≤2h):** pin the exact GPUI Kit v0.8.1 / gpui-pre 0.3.10 crate archives by SHA-256; add a temporary-fixture download/check command and an inventory of approved patch target paths with source hashes; add source-gate tests for a mismatched archive hash and a missing file. Current source remains selected; CI validates the harness. Deferred-platform patches are listed under “Parked for later stages” and remain unchanged until A2-7.
2. **A2-2 — Rebase remaining GPUI core/platform patches (≤2h):** rebase still-needed GPUI core, macOS, Apple/platform, text, input, window, accessibility, focus, and tab-stop patches into `patches/candidate-kit-0.8/`; copy cleanly applicable patches unchanged; record every target file pre/post hash; no activation. The deferred web adapter UNKNOWNs stay parked. Any unresolved target conflict is listed as an A2-2 remainder and split into A2-2b.
3. **A2-3 — Retire OBSOLETE patches (≤2h):** remove the five obsolete patch/manifests/scripts/tests from the candidate only after their listed RustUI regressions pass against v0.8.1; leave main’s current patch set intact until candidate activation.
4. **A2-4 — Component-ticket PARTIAL deltas (≤2h each):** carry the remaining resize, Tree-row, and immediate Button focus-recovery behavior into their component-family tickets; keep upstream-covered hunks out of the rebased patches. Then rebase remaining Base/Component patches by family with stack tests and manifests.
5. **A2-5 — RustUI API compatibility (≤2h):** migrate the 69 grep sites, especially dynamic shortcut parsing; add focused tests; test against an isolated 0.8.1 source graph. Split API families into further ≤2h tickets if needed.
6. **A2-6 — Owned mirrors, lock, notices, consumers (≤2h):** refresh owned mirrors via scripts, update verifier/manifests, resolve the full workspace lock only in the candidate, regenerate notices, and run fresh external consumer checks. Split by verifier/notices/consumer if any exceeds two hours.
7. **A2-7 — Activate and release gates (≤2h):** apply the complete approved patch stacks, switch manifests to the candidate, run `scripts/check.sh`, `./scripts/verify-release.sh source`, owned-source verification and external consumer; merge only if all pass. If the integrated port cannot fit this bound, split activation/gates and keep each pre-activation PR CI-green.

### A2-2 candidate status (inert; current sources remain selected)

The 0.8.1 candidate stack is in `patches/candidate-kit-0.8/`. `scripts/check_kit08_core_stack.py` downloads and verifies the A2-1 fixture, applies the listed stack in order, writes each file's pre/post SHA-256 to `core-manifest.json`, reverses the stack, and checks byte-for-byte restoration. No vendor, owned-mirror, dependency manifest, or lockfile source was activated.

Status totals: **copied 6 / rebased 6 / remainder 0**.

| Approved patch | Candidate status | Candidate file / note |
|---|---|---|
| `block-0.1.6-stack-symbol-v1.patch` | copied | `block-0.1.6-stack-symbol-v1.patch` |
| `cosmic-text-initial-bidi-span-v1.patch` | copied | `cosmic-text-initial-bidi-span-v1.patch` |
| `g10-parent-node-id-v1.patch` | copied | `g10-parent-node-id-v1.patch` |
| `g10-labelled-by-tree-v1.patch` | rebased | `g10-labelled-by-tree-v1-kit08.patch` |
| `gpui-bidi-wrapped-rows-v1.patch` | rebased | `gpui-bidi-wrapped-rows-v1-kit08.patch`; the 0.8.1 `Corners` import ordering was carried into the target context. |
| `gpui-bidi-input-hit-v1.patch` | rebased | `gpui-bidi-input-hit-v1-kit08.patch`; stacked after wrapped rows, which defines `WrappedGlyphCluster`. |
| `gpui-macos-focused-element-forwarder-v1.patch` | rebased | `gpui-macos-focused-element-forwarder-v1-kit08.patch` |
| `gpui-pre-test-a11y-activation-v1.patch` | copied | `gpui-pre-test-a11y-activation-v1.patch` |
| `gpui-pre-web-accessible-name-v1.patch` | copied | `gpui-pre-web-accessible-name-v1.patch` |
| `gpui-primary-mousedown-click-v2.patch` | copied | `gpui-primary-mousedown-click-v2.patch` |
| `r15-test-tree-disabled-v1.patch` | rebased | `r15-test-tree-disabled-v1-kit08.patch`; stacked after labelled-by output. |
| `t17-inert-focus-scope-v1.patch` | rebased | `t17-inert-focus-scope-v1-kit08.patch`; uses 0.3.10's `TabStopMap::next` / `prev` traversal so SumTree tab-index and group ordering remain authoritative while inert descendants are filtered. |

### A2-4a candidate status — Dialog / modal focus

The A2-4a stack applies after all 12 core candidates. `scripts/check_kit08_kit_stack.py` records the combined pre/post file hashes in `patches/candidate-kit-0.8/kit-manifest.json` and verifies exact reverse application. No source selection or dependency graph is activated.

Status: **copied 0 / rebased 2 / partial 0 / remainder 0**.

| Approved patch | Candidate file / rebase note |
|---|---|
| `t13-modal-focus-recovery-v1.patch` | `t13-modal-focus-recovery-v1-kit08.patch`; composed modal traversal with the 0.3.10 `matching_preserving_order` API from the preceding inert-focus candidate. |
| `gpui-component-tracked-dialog-focus-rebase-v2.patch` | `gpui-component-tracked-dialog-focus-rebase-v2-kit08.patch`; maps tracked close requests to the target's stable dialog IDs and exit lifecycle. |

Fixture stack proof passes. The t13 core behavior test passed in a disposable GPUI 0.3.10 test window (`cargo test -p a2-4a-modal-proof --test modal_scope`: 1 passed). A2-4-compat resolved the Base/Component source-graph errors; the tracked Component-dialog proof now passes in the patched fixture (`cargo test -p a2-4a-modal-proof --test tracked_dialog`: 1 passed).

### A2-4b result — Sheet accessibility

Copied the approved `gpui-component-sheet-close-accessibility-v2.patch` unchanged into the candidate stack (1 copied, 0 rebased, 0 partial, 0 remainder). Its test-window proof passed in the disposable 0.3.10/0.8.1 fixture: `sheet_close_accessibility_label_is_exposed_in_the_test_window_tree` (1 passed), asserting the accessibility tree exposes “Dismiss details.” The cumulative candidate stack applies and reverses exactly; the Base/Component compile gate is recorded by `scripts/check_kit08_kit_stack.py`.

### A2-4c0 result — core aria-disabled provider

Split the approved `gpui-tree-aria-disabled-v1.patch`: its GPUI-pre core hunks are candidate `gpui-tree-aria-disabled-v1-kit08.patch`, applied last in the 13-patch core stack. The patch applies to GPUI-pre 0.3.10 with its original hunk content. Its GPUI Base Tree-row delta remains for A2-4g; the Switch comment hunk is not carried because #3531 covers that behavior. The minimal disposable fixture test `aria_disabled_sets_disabled_on_the_accessibility_tree_node` passed (1 passed) and the debug accessibility tree reported `"role": "TreeItem"` with `"disabled": true`. The full candidate stack compiled Base and Component offline and reversed exactly.

### Candidate patch API dependency audit

Reviewed added GPUI methods and cross-patch calls across the remaining queued patches. The concrete stack dependencies are:

- `gpui-tree-aria-disabled-v1-kit08.patch` provides core `aria_disabled`; it now precedes A2-4c Breadcrumb/Sidebar, A2-4d MenuItem, and A2-4g Tree consumers.
- The A2-4g Base Tree-row `aria_disabled` call must precede `gpui-tree-aria-level-v1.patch`, whose hunk uses it as context. The family plan now orders it that way.
- `r15b-popup-menu-recovery-v1.patch` provides `rebuild_preserving_selection`, called by `r15b-dropdown-menu-refresh-v1.patch`; A2-4d now orders the provider first.
- `gpui-calendar-rtl-v1.patch` provides `CalendarState::right_to_left`, used by `gpui-calendar-multi-month-heading-v1.patch`; A2-4k now orders the provider first.
- `gpui-component-select-popup-rtl-v1.patch` provides Select RTL state/setters used by later alignment and auto-width patches; the A2-4e order already places it first.
- `gpui-bidi-wrapped-rows-v1-kit08.patch` provides row hit/position helpers used by `gpui-bidi-input-hit-v1-kit08.patch`; the core order already places it first.
- A2-4a `t13-modal-focus-recovery-v1-kit08.patch` provides modal focus APIs consumed by the tracked-dialog patch; its family order already places it first.
- The queued patch scan found no other cross-patch GPUI API dependencies. Calls to existing target APIs such as `aria_label`, `role`, and `update` do not require patch ordering.

### A2-4 compatibility check

The optional fixture compile gate is `python3 scripts/check_kit08_kit_stack.py --compile`. It applies all 12 core candidates and both A2-4a candidates, reconstructs the verified crate archives in a disposable workspace, overlays the applied patch outputs, runs `cargo check --offline -p gpui-base -p gpui-component`, then verifies exact reverse application. Use `--compile-online` if the Cargo registry/index or dependency archives are not cached. Crate source archives are downloaded and SHA-256 checked by the fixture harness. In this run an initial Cargo resolution needed an online refresh for `gpui-fast-platform ^0.1.12`; after fetching that package metadata and dependencies, the compile gate passed offline.

The first package checks found these errors:

| Check | Error | Cause and candidate fix |
|---|---|---|
| `gpui-base` | `src/input/base/tab_stops.rs:110`: `E0063`, missing `bidi_clusters` in `gpui::LineLayout` | The pristine `gpui-pre 0.3.10` `LineLayout` has no `bidi_clusters` field; `gpui-bidi-input-hit-v1-kit08.patch` adds it. GPUI Base 0.8.1's tab-stop output constructor and test helper therefore need that field. The candidate now shifts each bidi cluster's x by the same tab delta as its glyphs, initializes the test helper to `None`, and adds `bidi_clusters_move_with_expanded_tab_stops`. This preserves bidi hit-testing coordinates when tab stops widen a line. |
| `gpui-component` | `src/root.rs:48`: `E0774`, derive applied to a type alias | The t13 candidate placed `#[derive(Clone)]` on `ModalFocusMemory`, a type alias. Removed the derive; the alias is an `Rc` and its uses clone the `Rc` itself. |
| `gpui-component` | `src/dialog/dialog.rs:760`: `E0425`, `opening_id` not found | The tracked-dialog close closure did not capture the dialog's identity. The candidate now copies `self.opening_id` before creating the closure. |

After those patch-only fixes, `python3 scripts/check_kit08_kit_stack.py --compile --write-manifest` passed: both package checks passed offline; 31 fixture archives verified, 26 target files hashed, 29 stack files, reverse exact. The tracked-dialog GPUI test and modal-focus test each passed (1/1). A separate bidi-layout integration proof passed (1/1); running GPUI Base's unit-test target itself is blocked by upstream `gpui-fast-macos 0.1.12` use of unstable `Option::map_or_default` under Rust 1.97.1, so the new tab-stop unit test remains unexecuted here.

### Approved patch destination ledger (71 patches)

Every approved patch has exactly one destination. **Totals after A2-4g: CANDIDATE 35 / REMAINDER 0 / A2-4 QUEUE 22 / OBSOLETE 5 / PARKED 5 / POPUP STAGE 4 = 71.** `t13-modal-focus-recovery-v1.patch` is assigned to the **A2-4 QUEUE — Dialog / modal focus** despite its GPUI-pre focus hunks; `a11y-validation-grid-v1.patch` remains assigned to the **A2-4 QUEUE — Input / validation / DataGrid** despite its GPUI-pre accessibility-tree hunks. Their feature families were outside A2-2 core candidate scope.

### A2-4 families

The queue is split into isolated candidate stacks. Each ticket is estimated at no more than two hours. Patches in a row apply in the listed order; PARTIAL entries carry only the named delta left after the cited upstream change.

| Ticket / family | Patch list and stacking order | PARTIAL delta | Estimate |
|---|---|---|---|
| A2-4a — Dialog / modal focus | `t13-modal-focus-recovery-v1.patch` → `gpui-component-tracked-dialog-focus-rebase-v2.patch` | None | 2h |
| A2-4b — Sheet accessibility | `gpui-component-sheet-close-accessibility-v2.patch` | None | 1h |
| A2-4c0 — GPUI core aria-disabled dependency | GPUI-pre hunk split from `gpui-tree-aria-disabled-v1.patch` as `gpui-tree-aria-disabled-v1-kit08.patch` | Base Tree-row delta stays for A2-4g. | 1h |
| A2-4c — Button / sidebar / breadcrumb disabled behavior | `r15-button-disabled-focus-v1-kit08.patch` → `r15-sidebar-disabled-v1.patch` → `r15-breadcrumb-disabled-v1.patch` | Button: GPUI Base 0.8.1 includes #3606 focus-binding retention; the Component patch adds immediate focus recovery. Sidebar/Breadcrumb use the core `aria_disabled` API supplied by A2-4c0. | 1.5h |
| A2-4d — Menus / popup items | `gpui-component-popup-menu-disabled-confirm-v1.patch` → `r15b-menu-item-disabled-v1.patch` → `r15b-popup-menu-recovery-v1.patch` → `r15b-dropdown-menu-refresh-v1.patch` | Core `aria_disabled` is provided by A2-4c0; popup recovery provides `rebuild_preserving_selection` to dropdown refresh. | DONE: 1 rebased, 3 copied; `disabled_selected_popup_menu_item_ignores_confirm` passed on the patched fixture; compile gate and exact reverse passed. |
| A2-4e — Select / Picker popup layout | `gpui-component-select-popup-rtl-v1.patch` → `gpui-component-select-rtl-alignment-v1.patch` → `gpui-component-select-rtl-alignment-v2.patch` → `gpui-component-select-rtl-alignment-v3.patch` → `gpui-component-select-rtl-auto-width-v4.patch` → `gpui-component-select-rtl-auto-width-v6.patch` | None | DONE: 1 copied, 1 rebased, 4 incremental deltas; cumulative Base/Component compile and exact reverse passed; `rtl_select_popup_aligns_to_the_trigger_end` passed. |
| A2-4f — Selectable list semantics | `gpui-selectable-list-semantics-v2.patch` | None | DONE: copied unchanged (APPLIES); cumulative Base/Component compile gate and exact reverse passed. No patch-specific proof was required because this family had no rebased patch. |
| A2-4g — Tree | `gpui-component-tree-accessible-name-v1-kit08.patch` → `gpui-tree-aria-disabled-row-v1-kit08.patch` → `gpui-tree-aria-level-v1-kit08.patch` → `gpui-tree-disabled-keyboard-v2.patch` → `r15-tree-disabled-focus-v1.patch` | Core `aria_disabled` comes from A2-4c0; Tree-row delta must precede the aria-level patch, whose context uses that row call. #3531 covers Switch. | DONE: 2 copied, 2 rebased, 1 partial; fixture test `tree_accessibility_label_and_levels_reach_the_tree_nodes` passed (label, levels, disabled row); Base/Component compile and exact reverse passed. |
| A2-4h — DataGrid validation and header metadata | `a11y-validation-grid-v1.patch` | None | 2h |
| A2-4i — Slider | `gpui-base-slider-endpoint-v1.patch` → `gpui-base-slider-first-move-v1.patch` → `gpui-base-slider-set-value-v2.patch` → `gpui-base-slider-snap-position-v1.patch` → `gpui-base-slider-zero-track-v1.patch` | None | 2h |
| A2-4j — Resizable panels | `gpui-base-resize-first-move-v1.patch` → `gpui-base-resize-first-move-v2.patch` → `gpui-base-resize-first-move-v3.patch` | Keep immediate initial-pointer sizing; v3 also retains explicit left-button release handling. #3629 supplies active-drag-state reads only. | 2h |
| A2-4k — Calendar / DatePicker | `gpui-base-calendar-date-limits-v1.patch` → `gpui-calendar-keyboard-v1.patch` → `gpui-calendar-rtl-v1.patch` → `gpui-calendar-multi-month-heading-v1.patch` → `gpui-component-calendar-fr-ar-v1.patch` → `gpui-component-date-picker-accessibility-v1.patch` → `gpui-component-date-picker-localization-v2.patch` → `gpui-component-date-picker-rtl-alignment-v1.patch` | Calendar RTL provides the `CalendarState::right_to_left` API used by multi-month heading. | 2h |
| A2-4l — Input placeholder | `gpui-input-placeholder-color-v1.patch` | None | 1h |
| A2-4m — Toolbar RTL | `gpui-toolbar-rtl-v1.patch` | None | 1h |
| A2-4n — Diff / syntax registry | `gpui-component-web-syntax-v1.patch` → `gpui-component-web-syntax-runtime-v1.patch` → `gpui-component-web-syntax-runtime-v2.patch` → `gpui-component-web-syntax-runtime-v3.patch` | None | 2h |

Original queue total: **45 patches**. A2-4a is first because its tracked dialog patch stacks on the modal focus recovery change. A2-4a includes only the two dialog/focus patches; Sheet is separated into A2-4b because its patch touches an independent file and has no stack dependency.

| Approved patch | Sole destination |
|---|---|
| `a11y-validation-grid-v1.patch` | A2-4 QUEUE — Input / validation / DataGrid |
| `block-0.1.6-stack-symbol-v1.patch` | CANDIDATE — `patches/candidate-kit-0.8/block-0.1.6-stack-symbol-v1.patch` |
| `cosmic-text-initial-bidi-span-v1.patch` | CANDIDATE — `patches/candidate-kit-0.8/cosmic-text-initial-bidi-span-v1.patch` |
| `g10-labelled-by-tree-v1.patch` | CANDIDATE — `patches/candidate-kit-0.8/g10-labelled-by-tree-v1-kit08.patch` |
| `g10-parent-node-id-v1.patch` | CANDIDATE — `patches/candidate-kit-0.8/g10-parent-node-id-v1.patch` |
| `gpui-base-button-disabled-accessibility-v1.patch` | OBSOLETE — A2-3 |
| `gpui-base-calendar-date-limits-v1.patch` | A2-4 QUEUE — Calendar / DatePicker |
| `gpui-base-disabled-semantic-roots-v1.patch` | OBSOLETE — A2-3 |
| `gpui-base-focus-trap-accessibility-v1.patch` | OBSOLETE — A2-3 |
| `gpui-base-resize-first-move-v1.patch` | A2-4 QUEUE — Resizable panels |
| `gpui-base-resize-first-move-v2.patch` | A2-4 QUEUE — Resizable panels |
| `gpui-base-resize-first-move-v3.patch` | A2-4 QUEUE — Resizable panels |
| `gpui-base-slider-endpoint-v1.patch` | A2-4 QUEUE — Slider |
| `gpui-base-slider-first-move-v1.patch` | A2-4 QUEUE — Slider |
| `gpui-base-slider-set-value-v2.patch` | A2-4 QUEUE — Slider |
| `gpui-base-slider-snap-position-v1.patch` | A2-4 QUEUE — Slider |
| `gpui-base-slider-zero-track-v1.patch` | A2-4 QUEUE — Slider |
| `gpui-bidi-input-hit-v1.patch` | CANDIDATE — `patches/candidate-kit-0.8/gpui-bidi-input-hit-v1-kit08.patch` |
| `gpui-bidi-wrapped-rows-v1.patch` | CANDIDATE — `patches/candidate-kit-0.8/gpui-bidi-wrapped-rows-v1-kit08.patch` |
| `gpui-calendar-keyboard-v1.patch` | A2-4 QUEUE — Calendar / DatePicker |
| `gpui-calendar-multi-month-heading-v1.patch` | A2-4 QUEUE — Calendar / DatePicker |
| `gpui-calendar-rtl-v1.patch` | A2-4 QUEUE — Calendar / DatePicker |
| `gpui-component-calendar-fr-ar-v1.patch` | A2-4 QUEUE — Calendar / DatePicker |
| `gpui-component-date-picker-accessibility-v1.patch` | A2-4 QUEUE — Calendar / DatePicker |
| `gpui-component-date-picker-localization-v2.patch` | A2-4 QUEUE — Calendar / DatePicker |
| `gpui-component-date-picker-rtl-alignment-v1.patch` | A2-4 QUEUE — Calendar / DatePicker |
| `gpui-component-popup-menu-disabled-confirm-v1.patch` | CANDIDATE — `patches/candidate-kit-0.8/gpui-component-popup-menu-disabled-confirm-v1-kit08.patch` |
| `gpui-component-select-committed-title-v1.patch` | OBSOLETE — A2-3 |
| `gpui-component-select-popup-rtl-v1.patch` | CANDIDATE — `patches/candidate-kit-0.8/gpui-component-select-popup-rtl-v1-kit08.patch` |
| `gpui-component-select-rtl-alignment-v1.patch` | CANDIDATE — `patches/candidate-kit-0.8/gpui-component-select-rtl-alignment-v1.patch` |
| `gpui-component-select-rtl-alignment-v2.patch` | CANDIDATE — `patches/candidate-kit-0.8/gpui-component-select-rtl-alignment-v2-kit08.patch` |
| `gpui-component-select-rtl-alignment-v3.patch` | CANDIDATE — `patches/candidate-kit-0.8/gpui-component-select-rtl-alignment-v3-kit08.patch` |
| `gpui-component-select-rtl-auto-width-v4.patch` | CANDIDATE — `patches/candidate-kit-0.8/gpui-component-select-rtl-auto-width-v4-kit08.patch` |
| `gpui-component-select-rtl-auto-width-v6.patch` | CANDIDATE — `patches/candidate-kit-0.8/gpui-component-select-rtl-auto-width-v6-kit08.patch` |
| `gpui-component-sheet-close-accessibility-v2.patch` | CANDIDATE — `patches/candidate-kit-0.8/gpui-component-sheet-close-accessibility-v2.patch` |
| `gpui-component-slider-accessibility-label-v2.patch` | OBSOLETE — A2-3 |
| `gpui-component-tracked-dialog-focus-rebase-v2.patch` | CANDIDATE — `patches/candidate-kit-0.8/gpui-component-tracked-dialog-focus-rebase-v2-kit08.patch` |
| `gpui-component-tree-accessible-name-v1.patch` | CANDIDATE — `patches/candidate-kit-0.8/gpui-component-tree-accessible-name-v1-kit08.patch` |
| `gpui-component-web-syntax-runtime-v1.patch` | A2-4 QUEUE — Syntax highlighting / Diff |
| `gpui-component-web-syntax-runtime-v2.patch` | A2-4 QUEUE — Syntax highlighting / Diff |
| `gpui-component-web-syntax-runtime-v3.patch` | A2-4 QUEUE — Syntax highlighting / Diff |
| `gpui-component-web-syntax-v1.patch` | A2-4 QUEUE — Syntax highlighting / Diff |
| `gpui-input-placeholder-color-v1.patch` | A2-4 QUEUE — Input / validation / DataGrid |
| `gpui-macos-anchored-popup-v1.patch` | POPUP STAGE — native macOS popups |
| `gpui-macos-anchored-popup-v2.patch` | POPUP STAGE — native macOS popups |
| `gpui-macos-anchored-popup-v3.patch` | POPUP STAGE — native macOS popups |
| `gpui-macos-anchored-popup-v4.patch` | POPUP STAGE — native macOS popups |
| `gpui-macos-focused-element-forwarder-v1.patch` | CANDIDATE — `patches/candidate-kit-0.8/gpui-macos-focused-element-forwarder-v1-kit08.patch` |
| `gpui-pre-test-a11y-activation-v1.patch` | CANDIDATE — `patches/candidate-kit-0.8/gpui-pre-test-a11y-activation-v1.patch` |
| `gpui-pre-web-accessible-name-v1.patch` | CANDIDATE — `patches/candidate-kit-0.8/gpui-pre-web-accessible-name-v1.patch` |
| `gpui-pre-web-enter-composition-v1.patch` | PARKED — A2-7 (deferred platform) |
| `gpui-pre-web-keyboard-focus-activation-v3.patch` | PARKED — A2-7 (deferred platform) |
| `gpui-pre-web-noopener-v1.patch` | PARKED — A2-7 (deferred platform) |
| `gpui-primary-mousedown-click-v2.patch` | CANDIDATE — `patches/candidate-kit-0.8/gpui-primary-mousedown-click-v2.patch` |
| `gpui-selectable-list-semantics-v2.patch` | CANDIDATE — `patches/candidate-kit-0.8/gpui-selectable-list-semantics-v2.patch` |
| `gpui-toolbar-rtl-v1.patch` | A2-4 QUEUE — Toolbar |
| `gpui-tree-aria-disabled-v1.patch` | Split: CANDIDATE core API `gpui-tree-aria-disabled-v1-kit08.patch`; CANDIDATE Base Tree-row delta `gpui-tree-aria-disabled-row-v1-kit08.patch` |
| `gpui-tree-aria-level-v1.patch` | CANDIDATE — `patches/candidate-kit-0.8/gpui-tree-aria-level-v1-kit08.patch` |
| `gpui-tree-disabled-keyboard-v2.patch` | CANDIDATE — `patches/candidate-kit-0.8/gpui-tree-disabled-keyboard-v2.patch` |
| `gpui-web-combobox-enter-v1.patch` | PARKED — A2-7 (deferred platform) |
| `gpui-web-semantic-adapter-v9.patch` | PARKED — A2-7 (deferred platform) |
| `r15-breadcrumb-disabled-v1.patch` | CANDIDATE — `patches/candidate-kit-0.8/r15-breadcrumb-disabled-v1.patch` |
| `r15-button-disabled-focus-v1.patch` | CANDIDATE — `patches/candidate-kit-0.8/r15-button-disabled-focus-v1-kit08.patch` |
| `r15-sidebar-disabled-v1.patch` | CANDIDATE — `patches/candidate-kit-0.8/r15-sidebar-disabled-v1.patch` |
| `r15-test-tree-disabled-v1.patch` | CANDIDATE — `patches/candidate-kit-0.8/r15-test-tree-disabled-v1-kit08.patch` |
| `r15-tree-disabled-focus-v1.patch` | CANDIDATE — `patches/candidate-kit-0.8/r15-tree-disabled-focus-v1.patch` |
| `r15b-dropdown-menu-refresh-v1.patch` | CANDIDATE — `patches/candidate-kit-0.8/r15b-dropdown-menu-refresh-v1.patch` |
| `r15b-menu-item-disabled-v1.patch` | CANDIDATE — `patches/candidate-kit-0.8/r15b-menu-item-disabled-v1.patch` |
| `r15b-popup-menu-recovery-v1.patch` | CANDIDATE — `patches/candidate-kit-0.8/r15b-popup-menu-recovery-v1.patch` |
| `t13-modal-focus-recovery-v1.patch` | CANDIDATE — `patches/candidate-kit-0.8/t13-modal-focus-recovery-v1-kit08.patch` |
| `t17-inert-focus-scope-v1.patch` | CANDIDATE — `patches/candidate-kit-0.8/t17-inert-focus-scope-v1-kit08.patch` |

A2-2b rebased `t17-inert-focus-scope-v1.patch` against the 0.3.10 `TabStopMap` ordering and added a GPUI test-window regression. Inert filtering is used only while a rendered inert scope exists; otherwise `focus_next` and `focus_prev` retain their original direct `next` / `prev` calls. The test covers ordinary traversal with no inert scope, forward/backward traversal past nested inert descendants, direct focus rejection, and traversal restored after clearing inert state.

Anchored-popup v1–v4 remain excluded for the native-popups stage; the v5 proposal is unchanged. The five patches under “Parked for later stages” remain untouched. Mixed GPUI Base/Component accessibility and modal patches remain assigned to A2-4.

### A2-4d result — Menus / popup items

- Rebased `gpui-component-popup-menu-disabled-confirm-v1` against GPUI Component 0.8.1 and copied the three R15b menu patches unchanged, preserving provider-before-consumer ordering.
- `disabled_selected_popup_menu_item_ignores_confirm` passed in the disposable patched fixture; the full stack compiled offline locally and reverse application restored the fixture exactly. CI uses `--compile-online` because its Cargo cache lacks `aho-corasick` and the offline registry lookup fails.
- Remaining A2-4 QUEUE after A2-4d: 34 patches.

### A2-4e result — Select / Picker popup layout

- Restored the uncommitted candidate draft and fixed only the test import hunk in `gpui-component-select-rtl-auto-width-v6-kit08.patch` (`Entity`, `IntoElement`, and `Window`); behavior hunks stayed unchanged.
- The fixture proof `rtl_select_popup_aligns_to_the_trigger_end` passed. Base/Component compile and exact reverse passed with 13 core patches, 2 A2-4a, 1 A2-4b, 3 A2-4c, 4 A2-4d, and 6 A2-4e patches; 31 fixture archives verified, 37 target files hashed, 41 stack files.
- Remaining A2-4 QUEUE after A2-4e: 28 patches.

### A2-4c result — Button / Sidebar / Breadcrumb

- Reclassified `r15-button-disabled-focus-v1` from PARTIAL to REBASE after inspecting GPUI Kit #3606 in the pinned v0.8.1 fixture: GPUI Base preserves the focused binding/tab traversal; it does not move focus immediately. The candidate Component patch adds only the immediate `window.focus_next(cx)` recovery when the focused Button renders disabled.
- Copied the Sidebar and Breadcrumb patches unchanged. The `aria_disabled` core provider from A2-4c0 precedes both consumers.
- Focus proof passed in a disposable 0.3.10/0.8.1 workspace with all core and A2-4a/b/c patches applied: `disabling_a_focused_button_moves_focus_to_the_next_control` (1 passed).
- Stack check and offline Base/Component compile gate passed: 13 core, 2 A2-4a, 1 A2-4b, 3 A2-4c; 31 archives verified; 30 target files hashed; 34 stack files; exact reverse.
- Remaining A2-4 QUEUE after A2-4d: 34 patches.

### A2-4h stop diagnosis — DataGrid validation/header metadata

The candidate Base/Component compile gate passed, but the GPUI Component test-window proof `validation_description_and_grid_aria_metadata_reach_the_tree` failed at `sources/gpui-component-0.8.1/src/input/input.rs:878`: `assertion failed: nodes.values().any(|node| node["aria"]["description"] == "Validation error: required")`. In 0.8.1, `Input::aria_description` forwards through `InputBase` to the outer frame, whose role is `Presentational`; that frame does not surface the description on the native text-input accessibility node. The Input description path needs to attach the description to the editor-owned semantic node before this candidate can be accepted. The family stopped before testing the separate header/cell metadata assertion.
