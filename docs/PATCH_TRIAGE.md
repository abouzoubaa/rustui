# Proposed patch triage

Baseline: `601ff794937bcfb1c8f47f17d3303ceb88e61d40` (H0 merge). 29 proposals at the P0 baseline; historical recommendations: PROMOTE **2**, REBASE **6**, REJECT **12**, LATER **9**.

Applicability used `git apply --check` in `/tmp/p0`; non-macOS platform targets were not applied or tested. For a clean apply, the changed vendor files were mirrored into the corresponding owned source **only in the disposable worktree**, so the workspace build exercised the patch. Each candidate was reset before the next. Builds used `rustup run 1.97.1 cargo check --workspace --all-targets --offline`; tests used `cargo test -p rustui --lib --offline <filter>`. `NT` means not tested because of a conflict or deferred platform. A successful broad or related test is not proof of native or VoiceOver behavior.

| Patch and companions (under `patches/proposed/`) | Target | Purpose | Status | Applies? | Build | Related tests | Recommendation | Reason |
|---|---|---|---|---|---|---|---|---|
| `block-0.1.6-stack-symbol.patch`<br>Companions: block-stack-symbol-ffi.c, block-stack-symbol-probe.rs, block-stack-symbol-rust-probe.rs | `vendor/block-0.1.6/src/lib.rs` | block stack symbol linkage | Removed in P1 | No: vendor/block-0.1.6/src/lib.rs:61 | NT | NT | REJECT | Already approved. |
| `gpui-base-focus-trap-accessibility-v1.patch` | `vendor/gpui-base-0.7.3/src/focus_trap.rs` | Forward focus-trap accessibility role and properties | Promoted in P1 | Yes | exit 0 | `focus_recovery`: 5 passed, 0 failed | PROMOTE | Dialog role can be lost through the focus-trap wrapper; accessibility. |
| `gpui-base-slider-set-value-v2.patch` | `vendor/gpui-base-0.7.3/src/slider.rs` | Slider SetValue and increment/decrement actions | Promoted in P2b | Yes | exit 0 | Test-window action-handler and bound-value regression | PROMOTED | Approved v2 replaces the old v1 proposal; min/max endpoints and step are preserved. |
| `gpui-bidi-input-hit-v1.patch` | `vendor/gpui-pre-0.3.9/src/text_system/line_layout.rs`, `vendor/gpui-pre-0.3.9/src/text_system/line.rs`, `vendor/gpui-pre-0.3.9/src/platform.rs`, `vendor/gpui-pre-web-0.3.9/src/run_replacements.rs`, `vendor/gpui-pre-wgpu-0.3.9/src/cosmic_text_system.rs`, `vendor/gpui-pre-apple-0.3.9/src/text_system.rs`, `vendor/gpui-pre-windows-0.3.9/src/direct_write.rs`, `vendor/gpui-base-0.7.3/src/text/inline.rs` | Bidi hit testing across text backends | Deferred platform; identical approved patch | NT — deferred platform | NT | NT | LATER | Cross-platform text paths; defer platform testing. |
| `gpui-bidi-wrapped-rows-v1.patch`<br>Companions: gpui-bidi-wrapped-rows-v1.json | `vendor/gpui-pre-0.3.9/src/platform.rs`, `vendor/gpui-pre-0.3.9/src/text_system/line.rs`, `vendor/gpui-pre-0.3.9/src/text_system/line_layout.rs`, `vendor/gpui-pre-web-0.3.9/src/text_system.rs`, `vendor/gpui-pre-wgpu-0.3.9/src/cosmic_text_system.rs` | Bidi wrapped-row geometry across text backends | Deferred platform; identical approved patch | NT — deferred platform | NT | NT | LATER | Cross-platform text paths; defer platform testing. |
| `gpui-component-date-picker-accessibility.patch` | `vendor/gpui-component-0.7.3/src/time/date_picker.rs` | Date picker accessibility metadata | Removed in P1 | No: vendor/gpui-component-0.7.3/src/time/date_picker.rs:551 | NT | NT | REJECT | Already approved. |
| `gpui-component-date-picker-rtl-alignment.patch` | `vendor/gpui-component-0.7.3/src/time/date_picker.rs` | Date picker RTL alignment | Removed in P1 | No: vendor/gpui-component-0.7.3/src/time/date_picker.rs:566 | NT | NT | REJECT | Already approved. |
| `gpui-component-modal-focus-v1.patch`<br>Companions: gpui-component-modal-focus-v1-test.rs, check_modal_focus_proposal.py | `src/root.rs`, `src/window_ext.rs` | Restore top modal focus | Removed in P1 | No: src/root.rs: No such file or directory; src/window_ext.rs: No such file or directory | NT | NT | REJECT | T13 supplies modal recovery; old paths are package-relative. |
| `gpui-component-popup-menu-disabled-confirm.patch` | `vendor/gpui-component-0.7.3/src/menu/popup_menu.rs` | Block confirmation of disabled menu items | Promoted in P1 | Yes | exit 0 | `disabled_confirmation`: ok. 2 passed; 0 failed; 0 ignored; `menu_renders_with_composited_theme_in_both_appearances`: 1 passed | PROMOTE | Disabled menu entries can otherwise invoke actions and dismiss the menu. |
| `gpui-component-sheet-close-accessibility-v1.patch` | `vendor/gpui-component-0.7.3/src/sheet.rs` | Allow an accessible name for sheet close button | Promoted in P2 (v2) | Yes | exit 0 | `sheet`: ok. 24 passed; 0 failed; 0 ignored | LATER | Needs owner decision on wiring the new optional label into RustUI and direct accessibility tests. |
| `gpui-component-slider-accessibility-label-v2.patch` | `vendor/gpui-component-0.7.3/src/slider.rs` | Allow an accessible slider label | Promoted in P2b | Yes | exit 0 | Component and RustUI test-window accessibility trees | PROMOTED | Approved v2 forwards the label to the slider node; the old v1 proposal is removed. |
| `gpui-component-tracked-dialog-focus-rebase.patch` | `vendor/gpui-component-0.7.3/src/root.rs`, `vendor/gpui-component-0.7.3/src/window_ext.rs`, `vendor/gpui-component-0.7.3/src/lib.rs`, `vendor/gpui-component-0.7.3/src/dialog/dialog.rs` | Identify and close a specific stacked dialog | Promoted in P2 (v2) | Yes (v2) | exit 0 | `tracked_dialog_closes_specific_opening`: 1 passed | PROMOTE | Close a specific stacked dialog while preserving focus. |
| `gpui-component-tracked-dialog.patch` | `vendor/gpui-component-0.7.3/src/root.rs`, `vendor/gpui-component-0.7.3/src/window_ext.rs`, `vendor/gpui-component-0.7.3/src/lib.rs` | Identify and close a specific stacked dialog | Removed in P1 | No: vendor/gpui-component-0.7.3/src/root.rs:32 | NT | NT | REJECT | Older version of the retained rebased proposal. |
| `gpui-component-tree-accessible-name-v1.patch` | `vendor/gpui-base-0.7.3/src/tree.rs`, `vendor/gpui-component-0.7.3/src/tree.rs` | Expose tree root accessible names | Removed in P1 | No: vendor/gpui-base-0.7.3/src/tree.rs:498; vendor/gpui-component-0.7.3/src/tree.rs:2 | NT | NT | REJECT | Already approved. |
| `gpui-cosmic-wrapped-rows-seam-v1.patch`<br>Companions: gpui-cosmic-wrapped-rows-seam-v1.json, cosmic-wrapped-rows-probe/{Cargo.toml, Cargo.lock, src/lib.rs} | `vendor/gpui-pre-wgpu-0.3.9/src/cosmic_text_system.rs` | Expose wrapped-row geometry in cosmic text backend | Deferred platform | NT — deferred platform | NT | NT | LATER | Targets the deferred wgpu backend. |
| `gpui-macos-anchored-popup-v5-coordinate-fix.patch` | `vendor/gpui-pre-macos-0.3.9/src/window.rs` | Correct AppKit popup anchor coordinates | Not approved | Yes | exit 101 | `popover`: exit 101 (compile failed) | REBASE | Zero-context apply corrupts `gpui-pre-macos/src/window.rs`; compiler rejects `let` at item level. |
| `gpui-pre-quad-stable-sort-v2.patch` | `vendor/gpui-pre-0.3.9/src/scene.rs` | Candidate stable quad sorting | Removed in P1 | Yes | exit 0 | `transition`: ok. 12 passed; 0 failed; 0 ignored | REJECT | T20 found no meaningful difference; ordered regression is not covered. |
| `gpui-pre-quad-stable-sort.patch` | `vendor/gpui-pre-0.3.9/src/scene.rs` | Earlier stable quad sorting candidate | Removed in P1 | Yes | exit 0 | `transition`: ok. 12 passed; 0 failed; 0 ignored | REJECT | Older experiment; V2 was measured and closed in T20. |
| `gpui-pre-test-a11y-activation-v1.patch` | `vendor/gpui-pre-0.3.9/src/platform/test/window.rs` | Activate accessibility in GPUI test windows | Removed in P1 | No: vendor/gpui-pre-0.3.9/src/platform/test/window.rs:1 | NT | NT | REJECT | Already approved. |
| `gpui-pre-web-accessible-name-v1.patch` | `vendor/gpui-pre-0.3.9/src/window.rs`, `vendor/gpui-pre-web-0.3.9/src/window.rs`, `vendor/gpui-pre-web-0.3.9/src/ime_mirror.rs` | Accessible names in the deferred platform backend | Deferred platform; identical approved patch | NT — deferred platform | NT | NT | LATER | Targets the deferred platform backend. |
| `gpui-pre-web-keyboard-focus-activation-v1.patch` | `vendor/gpui-pre-web-0.3.9/src/window.rs`, `vendor/gpui-pre-web-0.3.9/src/events.rs` | Keyboard and focus activation, first version | Deferred platform; superseded by approved V3 | NT — deferred platform | NT | NT | LATER | Targets the deferred platform backend. |
| `gpui-pre-web-keyboard-focus-activation-v2.patch` | `vendor/gpui-pre-web-0.3.9/src/window.rs`, `vendor/gpui-pre-web-0.3.9/src/events.rs` | Keyboard and focus activation, second version | Deferred platform; superseded by approved V3 | NT — deferred platform | NT | NT | LATER | Targets the deferred platform backend. |
| `gpui-pre-web-keyboard-focus-activation-v3.patch` | `vendor/gpui-pre-web-0.3.9/src/window.rs`, `vendor/gpui-pre-web-0.3.9/src/events.rs` | Keyboard and focus activation, third version | Deferred platform; identical approved patch | NT — deferred platform | NT | NT | LATER | Targets the deferred platform backend. |
| `gpui-primary-mousedown-click-v1.patch`<br>Companions: gpui-row-accessibility-series-v1.json, row-accessibility-probe.rs, row-accessibility-probe.Cargo.toml.in | `vendor/gpui-pre-0.3.9/src/elements/div.rs` | Expose accessibility Click for primary-mousedown listeners | Promoted in P2 (v2) | Yes; pinned preimage differs | exit 0 | `accessibility`: ok. 93 passed; 0 failed; 0 ignored | REBASE | `gpui-pre/src/elements/div.rs` preimage differs from pinned source despite a clean textual apply. |
| `gpui-selectable-list-semantics-v1.patch`<br>Companions: gpui-row-accessibility-series-v1.json, row-accessibility-probe.rs, row-accessibility-probe.Cargo.toml.in | `vendor/gpui-component-0.7.3/src/list/list.rs` | Expose listbox, option and selection semantics | Promoted in P2 (v2) | Yes; pinned preimage differs | exit 0 | `list`: ok. 42 passed; 0 failed; 0 ignored | PROMOTE | Selectable lists expose listbox, option and selected state. |
| `gpui-tree-aria-disabled.patch` | `vendor/gpui-pre-0.3.9/src/elements/div.rs`, `vendor/gpui-base-0.7.3/src/tree.rs`, `vendor/gpui-base-0.7.3/src/switch.rs` | Expose disabled tree semantics | Removed in P1 | No: vendor/gpui-pre-0.3.9/src/elements/div.rs:2110; vendor/gpui-base-0.7.3/src/tree.rs:463; vendor/gpui-base-0.7.3/src/switch.rs:732 | NT | NT | REJECT | Already approved. |
| `gpui-tree-aria-level-v1.patch` | `vendor/gpui-base-0.7.3/src/tree.rs` | Expose tree item depth | Removed in P1 | No: vendor/gpui-base-0.7.3/src/tree.rs:461 | NT | NT | REJECT | Already approved. |
| `gpui-tree-disabled-keyboard-v1.patch`<br>Companions: row-accessibility-probe.rs, row-accessibility-probe.Cargo.toml.in | `vendor/gpui-base-0.7.3/src/tree.rs` | Block expansion of disabled tree folders | Promoted in P2 (v2) | No: vendor/gpui-base-0.7.3/src/tree.rs:353 | NT | NT | REBASE | Useful disabled-control fix; `gpui-base/src/tree.rs` preimage and hunks conflict. |
| `rustui-modal-focus-integration-v1.patch`<br>Companions: check_modal_focus_proposal.py | `crates/rustui/src/runtime/window.rs` | Invoke modal focus recovery during traversal | Removed in P1 | Yes; pinned preimage differs | exit 101 | `focus_recovery`: exit 101 (compile failed) | REJECT | Depends on absent `focus_active_modal`; T13 recovery tests pass on the current branch. |

## Preimage comparisons

- Matched: focus trap `5a110237…b1d59e8be6`; popup menu `3e2e2b4a…ea2cb0954f2`; sheet `0bfd572b…d428fc5d5af`; component slider `94afa288…cbdb984d`; quad scene `3addca59…aa5b8880`. The popup and focus-trap matches are their proposal pins.
- Block preimage `eb31678a…6a146fa` differs from current `798f4b28…8b4dd`, which matches the approved postimage. DatePicker RTL preimage `83e639bb…462467d` differs from current `8d8e8a7f…d179712`.
- Changed: base slider expected `b677bee1…ff9e0b6`, actual `f9c3b88a…6115f1f`; DatePicker expected `e7c8c87c…0095248`, actual `8d8e8a7f…d179712`; tracked-dialog root expected `c438aabc…b9c2749`, actual `a2c28baa…1037457`. Its `window_ext.rs` and `lib.rs` pins still match.
- The approved tree-series preimages differ and all three current sources match their approved postimages. The disabled-keyboard Tree pin expected `9ef2efa0…736924`, actual `0a1321f5…8e4747`.
- Row-accessibility `div.rs` expected `de8298ce…1033b84e`, actual `81429bdb…c819df50`; `list/list.rs` expected `50103222…544a453`, actual `b7cc17c9…4034edd`. RustUI modal integration expected `9899b606…10efd85`, actual `8e6f445c…09f0576`.

## Full checks and stacking

- Focus-trap candidate: `scripts/check.sh` exit 0; 1,584 passed, 0 failed, 4 ignored. Popup-menu candidate: exit 0; 1,584 passed, 0 failed, 4 ignored. Both ran independently with the corresponding owned mirror updated only in the disposable worktree.
- The two promotion candidates touch different files; applying focus-trap first and checking popup-menu second succeeded.
- Shared-file groups: the quad V1 and V2 patches touch `scene.rs` and **do not stack** (V2 failed its check after V1). The two DatePicker proposals share `date_picker.rs`; both are already approved and neither proposed copy reapplies on the current source. The tree name, level, disabled-state, and disabled-keyboard proposals share `tree.rs`; their proposed copies do not apply on the current source, so their proposed stack could not be tested. The disabled-state Tree proposal also shares `div.rs` with primary-mousedown, but cannot be reapplied to the current source. The tracked-dialog variants share `root.rs` and both conflict on the current source. Reassess these stacks against the approved postimages before any promotion. Deferred-platform groups were not applied or tested.
- Focus-recovery tests do not inspect the forwarded accessibility tree; menu and disabled-selection tests do not directly exercise a disabled `PopupMenu::confirm` event. Add targeted behavior checks during promotion; these results are not VoiceOver or native acceptance.

## Suggested promotion order

Both recommendations below were promoted in P1.

1. **Focus trap accessibility** — forward dialog role and accessibility properties; add a direct accessibility-tree regression.
2. **Disabled popup confirmation** — prevent disabled menu actions and dismissal; add a direct confirmation regression.

RTL proposals in this inventory either include deferred platform paths or already have an approved equivalent. Rebase the remaining accessibility and disabled-tree proposals after the two candidates, with owner direction for deferred or prerequisite-dependent work.

## P2 progress

- Slider pair: promoted in P2b as `gpui-base-slider-set-value-v2.patch` and `gpui-component-slider-accessibility-label-v2.patch`. Test-window trees expose labels and advertise SetValue/Increment/Decrement only when enabled. `Window::handle_a11y_action` is crate-private, so tests call the actual registered slider handler functions directly and verify RustUI binding updates, min/max, and step.
- Sheet close accessibility: promoted in P2 as `gpui-component-sheet-close-accessibility-v2.patch`; English and French close labels passed test-window accessibility-tree checks.
- Disabled Tree keyboard: promoted in P2 as `gpui-tree-disabled-keyboard-v2.patch`; direct and RustUI keyboard regressions passed, and the unpatched direct control failed as expected.
- Primary mouse-down accessibility: promoted in P2 as `gpui-primary-mousedown-click-v2.patch`; the test-window tree advertises Click only for primary mouse-down and existing click handlers.

- Selectable list semantics: promoted in P2 as `gpui-selectable-list-semantics-v2.patch`; selectable and non-selectable accessibility tree tests pass.

- Tracked dialog focus: promoted in P2 as `gpui-component-tracked-dialog-focus-rebase-v2.patch`; identity-specific closing preserves the remaining modal and focus chain.

## R15 disabled controls

- `r15-test-tree-disabled-v1.patch`: promoted; pinned replay exposes existing disabled flags and device bounds in the test-tree inspector.
- `r15-button-disabled-focus-v1.patch`: promoted; a focused button moves to the next element when disabled.
- `r15-breadcrumb-disabled-v1.patch`: promoted; label, disabled semantics, Tab focus and disable/re-enable behavior passed the disposable test-window matrix.
- `r15-sidebar-disabled-v1.patch`: promoted; disabled item semantics passed the disposable test-window matrix and are pinned by exact replay.
- `r15-tree-disabled-focus-v1.patch`: promoted; disabled-group focus passed the disposable test-window matrix, and exact stack replay preserves the P2 Tree pins.

## R15b popup-menu recovery

- `r15b-menu-item-disabled-v1.patch`: promoted; popup-menu item nodes forward the existing aria-disabled property.
- `r15b-popup-menu-recovery-v1.patch`: promoted; live rebuilds retain focus and choose the next enabled item or no selection; arrow and hover selection skip disabled items. Exact replay preserves the earlier disabled-confirm patch beneath it.
- `r15b-dropdown-menu-refresh-v1.patch`: promoted; RustUI opts into refreshing live item descriptors while retaining the menu entity and focus handle.

## G10 — cell/header relations

- `g10-parent-node-id-v1`: promoted; exposes the actual current element node ID to its accessibility callback.
- `g10-labelled-by-tree-v1`: promoted; exposes actual labelled-by IDs in test-window tree inspection, stacked over the approved R15 disabled/bounds layer.
- Disposable-worktree grid tree, full stable checks and source-release gates passed; apply/reverse/tamper script tests and owned mirrors are complete.
