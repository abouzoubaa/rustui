# GPUI Kit 0.9.x Upgrade Plan

## Scope and evidence

Plan only: no repository source, vendor, patch, or lockfile was changed for this assessment. Target is the latest release checked on 2026-11-12: GPUI Kit **v0.9.1**, tag commit `39ab849a93b5f15507b969164d49891db8d5b470`. Its `Cargo.toml` pins **gpui-pre 0.3.12** and optional **GPUI Fast 0.1.14**. v0.9.0 pinned 0.3.11 / 0.1.13; v0.9.1 is the selected target.

Sources: [upstream releases](https://github.com/longbridge/gpui-kit/releases), [v0.9.1 Cargo.toml](https://github.com/longbridge/gpui-kit/blob/v0.9.1/Cargo.toml). Disposable sources: `/tmp/a3-kit09-sources/v0.8.1/` and `/tmp/a3-kit09-sources/v0.9.1/`; snapshot archives: `/tmp/a3-kit09-crates/`. The v0.9.1 source `cargo metadata --locked` was run from its disposable source tree with Rust 1.98.0 and exited 0. Its initial offline metadata attempt exited 101 because that disposable tree's index/cache lacked `console_log`; the online locked resolution succeeded.

## 1. Target versions, Rust floor, and crate changes

| Area | Current A2 base | v0.9.1 target |
|---|---:|---:|
| GPUI Kit: `gpui-base`, `gpui-component`, `gpui-component-macros`, `gpui-kit-assets` | 0.8.1 | 0.9.1 |
| GPUI snapshot: `gpui-pre` family | 0.3.10 | 0.3.12 |
| Optional alternate backend | gpui-fast family 0.1.12 | gpui-fast family 0.1.14 |

The upstream workspace lock comparison used the two disposable tag sources (`tomllib` package identity `(name, version, source)`): 1,238 → 1,245 lock entries; 53 registry package-version pairs added and 47 removed. The 32 `gpui-pre*` and 9 `gpui-fast*` pairs are snapshot/backend version changes, not new RustUI default-backend dependencies. Third-party deltas in that release lock include ICU 2.3→2.4, `rustls` 0.23.43→0.23.46, `rustls-webpki` 0.103.15→0.103.16, `webpki-root-certs` 1.0.9→1.0.11, `tinystr` 0.8.4→0.8.5, `yoke` 0.8.3→0.8.4, `yoke-derive` 0.8.2→0.8.4, `zerovec` 0.11.8→0.11.9, `zerovec-derive` 0.11.6→0.11.7, and `synstructure` 0.13.2→0.14.0. New non-GPUI package names in the release lock are `icu_locale_fallback`, `icu_locale_fallback_data`, `icu_segmenter`, `icu_segmenter_data`, and `objc2-uniform-type-identifiers`.

A3-1 resolved a disposable RustUI GPUI dependency probe with native + macOS test-support features: 804 reachable lock packages. Compared with the repository lock snapshot, the probe has 145 added and 76 removed package identities (GPUI Kit/gpui-pre version replacements are expected). Its pinned archive fixture contains 82 SHA-256-verified crates: 34 GPUI Kit/pre and patch-target sources, 48 additional exact registry package versions from this lock graph, including the two GPUI Fast 0.1.14 packages referenced by the new Kit manifests. Of those 48 registry entries, 28 names are absent from the current vendor tree (including two GPUI Fast packages) and 20 have another version already present; the three new upstream dependency names confirmed here are `icu_segmenter`, `icu_segmenter_data`, and `objc2-uniform-type-identifiers`. This is a fixture-only inventory: it does not edit `vendor/`, root `Cargo.lock`, or activate the target. A3-5/A3-6 must decide which exact package versions are needed by the resolved native build before any vendor or lock change.

Rust minimum: the tag declares edition 2024 (edition floor 1.85) and no `rust-version`. Source scanning of the Kit and gpui-pre 0.3.12 archives found `str::floor_char_boundary` / `ceil_char_boundary` (stable since 1.91), `Option::is_none_or` (1.82), `div_ceil` (1.73), and `is_multiple_of` (1.87), so the highest found standard-library floor is 1.91. The locked v0.9.1 workspace metadata resolved 1,244 packages; among 757 packages declaring `rust-version`, the highest declaration is **Rust 1.92** (`oo7` 0.6.0). Both floors are below RustUI's **1.98.0**. This is source/API and metadata evidence, not a full compile on the final activated RustUI graph.

## 2. Approved patch matrix

Mechanical status was tested against the downloaded v0.9.1 Kit source and gpui-pre 0.3.12 archives in `/tmp/a3-kit09-fixture`, keeping each prior path prefix and applying successful patches in `patches/approved/kit08-stack.json` order. For each patch, `git apply --check` ran before applying it; failures were left unapplied and later checks continued. `APPLIES` means this ordered source-image check succeeded; `REBASE` means the hunk needs rework; neither is an intent verdict. Counts: **APPLIES 41 / REBASE 18 / UNKNOWN 0** (59 patches).

Intent review compared the target release notes and named upstream PRs with each patch purpose. Intent counts: **OBSOLETE 0 / PARTIAL 3 / STILL NEEDED 56**. No v0.9.0/v0.9.1 release note was found to fully cover any approved patch's RustUI-tested behavior. `PARTIAL` deltas and overlaps are called out below. Keep all parked/retired inventories as-is until a separate approved activation/retirement ticket.

| Patch | Mechanical | Purpose | Intent | v0.9 release-note comparison |
|---|---|---|---|---|
| `a11y-validation-grid-v1-kit08.patch` | REBASE | Expose input validation descriptions and editable-grid readonly/sort metadata. | PARTIAL | Upstream #3727/#3758 adds focused-grid semantics and AT focus synchronization; validation descriptions and explicit cell→header association remain to be proven/rebased. |
| `block-0.1.6-stack-symbol-v1.patch` | APPLIES | Repair the block runtime stack-symbol linkage. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `cosmic-text-initial-bidi-span-v1.patch` | APPLIES | Preserve initial bidirectional span geometry in text shaping. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `g10-labelled-by-tree-v1-kit08.patch` | APPLIES | Expose labelled-by IDs in test-window accessibility-tree output. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `g10-parent-node-id-v1.patch` | APPLIES | Expose the existing AccessKit parent node ID to accessibility callbacks. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-base-calendar-date-limits-v1-kit08.patch` | APPLIES | Clamp calendar month/year calculations to representable date limits. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-base-resize-first-move-v1-kit08.patch` | APPLIES | Apply pointer position immediately when resizing begins. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-base-resize-first-move-v3-kit08.patch` | APPLIES | Apply pointer position immediately and finish resize on left-button release. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-base-slider-endpoint-v1-kit08.patch` | REBASE | Clamp snapped slider values to configured minimum and maximum. | PARTIAL | Upstream #3822/#3823 changes min-origin snapping and permits min above the default max; endpoint clamping still needs an unchanged RustUI range/step regression. |
| `gpui-base-slider-first-move-v1.patch` | APPLIES | Apply pointer position immediately when slider or thumb dragging begins. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-base-slider-set-value-v2-kit08.patch` | REBASE | Handle slider SetValue, Increment, and Decrement accessibility actions consistently. | PARTIAL | Upstream #3840 steps one thumb from assistive technology; explicit SetValue and RustUI min/max/step behavior still needs proof. |
| `gpui-base-slider-snap-position-v1.patch` | REBASE | Recompute thumb geometry after slider range or value updates. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-base-slider-zero-track-v1.patch` | APPLIES | Ignore invalid or zero-sized slider track geometry safely. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-bidi-input-hit-v1-kit08.patch` | REBASE | Use shaped bidirectional clusters for caret and pointer hit testing. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-bidi-wrapped-rows-v1-kit08.patch` | REBASE | Use row-shaped geometry for wrapped bidirectional text painting and positions. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-calendar-keyboard-v1.patch` | APPLIES | Add a separate uncommitted keyboard cursor to calendar navigation. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-calendar-multi-month-heading-v1-kit08.patch` | APPLIES | Position month headings correctly for multi-month calendars. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-calendar-rtl-v1-kit08.patch` | APPLIES | Order calendar cells, navigation, and date-picker layout for RTL. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-component-calendar-fr-ar-v1-kit08.patch` | REBASE | Add French and Arabic calendar message catalog entries. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-component-date-picker-accessibility-v1.patch` | APPLIES | Allow a supplied accessible name on the DatePicker control. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-component-date-picker-localization-v2-kit08.patch` | APPLIES | Resolve DatePicker labels and month text from RustUI localization. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-component-date-picker-rtl-alignment-v1-kit08.patch` | APPLIES | Align DatePicker popup placement and content for RTL. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-component-popup-menu-disabled-confirm-v1-kit08.patch` | APPLIES | Prevent disabled popup-menu selections from invoking actions or dismissing. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-component-select-popup-rtl-v1-kit08.patch` | APPLIES | Apply RTL order and alignment to Select popup rows. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-component-select-rtl-alignment-v1.patch` | APPLIES | Align the Select popup to the logical start of an RTL trigger. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-component-select-rtl-alignment-v2-kit08.patch` | APPLIES | Correct Select popup RTL alignment after the first alignment revision. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-component-select-rtl-alignment-v3-kit08.patch` | APPLIES | Correct Select popup RTL alignment after the second alignment revision. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-component-select-rtl-auto-width-v4-kit08.patch` | APPLIES | Measure all option labels to fit an automatic Select popup width. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-component-select-rtl-auto-width-v6-kit08.patch` | APPLIES | Cache option-label measurements for automatic Select popup width. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-component-sheet-close-accessibility-v2.patch` | REBASE | Expose a configurable accessible name for the Sheet close button. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-component-tracked-dialog-focus-rebase-v2-kit08.patch` | REBASE | Close a specific stacked dialog and restore its successor focus safely. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-component-tree-accessible-name-v1-kit08.patch` | APPLIES | Expose an accessible name on the Tree root. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-component-web-syntax-runtime-v1-kit08.patch` | APPLIES | Use the runtime syntax registry instead of a fixed syntax feature path. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-component-web-syntax-runtime-v2-kit08.patch` | APPLIES | Refine runtime syntax registration for highlighter state. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-component-web-syntax-runtime-v3-kit08.patch` | APPLIES | Route Diff and Input syntax selection through the runtime registry. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-component-web-syntax-v1.patch` | REBASE | Adjust syntax dependency features for the target package configuration. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-input-placeholder-color-v1.patch` | REBASE | Apply configured placeholder color to input and editor placeholders. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-macos-anchored-popup-kit08-v1.patch` | REBASE | Implement native macOS anchored-popup window creation. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-macos-focused-element-forwarder-v1-kit08.patch` | APPLIES | Forward focused-element information from the macOS platform window. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-macos-popup-child-teardown-v1.patch` | REBASE | Close native child popups when the parent window is torn down. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-pre-test-a11y-activation-v1.patch` | APPLIES | Enable accessibility tree construction in GPUI test windows. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-pre-web-accessible-name-v1.patch` | APPLIES | Expose accessible names through the platform accessibility bridge. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-primary-mousedown-click-v2.patch` | APPLIES | Advertise Click for elements whose primary mouse-down listener activates them. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-selectable-list-semantics-v2.patch` | APPLIES | Expose listbox, option, and selected-state semantics for selectable lists. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-toolbar-rtl-v1-kit08.patch` | REBASE | Mirror toolbar ordering and alignment for RTL layouts. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-tree-aria-disabled-row-v1-kit08.patch` | APPLIES | Expose disabled state on Tree row accessibility nodes. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-tree-aria-disabled-v1-kit08.patch` | APPLIES | Expose disabled state on Tree rows and preserve disabled semantics support. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-tree-aria-level-v1-kit08.patch` | APPLIES | Expose one-based aria-level values on Tree rows. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `gpui-tree-disabled-keyboard-v2.patch` | APPLIES | Prevent disabled Tree folders from expanding through keyboard actions. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `r15-breadcrumb-disabled-v1.patch` | APPLIES | Preserve disabled Breadcrumb item semantics and interaction behavior. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `r15-button-disabled-focus-v1-kit08.patch` | APPLIES | Move focus onward when a focused Button becomes disabled. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `r15-sidebar-disabled-v1.patch` | APPLIES | Preserve disabled Sidebar item semantics and interaction behavior. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `r15-test-tree-disabled-v1-kit08.patch` | APPLIES | Expose disabled state and bounds in the GPUI test accessibility tree. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `r15-tree-disabled-focus-v1.patch` | APPLIES | Move focus past a Tree group when it becomes disabled. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `r15b-dropdown-menu-refresh-v1.patch` | APPLIES | Refresh live dropdown item descriptors while preserving menu state and focus. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `r15b-menu-item-disabled-v1.patch` | REBASE | Expose disabled state on popup-menu item accessibility nodes. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `r15b-popup-menu-recovery-v1.patch` | REBASE | Move selection to an enabled popup-menu item after live disable or rebuild. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `t13-modal-focus-recovery-v1-kit08.patch` | REBASE | Restore focus to the next modal or owner after modal removal. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |
| `t17-inert-focus-scope-v1-kit08.patch` | REBASE | Exclude inert focus scopes from keyboard traversal and restore traversal after changes. | STILL NEEDED | Not identified as fixed by v0.9.0/v0.9.1 release notes; retain unless A3 proof or source inspection disproves need. |

Upstream overlap details:
- DataTable #3727/#3758 adds a focused grid role and moves assistive-technology focus with selection; v0.9.1 #3773 names rows and #3731 adds Tab/Shift+Tab exit. These do not establish the RustUI cell-to-real-header relation. In `a11y-validation-grid-v1-kit08.patch`, keep the input validation-description portion and re-evaluate only the table delta against the unchanged A2 grid tests; this is the first `PARTIAL` row.
- Slider #3823 permits `min` above the default `max`, #3822 snaps drag values from `min`, and #3840 steps a thumb from assistive technology. They overlap the endpoint and action patches but do not prove all RustUI clamp, SetValue, Increment, Decrement, and min/max/step contracts. Retain the uncovered behavior and require the slider regression suite at A3-7.
- Menu #3714 makes submenu rows activatable and #3736 conditions left opening on space; #3729 corrects checked/unchecked menu semantics; #3809/#3814 update sizing and checked-item scroll. None explicitly covers RustUI's disabled-confirm guard, disabled-node state, live item refresh, or disabled-item selection recovery. Keep those patches unless the unchanged menu regressions on v0.9.1 prove equivalent behavior.
- The release notes do not claim native macOS `WindowKind::AnchoredPopup`; the consolidated `gpui-macos-anchored-popup-kit08-v1.patch` remains needed. Its mechanical status is REBASE, and it must stay opt-in with in-window fallback.

## 3. Patch dependency order

Rebuild the stack order before rebasing families; use the existing A2 stack checker as the source of order, then verify every dependency against the 0.9.1 source. Known edges to preserve:

1. `g10-parent-node-id-v1.patch` precedes `g10-labelled-by-tree-v1-kit08.patch`: the latter consumes the parent/accessibility-node identity added by the former.
2. The `gpui-tree-aria-disabled-v1-kit08.patch` provider must precede Tree-row, Breadcrumb, Sidebar, and any test patch that calls its `aria_disabled` extension. This was a real A2 ordering dependency, not upstream API.
3. The shared input accessibility API patch must precede the component Input validation-description consumer; `a11y-validation-grid` currently carries both Base and Component portions and may need splitting for a clean family stack.
4. Calendar/DatePicker shared Base localization/accessibility providers precede Component DatePicker consumers. Select popup direction/measurement foundations precede later alignment and auto-width refinements.

Before each A3-4 family, scan candidate diffs for calls to APIs introduced by other candidate patches; include the provider edge in the stack manifest and test the full cumulative stack. Do not assume the A2 order is still valid merely because names are unchanged.

### A3-2 core/macOS candidate status

The inert core stack contains **10 copied patches, 3 rebased patches, and 1 new Rust 1.98 compatibility delta** (14 total); no patch was retired or marked partial in this step. Rebased files are `gpui-bidi-wrapped-rows-v1-kit08-kit09.patch`, `gpui-bidi-input-hit-v1-kit08-kit09.patch`, and `t17-inert-focus-scope-v1-kit08-kit09.patch`. The new `gpui-macos-map-or-default-v1-kit09.patch` replaces three upstream `Option::map_or_default` calls with equivalent `map_or_else(Default::default, ...)` on the macOS source, because the 0.3.12 source does not compile with the repository's Rust 1.98.0 toolchain.

`python3 scripts/check_kit09_core_stack.py --compile-online --proof-core` passed: 14 candidates applied and reversed exactly (82 verified archives, 21 target sources hashed, 25 files in the resulting stack); `cargo check -p gpui-pre -p gpui-pre-macos` passed. The GPUI fixture tests passed: `inert_scope_skips_tab_and_rejects_direct_focus` (1/1) and the `bidi_` filter (2/2: single-line visual hit and wrapped-row geometry). The test harness adds only disposable missing upstream workspace assets and a no-op test macro crate; neither is added to candidate source.

NP-7 was retried once while inspecting the macOS package, with `env -u RUSTC -u RUSTDOC CARGO_TARGET_DIR="$HOME/.cache/rustui-target" rustup run 1.98.0 cargo test --manifest-path crates/rustui-gpui-macos/Cargo.toml nested_grabbing_popups_dismiss_leaf_before_parent --features test-support --locked --offline`. It remains parked: the currently selected 0.3.10 owned source reports E0658 for `Option::map_or_default` at `crates/rustui-gpui-macos/src/window.rs:862`, `:923`, and `:1454` (issue #138099). The new 0.9.1 candidate has a patch-only compatibility delta; it does not change the current owned source or complete the NP-7 native test.

### A3-3 retirement decision

Completed the intent/proof gate with **0 additional retirements**. None of the approved patches is proven fully obsolete on v0.9.1; the three already classified `PARTIAL` (`a11y-validation-grid-v1-kit08.patch`, `gpui-base-slider-endpoint-v1-kit08.patch`, and `gpui-base-slider-set-value-v2-kit08.patch`) retain only their uncovered behavior and stay in the candidate queue for their component families. The corresponding RustUI regression checks passed on the current baseline: `text_field_validation_lifecycle`, `grid_cells_reference_actual_header_nodes_across_updates`, and `slider_accessibility_set_value_clamps_and_applies_step`. These baseline passes do not establish that the upstream fixture covers the full behavior, so they do not justify retirement. Previously retired A2 patches remain untouched.

## 4. Owned manifests, test support, and locks

Read-only TOML comparison against v0.9.1 manifests found intentional A2 differences that need an explicit rebase decision:

- Core `rustui-gpui`: upstream adds `bench_metrics` to `test-support` and changed dependency/features including `icu_segmenter`; RustUI intentionally trims screen-capture-only feature wiring. Preserve `test-support`, compare each target dependency and feature, and do not restore capture-only optional dependencies by accident.
- `rustui-gpui-macos`: current manifest omits only upstream screen-capture wiring in the compared feature/dependency sections; `test-support = [gpui/test-support, gpui_apple/test-support]` matches. Preserve it for NP macOS tests.
- `rustui-gpui-apple`: `test-support = [gpui/test-support]` matches; dependency key differences require line-by-line comparison before adopting upstream target cfgs.
- `rustui-gpui-platform`: upstream test-support also enables Windows and WGPU test support; determine whether those are required for this repository's enabled test targets. Preserve current capture trimming.
- `rustui-gpui-base`: upstream test-support additionally enables optional `gpui-fast?/test-support`; RustUI's active GPUI-pre backend must retain its GPUI test-support wiring.
- `rustui-gpui-component`: upstream test-support adds optional `gpui-fast?/test-support`; retain GPUI and Base test support.
- `rustui-gpui-web`, `rustui-gpui-windows`, and `rustui-gpui-wgpu`: review target-specific dependency changes; don't activate deferred platforms as part of a native macOS A3 ticket.

Current standalone locks exist in 13 `crates/rustui-*/Cargo.lock` paths: `rustui-gpui`, `rustui-gpui-platform`, `rustui-gpui-macos`, `rustui-gpui-apple`, `rustui-gpui-base`, `rustui-gpui-component`, `rustui-gpui-kit-assets`, `rustui-gpui-linux`, `rustui-gpui-web`, `rustui-gpui-wgpu`, `rustui-gpui-windows`, `rustui-metal`, and `rustui-cocoa`. A3-1 inventories which standalone locks are actually exercised; regenerate only stale test-only locks offline with Rust 1.98.0. Root lock changes stay targeted and are reviewed as a separate A3-6 artifact.

## 5. RustUI API impact

Source-only grep (`rg -n` over `crates/rustui/src apps --glob '*.rs'`) against the v0.9.0 migration notes found these call-site counts; they are triage counts, not compile failures:

| Upstream change | Grep count / call sites |
|---|---|
| `Keystroke::parse` now takes static text; dynamic values require the OS-aware parse path | 40 calls; inspect `crates/rustui/src` and Playground references for dynamic `String` inputs |
| `KeyBinding::new` static-string requirement | 25 calls |
| `Modifiers::secondary()` / `secondary_key()` take `&App` | 0 direct GPUI method matches in RustUI/apps; recheck imports/call chains during compile |
| `Keystroke::unparse` / keybinding unparse take `OperatingSystem` | 0 direct unparse calls |
| `PlatformInput::ScrollWheel` → `Scroll` / scroll input source field | 4 `ScrollWheel` / `ScrollWheelEvent` matches |
| `Window::text_system()` returns `&Rc<WindowTextSystem>`; no longer Send/Sync | 1 direct call; inspect any values moved into Send closures during compilation |
| `DisplayMap` no longer `UnwindSafe`/`RefUnwindSafe` | 0 direct type mentions in RustUI/apps |
| Public Kit theme structs become `#[non_exhaustive]` | 14 struct-literal-pattern grep hits; inspect the actual affected theme types before editing |

Expected APIs from release notes are detailed in the A3-5 compile ticket: migrate dynamic keyboard strings without changing shortcut behavior; pass `&App` for secondary modifiers only where used; update scroll variant/event literals; remove Arc/Send assumptions around `WindowTextSystem`; replace external theme literals with constructors plus field assignment. v0.9.1 is described as backward-compatible relative to v0.9.0, but still verify its additional APIs against existing RustUI calls.

## 6. Parked, retired, rollback, and integration flow

The **5 deferred-platform patches remain in `patches/parked/` unchanged**. The **11 retired patches remain in `patches/retired/`**, including the 6 A2 upstream-obsolete proofs and the 4 superseded popup prototypes; do not revive them during A3. The active consolidated native popup patch remains in `patches/approved/` and requires a v0.9.1 rebase.

Before any A3 source changes, create and verify `archive/pre-a3` at the then-current `origin/main`. Branch `a3-integration` from updated main. All A3 activation work lands only on that integration branch through sub-branch PRs; main remains untouched until the complete A3-7 gate passes and the owner-approved final merge PR is green. Abort by discarding the sub-branch and resetting the integration branch to the recorded archive commit; never reset main.

## 7. Proposed ticket split (each ≤2 hours, CI green independently)

| Ticket | Scope | Estimate / exit condition |
|---|---|---|
| A3-1 | **Completed:** pin v0.9.1 / gpui-pre 0.3.12 and lock-graph archives with SHA-256s; add the disposable fixture extractor and inventory command. The 804-package dependency probe and 82-archive fixture are recorded above; selected sources remain unchanged. | Hash-mismatch/missing-file/path-traversal tests and main-source gates pass. |
| A3-2 | Core GPUI-pre text/input/window/a11y/focus/tab-stop and macOS platform patch rebase; compile core and macOS owned crates in fixture. | ≤2h per core subfamily; full apply/reverse manifest and core proofs pass. |
| A3-3 | Prove v0.9.1 coverage for each proposed retirement; move only proven-obsolete candidates in the inert integration branch, with RustUI regression names retained. | ≤2h; every retired behavior test passes unchanged. |
| A3-4a…n | Rebase Base/Component families in dependency order: focus/dialog/sheet; menu/select/popup; tree/list/table/grid; slider/resize; calendar/DatePicker; input/text/placeholder; toolbar/RTL; syntax/Diff; native popup; remaining isolated families. One family per ≤2h PR. | ≤2h each; cumulative apply/reverse, Base/Component compile gate, family proof, and earlier proofs pass. |
| A3-5 | RustUI API migration in small groups: keyboard shortcuts, scroll input, window text-system ownership, non-exhaustive theme construction, then integration-specific fixes. | ≤2h per group; unchanged tests and focused migration tests pass. |
| A3-6 | Owned mirror refresh, manifest/test-support alignment, targeted root/standalone lock changes, notices, CI cache inputs, consumer build against the integration branch. | ≤2h per artifact group; mirror/notices checks and fresh consumer pass. |
| A3-7 | Full candidate proofs and final switch: current full `scripts/check.sh` count or higher; `verify-release.sh source`; owned-source verification; focused accessibility/disabled/async/popup/focus/inert/grid/validation/file-drop/theme suites; cumulative A2/A3 proofs; smoke; perf ≤8.5 ms. | ≤2h; merge `a3-integration` to main only after every gate passes and source CI is green. |

## 8. Final gates

Reuse the A2 proof runners and require: (1) `scripts/check.sh` at least the current **1,826 passed**, zero failures; (2) `./scripts/verify-release.sh source` and `python3 scripts/verify_owned_gpui_sources.py` exit 0; (3) all focused RustUI suites listed in A3-7 pass; (4) full cumulative candidate-stack proofs pass on the 0.9.1 fixture; (5) `scripts/smoke-playground.sh` exits 0; (6) `scripts/perf-baseline.sh` 1,000-row forced-draw median ≤ **8.5 ms**. If any gate fails, keep integration isolated and report the exact failure; do not merge to main.
