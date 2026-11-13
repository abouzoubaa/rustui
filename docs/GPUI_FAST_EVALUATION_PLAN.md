# GPUI Fast Evaluation Plan

## Scope and evidence

Plan only. No code, vendor, patch, or lockfile changes are part of F0 planning. The source reviewed was GPUI Kit v0.9.1 (tag commit `39ab849a93b5f15507b969164d49891db8d5b470`) and GPUI Fast v0.1.14 (tag commit `a427a7d9a6072473ad092b1673cd5235f99284cc`), downloaded to disposable directories on 2026-11-13. The Kit manifest pins gpui-pre 0.3.12 and optional gpui-fast 0.1.14. GPUI Fast publishes `gpui-fast` (library name `gpui`) and `gpui-fast-macros`; its core still depends on selected `gpui-pre-*` support crates at 0.3.12. Kit's `gpui-fast` feature wires the optional `gpui-fast` dependency through Base, Component and Kit Assets; Base and Component alias that dependency as `gpui` with `extern crate gpui_fast as gpui`. Rust imports through that alias stay spelled `gpui::…`. The GPUI Kit macro resolver explicitly checks package names `gpui-kit`, `gpui-pre`, and `gpui-fast` (`vendor/gpui-component-macros-0.9.1/src/crate_path.rs`).

This feature does not remove the non-optional `gpui-pre` dependency in the v0.9.1 Base and Component manifests. It therefore does not, by itself, prove a complete backend replacement for RustUI: the integration must check the resolved graph and prevent RustUI from mixing GPUI types from two crate identities. `gpui-fast` changes the `gpui` implementation; the `gpui-pre` support/platform crates remain in the graph where the Fast crate uses them. Do not equate enabling the Kit feature with eliminating every `gpui-pre` crate.

Sources: [GPUI Kit v0.9.1 release and manifest](https://github.com/longbridge/gpui-kit/releases/tag/v0.9.1), [Kit Cargo.toml v0.9.1](https://github.com/longbridge/gpui-kit/blob/v0.9.1/Cargo.toml), [GPUI Fast v0.1.14 release](https://github.com/longbridge/gpui-fast/releases/tag/v0.1.14), [GPUI Fast retained-mode guide](https://github.com/longbridge/gpui-fast/blob/v0.1.14/docs/retained-mode.md), [GPUI Fast scroll-layer guide](https://github.com/longbridge/gpui-fast/blob/v0.1.14/docs/scroll-layers.md). Local comparison sources: `target/headstart/a3-1-fixture/archives/` and `/tmp/f0-gpui-fast.hIb5hJ/source/`; neither is an activated workspace source.

## 1. Version and Rust requirement

| Component | Evaluated version | Notes |
|---|---:|---|
| GPUI Kit | 0.9.1 | Current RustUI base. |
| gpui-pre snapshot used by Kit | 0.3.12 | Still supplies support crates; Kit's standard backend remains available. |
| GPUI Fast | 0.1.14 | Optional Kit `gpui-fast` feature; release tag commit above. |
| Fast macro package | gpui-fast-macros 0.1.14 | Kit's macro resolver accepts the `gpui-fast` package name. |

GPUI Fast's published manifest has edition 2024 and no `rust-version`. The Fast source uses `is_multiple_of`, stabilized in Rust 1.87; the Kit 0.9.1 locked dependency graph has a declared maximum Rust version of 1.92 (`oo7` 0.6.0). These observations put the known floor below RustUI's pinned Rust 1.98.0, but they are not a substitute for compiling the exact F0 graph with 1.98.0. The 2024 edition itself requires Rust 1.85. [Rust 1.87 release notes](https://blog.rust-lang.org/2025/05/15/Rust-1.87.0/) document `is_multiple_of` stabilization.

## 2. Design changes and known limitations

GPUI Fast is a performance-focused fork, and its README calls it experimental. The relevant changes are:

- **Layout and retained views:** tracks view dependencies and retains clean view subtrees, layout nodes, hitboxes, dispatch nodes, primitives, and text measurements. Changed text is measured against cached constraints; only changed geometry should dirty ancestors.
- **Scroll layers:** caches scrolled content as GPU tiles and retains list rows so offset-only frames can move/replay cached output. The implementation uses the Metal renderer on macOS. Layer support is conditional; content that changes every frame still rebuilds, and accessibility-active virtual lists take a non-layer path.
- **Renderer:** adds tile/layer-aware rendering and dynamic texture paths; the ordinary GPUI renderer remains the fallback where a layer is unsupported.
- **Window/platform:** adds window composition APIs for native content with GPUI overlays. The README documents macOS AppKit/Metal support. This is a separate feature from native anchored popup windows and does not implement RustUI's popup behavior.
- **Text:** retained text measurements and Fast-specific text shaping/segmentation paths may change cost profiles; the Kit 0.9.1 profile identified ICU segmentation as a candidate cost already.

Known risks and evidence:

- **Stale frames:** any render input outside tracked entities, globals, list/scroll state, or tracked window state must trigger `cx.notify()`. Otherwise the retained view keeps the old output. The docs provide `GPUI_VIEW_RETENTION=0` as a diagnostic switch; it is not a correctness fix.
- **Invalidation cost:** notifying or writing state on every frame makes dependents rebuild. RustUI's render/prepaint paths need an audit for writes and external mutable state.
- **Recently fixed defects:** GPUI Fast history includes fixes for stale retained layout/rendering ([#46](https://github.com/longbridge/gpui-fast/pull/46), [#52](https://github.com/longbridge/gpui-fast/pull/52)), an unmeasured text panic ([#83](https://github.com/longbridge/gpui-fast/pull/83)), and renderer recovery after failed resize ([#93](https://github.com/longbridge/gpui-fast/pull/93)). The v0.1.14 release notes include avoiding redundant ancestor renders after child splices ([#91](https://github.com/longbridge/gpui-fast/pull/91)), forgetting released entity dependencies ([#106](https://github.com/longbridge/gpui-fast/pull/106)), and accessibility replay hierarchy ([#122](https://github.com/longbridge/gpui-fast/pull/122)). These are evidence of active edge cases and fixes, not evidence that RustUI has a remaining defect. The repository issues API returned no open non-PR issues on 2026-11-13: [open issues](https://api.github.com/repos/longbridge/gpui-fast/issues?state=open).
- **macOS coverage:** source and docs include AppKit and Metal paths, scroll-layer tests with headless Metal, and a native composition example. F0 still requires RustUI's native macOS checks and owner acceptance for visible stale rendering, resizing, text input, accessibility, popups, and window teardown.

## 3. Patch compatibility and intent inventory

### Active Kit 0.9.1 candidate stack against GPUI Fast

The active native/core and Kit-core-dependent set is the Kit 0.9.1 stack in `patches/approved/kit09/kit09-stack.json`, omitting only non-native-only rows from this native assessment. Mechanical checking used `git apply --check` in a disposable fixture assembled from the GPUI Fast v0.1.14 source and the Kit 0.9.1 crate archives, with the package path aliases expected by the current patch stack. Successful patches were applied in stack order; the four failures below were left unapplied. Result for the 52 native rows: **APPLIES 48, REBASE 4, UNKNOWN 0**. No patch was promoted or applied to the repository workspace.

Intent comparison asks whether GPUI Fast itself supplies the tested RustUI behavior, not whether a patch happens to apply. Fast's retention and scroll work does not replace these control, accessibility, focus, RTL, menu, date, or dialog behaviors. Intent results: **already covered 0, PARTIAL 3, still needed 49**. PARTIAL items retain only the still-missing RustUI behavior noted below. `UNKNOWN` in this section is zero for the active native stack; every listed intent was compared with the Fast source and v0.1.14 release notes.

| Candidate patch | Native target area | Mechanical | Intent |
|---|---|---:|---|
| `g10-parent-node-id-v1.patch` | `vendor/gpui-pre-0.3.12/src/window/a11y.rs` | REBASE | Still needed |
| `g10-labelled-by-tree-v1-kit08.patch` | `vendor/gpui-pre-0.3.12/src/window/a11y/debug.rs` | APPLIES | Still needed |
| `gpui-bidi-wrapped-rows-v1-kit08-kit09.patch` | `vendor/gpui-pre-0.3.12/src/platform.rs`, `vendor/gpui-pre-0.3.12/src/text_system/line.rs`, `vendor/gpui-pre-0.3.12/src/text_system/line_layout.rs`, `vendor/gpui-pre-wgpu-0.3.12/src/cosmic_text_system.rs` | APPLIES | Still needed |
| `gpui-bidi-input-hit-v1-kit08-kit09.patch` | `vendor/gpui-base-0.9.1/src/input/base/tab_stops.rs`, `vendor/gpui-base-0.9.1/src/text/inline.rs`, `vendor/gpui-pre-0.3.12/src/platform.rs`, `vendor/gpui-pre-0.3.12/src/text_system/line.rs`, `vendor/gpui-pre-0.3.12/src/text_system/line_layout.rs`, `vendor/gpui-pre-apple-0.3.12/src/text_system.rs`, `vendor/gpui-pre-wgpu-0.3.12/src/cosmic_text_system.rs`, `vendor/gpui-pre-windows-0.3.12/src/direct_write.rs` | APPLIES | Still needed |
| `gpui-macos-focused-element-forwarder-v1-kit08.patch` | `vendor/gpui-pre-macos-0.3.12/src/window.rs` | APPLIES | Still needed |
| `gpui-macos-map-or-default-v1-kit09.patch` | `vendor/gpui-pre-macos-0.3.12/src/window.rs` | APPLIES | Still needed |
| `gpui-pre-test-a11y-activation-v1.patch` | `vendor/gpui-pre-0.3.12/src/platform/test/window.rs` | APPLIES | Still needed |
| `gpui-primary-mousedown-click-v2.patch` | `vendor/gpui-pre-0.3.12/src/elements/div.rs` | REBASE | Still needed |
| `r15-test-tree-disabled-v1-kit08.patch` | `vendor/gpui-pre-0.3.12/src/window/a11y/debug.rs` | APPLIES | Still needed |
| `a11y-grid-readonly-sort-v1-kit09.patch` | `vendor/gpui-pre-0.3.12/src/elements/div.rs`, `vendor/gpui-pre-0.3.12/src/window/a11y/debug.rs` | APPLIES | Still needed |
| `t17-inert-focus-scope-v1-kit08-kit09.patch` | `vendor/gpui-pre-0.3.12/src/tab_stop.rs`, `vendor/gpui-pre-0.3.12/src/window.rs` | APPLIES | Still needed |
| `gpui-tree-aria-disabled-v1-kit08.patch` | `vendor/gpui-pre-0.3.12/src/elements/div.rs` | APPLIES | Still needed |
| `gpui-component-sidebar-disabled-a11y-v1-kit09.patch` | `vendor/gpui-component-0.9.1/src/sidebar/menu.rs` | APPLIES | Still needed |
| `t13-modal-focus-recovery-v1-kit09.patch` | `vendor/gpui-component-0.9.1/src/root.rs`, `vendor/gpui-pre-0.3.12/src/key_dispatch.rs`, `vendor/gpui-pre-0.3.12/src/window.rs` | APPLIES | Still needed |
| `gpui-component-tracked-dialog-focus-v1-kit09.patch` | `vendor/gpui-component-0.9.1/src/dialog/dialog.rs`, `vendor/gpui-component-0.9.1/src/lib.rs`, `vendor/gpui-component-0.9.1/src/root.rs`, `vendor/gpui-component-0.9.1/src/window_ext.rs` | APPLIES | Still needed |
| `gpui-component-sheet-close-accessibility-v1-kit09.patch` | `vendor/gpui-component-0.9.1/src/sheet.rs` | APPLIES | Still needed |
| `gpui-component-popup-menu-disabled-confirm-v1-kit08.patch` | `vendor/gpui-component-0.9.1/src/menu/popup_menu.rs` | APPLIES | Still needed |
| `r15b-menu-item-disabled-v1-kit09.patch` | `vendor/gpui-component-0.9.1/src/menu/menu_item.rs` | APPLIES | Still needed |
| `r15b-popup-menu-recovery-v1-kit09.patch` | `vendor/gpui-component-0.9.1/src/menu/popup_menu.rs` | APPLIES | Still needed |
| `r15b-dropdown-menu-refresh-v1.patch` | `vendor/gpui-component-0.9.1/src/menu/dropdown_menu.rs` | APPLIES | Still needed |
| `gpui-component-select-popup-rtl-v1-kit08.patch` | `vendor/gpui-component-0.9.1/src/list/list.rs`, `vendor/gpui-component-0.9.1/src/searchable_list/adapter.rs`, `vendor/gpui-component-0.9.1/src/searchable_list/item.rs` | APPLIES | Still needed |
| `gpui-component-select-rtl-alignment-v1.patch` | `vendor/gpui-component-0.9.1/src/select.rs` | APPLIES | Still needed |
| `gpui-component-select-rtl-alignment-v2-kit08.patch` | `vendor/gpui-component-0.9.1/src/select.rs` | APPLIES | Still needed |
| `gpui-component-select-rtl-alignment-v3-kit08.patch` | `vendor/gpui-component-0.9.1/src/select.rs` | APPLIES | Still needed |
| `gpui-component-select-rtl-auto-width-v4-kit08.patch` | `vendor/gpui-component-0.9.1/src/select.rs` | APPLIES | Still needed |
| `gpui-component-select-rtl-auto-width-v6-kit08.patch` | `vendor/gpui-component-0.9.1/src/select.rs` | APPLIES | Still needed |
| `gpui-selectable-list-semantics-v2.patch` | `vendor/gpui-component-0.9.1/src/list/list.rs` | APPLIES | Still needed |
| `gpui-component-tree-accessible-name-v1-kit08.patch` | `vendor/gpui-base-0.9.1/src/tree.rs`, `vendor/gpui-component-0.9.1/src/tree.rs` | APPLIES | Still needed |
| `gpui-tree-aria-disabled-row-v1-kit08.patch` | `vendor/gpui-base-0.9.1/src/tree.rs` | APPLIES | Still needed |
| `gpui-tree-aria-level-v1-kit08.patch` | `vendor/gpui-base-0.9.1/src/tree.rs` | APPLIES | Still needed |
| `gpui-tree-disabled-keyboard-v2.patch` | `vendor/gpui-base-0.9.1/src/tree.rs` | APPLIES | Still needed |
| `r15-tree-disabled-focus-v1.patch` | `vendor/gpui-base-0.9.1/src/tree.rs` | APPLIES | Still needed |
| `gpui-component-table-delegate-accessibility-v1-kit09.patch` | `vendor/gpui-component-0.9.1/src/table/state.rs`, `vendor/gpui-component-0.9.1/src/table/data_table.rs` | APPLIES | Still needed |
| `gpui-base-slider-endpoint-v1-kit09.patch` | `vendor/gpui-base-0.9.1/src/slider.rs`, `vendor/gpui-base-0.9.1/src/slider.rs` | APPLIES | PARTIAL — keep min/max endpoint clamping beyond Fast's rendering work |
| `gpui-base-slider-first-move-v1.patch` | `vendor/gpui-base-0.9.1/src/slider.rs` | APPLIES | Still needed |
| `gpui-base-slider-set-value-v2-kit09.patch` | `vendor/gpui-base-0.9.1/src/slider.rs` | APPLIES | PARTIAL — keep SetValue and RustUI range/step behavior beyond any upstream step action |
| `gpui-base-slider-zero-track-v1.patch` | `vendor/gpui-base-0.9.1/src/slider.rs` | APPLIES | Still needed |
| `gpui-base-resize-first-move-v1-kit08.patch` | `vendor/gpui-base-0.9.1/src/resizable/panel.rs` | APPLIES | Still needed |
| `gpui-base-resize-first-move-v3-kit08.patch` | `vendor/gpui-base-0.9.1/src/resizable/panel.rs` | APPLIES | Still needed |
| `gpui-base-calendar-date-limits-v1-kit08.patch` | `vendor/gpui-base-0.9.1/src/calendar.rs` | APPLIES | Still needed |
| `gpui-calendar-keyboard-v1.patch` | `vendor/gpui-base-0.9.1/src/calendar.rs`, `vendor/gpui-component-0.9.1/src/time/date_picker.rs` | APPLIES | Still needed |
| `gpui-calendar-rtl-v1-kit08.patch` | `vendor/gpui-base-0.9.1/src/calendar.rs`, `vendor/gpui-component-0.9.1/src/time/calendar.rs`, `vendor/gpui-component-0.9.1/src/time/date_picker.rs` | APPLIES | Still needed |
| `gpui-calendar-multi-month-heading-v1-kit08.patch` | `vendor/gpui-base-0.9.1/src/calendar.rs` | APPLIES | Still needed |
| `gpui-component-calendar-fr-ar-v1-kit09.patch` | `vendor/gpui-component-0.9.1/locales/ui.yml`, `vendor/gpui-component-0.9.1/src/time/calendar.rs` | APPLIES | Still needed |
| `gpui-component-date-picker-accessibility-v1.patch` | `vendor/gpui-component-0.9.1/src/time/date_picker.rs` | APPLIES | Still needed |
| `gpui-component-date-picker-localization-v2-kit08.patch` | `vendor/gpui-component-0.9.1/src/time/date_picker.rs` | APPLIES | Still needed |
| `gpui-component-date-picker-rtl-alignment-v1-kit08.patch` | `vendor/gpui-component-0.9.1/src/time/date_picker.rs` | APPLIES | Still needed |
| `gpui-base-input-validation-description-v1-kit09.patch` | `vendor/gpui-base-0.9.1/src/input/base/state.rs`, `vendor/gpui-component-0.9.1/src/input/input.rs` | APPLIES | PARTIAL — retain editor-owned-node semantics if Fast/Kit wrapper behavior does not supply them |
| `gpui-input-placeholder-color-v1-kit09.patch` | `vendor/gpui-base-0.9.1/src/input/editor/highlighting.rs`, `vendor/gpui-base-0.9.1/src/input/base/element.rs`, `vendor/gpui-component-0.9.1/src/input/input.rs` | APPLIES | Still needed |
| `gpui-toolbar-rtl-v1-kit09.patch` | `vendor/gpui-base-0.9.1/src/toolbar.rs`, `vendor/gpui-component-0.9.1/src/toolbar.rs` | APPLIES | Still needed |
| `gpui-macos-anchored-popup-v1-kit09.patch` | `vendor/gpui-pre-0.3.12/src/app/test_context.rs`, `vendor/gpui-pre-0.3.12/src/platform.rs`, `vendor/gpui-pre-0.3.12/src/platform/test/window.rs`, `vendor/gpui-pre-0.3.12/src/window.rs`, `vendor/gpui-pre-macos-0.3.12/src/platform.rs`, `vendor/gpui-pre-macos-0.3.12/src/window.rs` | REBASE | Still needed; Fast window composition is not an anchored-popup implementation |
| `gpui-macos-popup-child-teardown-v1-kit09.patch` | `vendor/gpui-pre-macos-0.3.12/src/window.rs` | REBASE | Still needed |

The four rebases are hunk mismatches against Fast's changed accessibility and macOS window structures. The A3 stack gate already runs the corresponding behavior proofs; F0 must rebase these patches with the same proof names before the backend can be considered. The three PARTIAL rows are not full retirements.

### Approved Kit 0.8 predecessor patches still present at the top level

The Kit 0.8 predecessor patches below remain approved on disk, so this native subset is included even though F0 must use the Kit 0.9.1 stack. The top-level predecessor stack also contains non-native entries outside this plan. Each predecessor was checked individually with `git apply --check` against a disposable fixture containing the GPUI Fast v0.1.14 core/platform/macOS/Apple source and Kit 0.9.1 Base/Component sources under the historical package path prefixes. This checks hunk compatibility only; it does not mean a patch is safe to stack or behaviorally proven. Result: **APPLIES 25 / REBASE 27 / UNKNOWN 0**. Successful patches were not applied to the repository. Map every row to its Kit 0.9 successor and retest the ordered cumulative stack before activation.

| Approved Kit 0.8 predecessor patch | Native target file(s) | Mechanical | Intent |
|---|---|---:|---|
| `g10-parent-node-id-v1.patch` | `vendor/gpui-pre-0.3.10/src/window/a11y.rs` | REBASE | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `g10-labelled-by-tree-v1-kit08.patch` | `vendor/gpui-pre-0.3.10/src/window/a11y/debug.rs` | APPLIES | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-bidi-wrapped-rows-v1-kit08.patch` | `vendor/gpui-pre-0.3.10/src/platform.rs`, `vendor/gpui-pre-0.3.10/src/text_system/line.rs`, `vendor/gpui-pre-0.3.10/src/text_system/line_layout.rs`, `vendor/gpui-pre-wgpu-0.3.10/src/cosmic_text_system.rs` | REBASE | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-bidi-input-hit-v1-kit08.patch` | `vendor/gpui-base-0.8.1/src/text/inline.rs`, `vendor/gpui-pre-0.3.10/src/platform.rs`, `vendor/gpui-pre-0.3.10/src/text_system/line.rs`, `vendor/gpui-pre-0.3.10/src/text_system/line_layout.rs`, `vendor/gpui-pre-apple-0.3.10/src/text_system.rs`, `vendor/gpui-pre-wgpu-0.3.10/src/cosmic_text_system.rs`, `vendor/gpui-pre-windows-0.3.10/src/direct_write.rs`, `vendor/gpui-base-0.8.1/src/input/base/tab_stops.rs`, `vendor/gpui-base-0.8.1/src/input/base/tab_stops.rs` | REBASE | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-macos-focused-element-forwarder-v1-kit08.patch` | `vendor/gpui-pre-macos-0.3.10/src/window.rs` | APPLIES | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-pre-test-a11y-activation-v1.patch` | `vendor/gpui-pre-0.3.10/src/platform/test/window.rs` | APPLIES | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-primary-mousedown-click-v2.patch` | `vendor/gpui-pre-0.3.10/src/elements/div.rs` | REBASE | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `r15-test-tree-disabled-v1-kit08.patch` | `vendor/gpui-pre-0.3.10/src/window/a11y/debug.rs` | REBASE | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `t17-inert-focus-scope-v1-kit08.patch` | `vendor/gpui-pre-0.3.10/src/tab_stop.rs`, `vendor/gpui-pre-0.3.10/src/window.rs` | REBASE | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-tree-aria-disabled-v1-kit08.patch` | `vendor/gpui-pre-0.3.10/src/elements/div.rs` | APPLIES | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `t13-modal-focus-recovery-v1-kit08.patch` | `vendor/gpui-pre-0.3.10/src/key_dispatch.rs`, `vendor/gpui-pre-0.3.10/src/window.rs`, `vendor/gpui-component-0.8.1/src/root.rs`, `vendor/gpui-pre-0.3.10/src/key_dispatch.rs`, `vendor/gpui-pre-0.3.10/src/window.rs` | REBASE | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-component-tracked-dialog-focus-rebase-v2-kit08.patch` | `vendor/gpui-component-0.8.1/src/root.rs`, `vendor/gpui-component-0.8.1/src/dialog/dialog.rs`, `vendor/gpui-component-0.8.1/src/window_ext.rs`, `vendor/gpui-component-0.8.1/src/lib.rs`, `vendor/gpui-component-0.8.1/src/root.rs`, `vendor/gpui-component-0.8.1/src/dialog/dialog.rs`, `vendor/gpui-component-0.8.1/src/window_ext.rs`, `vendor/gpui-component-0.8.1/src/dialog/dialog.rs` | REBASE | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-component-sheet-close-accessibility-v2.patch` | `vendor/gpui-component-0.8.1/src/sheet.rs` | REBASE | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `r15-button-disabled-focus-v1-kit08.patch` | `vendor/gpui-component-0.8.1/src/button/button.rs` | APPLIES | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `r15-sidebar-disabled-v1.patch` | `vendor/gpui-component-0.8.1/src/sidebar/menu.rs` | APPLIES | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `r15-breadcrumb-disabled-v1.patch` | `vendor/gpui-component-0.8.1/src/breadcrumb.rs` | APPLIES | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-component-popup-menu-disabled-confirm-v1-kit08.patch` | `vendor/gpui-component-0.8.1/src/menu/popup_menu.rs` | APPLIES | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `r15b-menu-item-disabled-v1.patch` | `vendor/gpui-component-0.8.1/src/menu/menu_item.rs` | REBASE | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `r15b-popup-menu-recovery-v1.patch` | `vendor/gpui-component-0.8.1/src/menu/popup_menu.rs` | REBASE | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `r15b-dropdown-menu-refresh-v1.patch` | `vendor/gpui-component-0.8.1/src/menu/dropdown_menu.rs` | APPLIES | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-component-select-popup-rtl-v1-kit08.patch` | `vendor/gpui-component-0.8.1/src/list/list.rs`, `vendor/gpui-component-0.8.1/src/searchable_list/adapter.rs`, `vendor/gpui-component-0.8.1/src/searchable_list/item.rs` | APPLIES | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-component-select-rtl-alignment-v1.patch` | `vendor/gpui-component-0.8.1/src/select.rs` | APPLIES | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-component-select-rtl-alignment-v2-kit08.patch` | `vendor/gpui-component-0.8.1/src/select.rs` | APPLIES | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-component-select-rtl-alignment-v3-kit08.patch` | `vendor/gpui-component-0.8.1/src/select.rs` | APPLIES | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-component-select-rtl-auto-width-v4-kit08.patch` | `vendor/gpui-component-0.8.1/src/select.rs` | REBASE | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-component-select-rtl-auto-width-v6-kit08.patch` | `vendor/gpui-component-0.8.1/src/select.rs` | REBASE | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-selectable-list-semantics-v2.patch` | `vendor/gpui-component-0.8.1/src/list/list.rs` | APPLIES | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-component-tree-accessible-name-v1-kit08.patch` | `vendor/gpui-base-0.8.1/src/tree.rs`, `vendor/gpui-component-0.8.1/src/tree.rs` | APPLIES | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-tree-aria-disabled-row-v1-kit08.patch` | `vendor/gpui-base-0.8.1/src/tree.rs` | APPLIES | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-tree-aria-level-v1-kit08.patch` | `vendor/gpui-base-0.8.1/src/tree.rs` | REBASE | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-tree-disabled-keyboard-v2.patch` | `vendor/gpui-base-0.8.1/src/tree.rs` | APPLIES | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `r15-tree-disabled-focus-v1.patch` | `vendor/gpui-base-0.8.1/src/tree.rs` | APPLIES | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `a11y-validation-grid-v1-kit08.patch` | `vendor/gpui-base-0.8.1/src/input/base/state.rs`, `vendor/gpui-component-0.8.1/src/input/input.rs`, `vendor/gpui-pre-0.3.10/src/elements/div.rs`, `vendor/gpui-pre-0.3.10/src/window/a11y/debug.rs`, `vendor/gpui-component-0.8.1/src/table/state.rs` | REBASE | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-base-slider-endpoint-v1-kit08.patch` | `vendor/gpui-base-0.8.1/src/slider.rs` | REBASE | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-base-slider-first-move-v1.patch` | `vendor/gpui-base-0.8.1/src/slider.rs` | APPLIES | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-base-slider-set-value-v2-kit08.patch` | `vendor/gpui-base-0.8.1/src/slider.rs` | REBASE | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-base-slider-snap-position-v1.patch` | `vendor/gpui-base-0.8.1/src/slider.rs` | REBASE | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-base-slider-zero-track-v1.patch` | `vendor/gpui-base-0.8.1/src/slider.rs` | APPLIES | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-base-resize-first-move-v1-kit08.patch` | `vendor/gpui-base-0.8.1/src/resizable/panel.rs` | APPLIES | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-base-resize-first-move-v3-kit08.patch` | `vendor/gpui-base-0.8.1/src/resizable/panel.rs` | APPLIES | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-base-calendar-date-limits-v1-kit08.patch` | `vendor/gpui-base-0.8.1/src/calendar.rs` | APPLIES | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-calendar-keyboard-v1.patch` | `vendor/gpui-base-0.8.1/src/calendar.rs`, `vendor/gpui-component-0.8.1/src/time/date_picker.rs` | APPLIES | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-calendar-rtl-v1-kit08.patch` | `vendor/gpui-base-0.8.1/src/calendar.rs`, `vendor/gpui-component-0.8.1/src/time/calendar.rs`, `vendor/gpui-component-0.8.1/src/time/date_picker.rs` | REBASE | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-calendar-multi-month-heading-v1-kit08.patch` | `vendor/gpui-base-0.8.1/src/calendar.rs` | REBASE | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-component-calendar-fr-ar-v1-kit08.patch` | `vendor/gpui-component-0.8.1/locales/ui.yml`, `vendor/gpui-component-0.8.1/src/time/calendar.rs` | REBASE | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-component-date-picker-accessibility-v1.patch` | `vendor/gpui-component-0.8.1/src/time/date_picker.rs` | APPLIES | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-component-date-picker-localization-v2-kit08.patch` | `vendor/gpui-component-0.8.1/src/time/date_picker.rs` | REBASE | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-component-date-picker-rtl-alignment-v1-kit08.patch` | `vendor/gpui-component-0.8.1/src/time/date_picker.rs` | REBASE | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-input-placeholder-color-v1.patch` | `vendor/gpui-base-0.8.1/src/input/editor/highlighting.rs`, `vendor/gpui-base-0.8.1/src/input/base/element.rs`, `vendor/gpui-component-0.8.1/src/input/input.rs` | REBASE | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-toolbar-rtl-v1-kit08.patch` | `vendor/gpui-base-0.8.1/src/toolbar.rs`, `vendor/gpui-component-0.8.1/src/toolbar.rs` | REBASE | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |
| `gpui-macos-popup-child-teardown-v1.patch` | `vendor/gpui-pre-macos-0.3.10/src/window.rs`, `crates/rustui-gpui-macos/src/window.rs` | REBASE | Still needed; current Kit 0.9 successor is listed in the active table. |
| `gpui-macos-anchored-popup-kit08-v1.patch` | `crates/rustui-gpui-macos/src/platform.rs`, `crates/rustui-gpui-macos/src/window.rs`, `crates/rustui-gpui/src/app/test_context.rs`, `crates/rustui-gpui/src/platform.rs`, `crates/rustui-gpui/src/platform/test/window.rs`, `crates/rustui-gpui/src/window.rs`, `vendor/gpui-pre-0.3.10/src/app/test_context.rs`, `vendor/gpui-pre-0.3.10/src/platform.rs`, `vendor/gpui-pre-0.3.10/src/platform/test/window.rs`, `vendor/gpui-pre-0.3.10/src/window.rs`, `vendor/gpui-pre-macos-0.3.10/src/platform.rs`, `vendor/gpui-pre-macos-0.3.10/src/window.rs` | REBASE | Still needed; use the Kit 0.9 candidate successor when one exists; verify a deliberate disposition otherwise. |

## 4. Owned-source and tooling impact

Current native owned mirrors checked by `scripts/verify_owned_gpui_sources.py` are:

| Owned path | Current mirrored source |
|---|---|
| `crates/rustui-gpui` | `vendor/gpui-pre-0.3.12` |
| `crates/rustui-gpui-platform` | `vendor/gpui-pre-platform-0.3.12` |
| `crates/rustui-gpui-macos` | `vendor/gpui-pre-macos-0.3.12` |
| `crates/rustui-gpui-apple` | `vendor/gpui-pre-apple-0.3.12` |
| `crates/rustui-gpui-base` | `vendor/gpui-base-0.9.1` |
| `crates/rustui-gpui-component` | `vendor/gpui-component-0.9.1` |
| `crates/rustui-gpui-kit-assets` | `vendor/gpui-kit-assets-0.9.1` |
| `crates/rustui-block` | `vendor/block-0.1.6` |
| `crates/rustui-cocoa`, `crates/rustui-cocoa-foundation`, `crates/rustui-metal`, `crates/rustui-objc-foundation` | corresponding vendored native support crates |

Fast's published core uses package `gpui-fast` with library crate name `gpui`; its macro package is `gpui-fast-macros`. A full switch needs a source-of-truth decision for Fast files modified by patches: likely add RustUI-owned mirrors for the Fast core and macros and keep/reconcile the existing owned platform/macOS/Apple mirrors where the Fast dependency graph still uses them. Do not hand-edit either source; introduce each change as a patch and apply it with a manifest-backed script. The `verify_owned_gpui_sources.py` mapping and its source tests must be extended if Fast copies are mirrored.

Reuse `prepare_gpui_kit_09_fixture.py`, `scripts/check_kit09_core_stack.py`, `scripts/check_kit09_family_stack.py`, their exact reverse manifests, and the cumulative family proof crate. Add a Fast source manifest and fixture extraction path; preserve the separate fixture target-directory rule so test packages never build into the workspace target. Extend proof logs to require actual passing test output, including the existing accessibility, disabled-control, async, theme, grid, popup, syntax, and lifecycle suites.

The current source cache key already hashes `patches/approved/*.json`, `patches/candidate-kit-0.9/**`, RustUI-owned source globs, and `vendor/gpui-*/**`; new Fast mirrors/vendor source names match the existing broad globs. Add Fast fixture manifests, harness scripts/tests, and any new nested patch-manifest paths explicitly to the key. Regenerate dependency notices from the resolved lock graph, then run the freshness gate. The notice set and lockfile may change only on the F0 integration path.

## 5. Strategy options and recommendation

| Option | Cost and maintenance | Assessment |
|---|---|---|
| A. Switch RustUI to GPUI Fast as its sole rendering core | Highest initial migration: reconcile duplicate `gpui-pre` and Fast type identities, rebase the four known patch conflicts, prove the full matrix, then keep one backend and one patch stack. Lower ongoing duplication. | **Recommended for the evaluation.** Use a separate integration branch; adopt only if every criterion below passes. If the feature leaves two incompatible core identities in the graph, the sole-base migration must resolve that before adoption. |
| B. Support both through a RustUI feature | Requires two tested dependency graphs, patch stacks, mirror/notice/cache matrices, and API compatibility for each release. Additive Cargo features can compile both cores and make public types incompatible. | Highest continuing maintenance; only choose if users require both and the type boundary can remain private. |
| C. Stay on gpui-pre | No migration now; continue implementing retained invalidation and scroll optimizations independently or wait for upstream. | Lowest immediate effort, but it leaves the Stage F performance question unanswered and may duplicate Fast work. |

Recommendation: run F0 with option A as the only adoption candidate, while keeping the current base untouched until the final owner-approved switch. Failure of any adoption criterion means remain on gpui-pre; do not silently ship dual backends.

## 6. Baselines to add before switching

The current baseline is `scripts/perf-baseline.sh`: a release-mode GPUI test-window workload with 1,000 rows, measured three times. A3-7 recorded update-only **4.4850 ms** and update-plus-forced-draw **8.6402 ms**. Add these matched workloads to the current base before evaluating Fast, then run the same tests, Rust version, cache warm-up, machine, and sampling procedure on the candidate:

1. **Long-list scrolling frame cost:** 5,000 fixed-height rows, 300 deterministic scroll offsets, warmed text and layout; report median and p95 per-frame test-window draw time and rows rebuilt per frame.
2. **Typing in a large editor:** 100 deterministic keystrokes into a 10,000-line CodeEditor with syntax highlighting warmed; report median/p95 key-dispatch-to-draw time and text segments reshaped per edit.
3. **Idle redraw/invalidation cost:** 60 unchanged frames for a window containing 60 mounted panel views; report median/p95 draw time and counts of views rendered, laid out, prepainted, and painted. Include a companion one-state-change frame to ensure real updates propagate.

These are proposed RustUI baselines, not GPUI Fast README measurements. Fast's own README workloads use a different showcase and hardware/path, so they cannot serve as before values.

## 7. Adopt criteria

Adopt GPUI Fast as RustUI's base only if **all** of these pass on the candidate integration branch:

- `scripts/check.sh`: full total at least 1,826 passed, zero failures.
- Every focused matrix and cumulative patch proof passes with actual test output recorded by the runner.
- `scripts/smoke-playground.sh` passes.
- Existing 1,000-row update-plus-forced-draw median is at most **7.776 ms** (at least 10% faster than 8.64 ms).
- The new long-list scrolling benchmark is clearly faster on the same workload; report median, p95, and rebuild counts. Use a predeclared target of at least 20% lower median frame time to make “clearly” measurable.
- No stale-rendering failures occur in automated tests or the owner's native checklist. Include an out-of-entity state change, scroll, text input, accessibility tree update, and popup lifecycle.

If any one condition misses, retain gpui-pre and record the failing metric/case; no partial adoption.

## 8. Rollback and ticket split

Create and push `archive/pre-f0` at the current `origin/main` before integration work, verify the tag commit, then create and push `f0-integration` from that commit. All F0 activation and migration work stays on ticket branches targeting `f0-integration`; main remains unchanged until every criterion passes and the owner authorizes the final PR. To roll back, abandon the integration branch and keep the tag; no vendor or lockfile rollback is needed on main.

| Ticket | Scope | Estimate |
|---|---|---:|
| F0-1 | Baseline scroll, typing, and idle benchmarks on current gpui-pre; test harness and recorded baseline | 2 h |
| F0-2 | Fast fixture/hashes, resolved lock graph, mirror and package inventory, full approved-patch ledger including legacy entries | 2 h |
| F0-3 | Candidate backend wiring and core/macOS patch rebases with apply/reverse and compile gates | 2 h |
| F0-4 | Kit family rebases in dependency order with cumulative proofs | 2 h each family |
| F0-5 | RustUI graph/API migration and focused behavior fixes | 2 h each API family |
| F0-6 | Owned mirrors, manifests, notices, CI cache inputs, external and fresh consumer builds | 2 h |
| F0-7 | Full matrices, cumulative proofs, Playground smoke, three-run performance comparison and native checklist | 2 h |
| F0-8 | Review measurements against criteria; adopt via final integration PR or discard candidate | 1 h |
