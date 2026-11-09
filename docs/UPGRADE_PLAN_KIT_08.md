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

Source acquisition: `git clone --depth 1 --branch v0.8.1 https://github.com/longbridge/gpui-kit.git /tmp/a2-plan-gpui-kit-v0.8.1`; downloaded exact 0.8.1 / 0.3.10 published source archives into `/tmp/a2-plan-crates`. Disposable source tree `/tmp/a2-plan-patchcheck` placed those sources at the versioned paths used in patch headers; each approved patch was checked with `git apply --check <absolute-patch-path>` from that disposable tree. No patch was applied to the repository. `APPLIES` means all patch hunks passed that check against the downloaded target source; `REBASE` means check failed and names the first failing file(s). This is syntactic applicability, not a claim the behavior remains needed. `OBSOLETE` is used only with upstream evidence of the same behavior; no patch met that bar in this pass. Two unchanged third-party dependencies were already patched in the local source image, so their clean target preimages were unavailable and they remain `UNKNOWN`.

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

1. **A2-1 — Snapshot and rebase harness (≤2h):** download/pin exact v0.8.1/0.3.10 sources in a temporary fixture; add an inventory/check command or fixture test for target paths and hashes. Current source remains selected; CI validates the harness.
2. **A2-2 — Rebase GPUI core/platform patch family (≤2h):** rebase core, macOS, web, platform, text and input patch series into proposed patches; per-stack disposable apply scripts/tests/manifests; no activation. Split again if any series exceeds the timebox.
3. **A2-3 — Rebase GPUI Base patches (≤2h):** proposed patches for buttons, calendars, focus, resize, sliders, toolbar, tree and input; stack tests + manifests; no activation.
4. **A2-4 — Rebase GPUI Component patches (≤2h):** proposed patches for dialogs, menus, select/date, accessibility, syntax and RTL; stack tests + manifests; no activation.
5. **A2-5 — RustUI API compatibility (≤2h):** migrate the 69 grep sites, especially dynamic shortcut parsing; add focused tests; test against an isolated 0.8.1 source graph. Split API families into further ≤2h tickets if needed.
6. **A2-6 — Owned mirrors, lock, notices, consumers (≤2h):** refresh owned mirrors via scripts, update verifier/manifests, resolve the full workspace lock only in the candidate, regenerate notices, and run fresh external consumer checks. Split by verifier/notices/consumer if any exceeds two hours.
7. **A2-7 — Activate and release gates (≤2h):** apply the complete approved patch stacks, switch manifests to the candidate, run `scripts/check.sh`, `./scripts/verify-release.sh source`, owned-source verification and external consumer; merge only if all pass. If the integrated port cannot fit this bound, split activation/gates and keep each pre-activation PR CI-green.
