# Approved GPUI Fast stack

Promoted in FA-1 from the active F0 stack. Patch bytes and source images are unchanged. FA-2 adds owned dependency wiring: there are now 65 ordered patches plus one alternative Component-manifest postimage proof (66 approved patch files). The 64 promoted F0 patch files remain unchanged.

`fast-stack.json` defines cumulative order. `python3 scripts/check_fast_patch_stack.py --verify` checks the final hashes, reverses and reapplies the ordered stack in a disposable fixture, and separately replays the alternative postimage. Existing family/runtime proofs remain in the source gate.

| Patch | Target crate(s) | Reason |
|---|---|---|
| `gpui-base-owned-gpui-pre-optional-v1` | `rustui-gpui-base` | Keep the legacy backend optional when Fast is selected. |
| `gpui-component-owned-gpui-pre-optional-v1` | `rustui-gpui-component` | Keep the legacy backend optional when Fast is selected. |
| `gpui-kit-assets-owned-gpui-pre-optional-v1` | `rustui-gpui-kit-assets` | Keep the legacy backend optional when Fast is selected. |
| `gpui-component-popup-menu-disabled-confirm-v1-kit08` | `gpui-component-0.9.1` | Prevent activation of disabled popup-menu items. |
| `r15b-menu-item-disabled-v1-kit09` | `gpui-component-0.9.1` | Expose disabled popup-menu item accessibility state. |
| `r15b-popup-menu-recovery-v1-kit09` | `gpui-component-0.9.1` | Recover selection/focus when an open menu item becomes disabled. |
| `r15b-dropdown-menu-refresh-v1` | `gpui-component-0.9.1` | Refresh dropdown state across disable/re-enable. |
| `gpui-component-select-popup-rtl-v1-kit08` | `gpui-component-0.9.1` | Align popup direction with RTL select content. |
| `gpui-component-select-rtl-alignment-v1` | `gpui-component-0.9.1` | Preserve select field alignment for RTL labels and values. |
| `gpui-component-select-rtl-alignment-v2-kit08` | `gpui-component-0.9.1` | Preserve select field alignment for RTL labels and values. |
| `gpui-component-select-rtl-alignment-v3-kit08` | `gpui-component-0.9.1` | Preserve select field alignment for RTL labels and values. |
| `gpui-component-select-rtl-auto-width-v4-kit08` | `gpui-component-0.9.1` | Measure RTL selects without incorrect automatic popup widths. |
| `gpui-component-select-rtl-auto-width-v6-kit08` | `gpui-component-0.9.1` | Measure RTL selects without incorrect automatic popup widths. |
| `gpui-selectable-list-semantics-v2` | `gpui-component-0.9.1` | Preserve selectable list roles, selection and keyboard semantics. |
| `gpui-component-tree-accessible-name-v1-kit08` | `gpui-base-0.9.1`, `gpui-component-0.9.1` | Preserve tree accessible names and control semantics. |
| `gpui-component-sidebar-disabled-a11y-v1-kit09` | `gpui-component-0.9.1` | Expose disabled sidebar item accessibility state. |
| `gpui-tree-aria-disabled-row-v1-kit08` | `gpui-base-0.9.1` | Expose disabled state on semantic/accessibility nodes. |
| `gpui-tree-aria-level-v1-kit08` | `gpui-base-0.9.1` | Expose one-based tree accessibility levels. |
| `gpui-tree-disabled-keyboard-v2` | `gpui-base-0.9.1` | Block disabled tree keyboard activation. |
| `r15-tree-disabled-focus-v1` | `gpui-base-0.9.1` | Move focus away from a newly disabled tree item. |
| `gpui-component-table-delegate-accessibility-v1-kit09` | `gpui-component-0.9.1` | Let EditableDataGrid provide header/cell indices and real header relations. |
| `gpui-base-slider-endpoint-v1-kit09` | `gpui-base-0.9.1` | Map slider endpoints correctly. |
| `gpui-base-slider-first-move-v1` | `gpui-base-0.9.1` | Make the first slider drag update its value. |
| `gpui-base-slider-set-value-v2-kit09` | `gpui-base-0.9.1` | Handle accessibility SetValue, Increment and Decrement with bounds/steps. |
| `gpui-base-slider-zero-track-v1` | `gpui-base-0.9.1` | Handle zero-length slider tracks safely. |
| `gpui-base-resize-first-move-v1-kit08` | `gpui-base-0.9.1` | Preserve resize behavior from the first drag movement. |
| `gpui-base-resize-first-move-v3-kit08` | `gpui-base-0.9.1` | Preserve resize behavior from the first drag movement. |
| `gpui-base-calendar-date-limits-v1-kit08` | `gpui-base-0.9.1` | Enforce calendar minimum/maximum selectable dates. |
| `gpui-calendar-keyboard-v1` | `gpui-base-0.9.1`, `gpui-component-0.9.1` | Support calendar keyboard navigation. |
| `gpui-calendar-rtl-v1-kit08` | `gpui-base-0.9.1`, `gpui-component-0.9.1` | Mirror calendar layout/navigation for RTL. |
| `gpui-calendar-multi-month-heading-v1-kit08` | `gpui-base-0.9.1` | Keep month headings correct in multi-month calendars. |
| `gpui-component-calendar-fr-ar-v1-kit09` | `gpui-component-0.9.1` | Provide French and Arabic calendar labels. |
| `gpui-component-date-picker-accessibility-v1` | `gpui-component-0.9.1` | Expose the date-picker field accessibility semantics. |
| `gpui-component-date-picker-localization-v2-kit08` | `gpui-component-0.9.1` | Format date-picker content with the selected locale. |
| `gpui-component-date-picker-rtl-alignment-v1-kit08` | `gpui-component-0.9.1` | Align date-picker content in RTL. |
| `gpui-base-input-validation-description-v1-kit09` | `gpui-base-0.9.1`, `gpui-component-0.9.1` | Put validation descriptions and disabled state on the editor-owned node. |
| `gpui-input-placeholder-color-v1-kit09` | `gpui-base-0.9.1`, `gpui-component-0.9.1` | Apply the configured input placeholder color. |
| `gpui-toolbar-rtl-v1-kit09` | `gpui-base-0.9.1`, `gpui-component-0.9.1` | Mirror toolbar order and navigation for RTL. |
| `gpui-component-web-syntax-v1-kit09` | `gpui-component-0.9.1` | Retain native Rust/JSON highlighting, registry refresh and Diff spans. |
| `gpui-component-web-syntax-runtime-v1-kit08` | `gpui-component-0.9.1` | Retain native Rust/JSON highlighting, registry refresh and Diff spans. |
| `gpui-component-web-syntax-runtime-v2-kit08` | `gpui-component-0.9.1` | Retain native Rust/JSON highlighting, registry refresh and Diff spans. |
| `gpui-component-web-syntax-runtime-v3-kit08` | `gpui-component-0.9.1` | Retain native Rust/JSON highlighting, registry refresh and Diff spans. |
| `gpui-bidi-wrapped-rows-v1-fast` | `gpui-fast-0.1.14`, `gpui-fast-web-0.1.14`, `gpui-fast-wgpu-0.1.14` | Preserve Arabic/bidi hit testing and wrapped-row geometry. |
| `gpui-tree-aria-disabled-v1-fast` | `gpui-fast-0.1.14` | Expose disabled state on semantic/accessibility nodes. |
| `gpui-fast-accessibility-parent-click-v1-fast` | `gpui-fast-0.1.14` | Retain accessibility parent relationships and pointer activation. |
| `rustui-bidi-tab-layout-fast-v1` | `rustui-gpui-base` | Use Fast line-layout bidi data for tab stops. |
| `gpui-base-fast-platform-test-dep-v1` | `rustui-gpui-base` | Use the Fast platform for Base test-window helpers. |
| `gpui-base-fast-test-support-v1` | `rustui-gpui-base` | Enable Fast test support for Base tests. |
| `gpui-base-fast-test-support-path-v1` | `rustui-gpui-base` | Route Base test helpers through the pinned Fast platform path. |
| `rustui-bidi-tab-layout-test-fast-v1` | `rustui-gpui-base` | Adapt the bidi tab-layout regression test to the Fast backend. |
| `gpui-base-fast-feature-wiring-v1` | `rustui-gpui-base` | Forward Fast inspector and test-support features without activating the legacy core. |
| `t17-inert-focus-scope-v1-fast` | `gpui-fast-0.1.14` | Skip inert scopes during traversal and reject direct focus into them. |
| `t13-modal-focus-recovery-v1-fast` | `gpui-fast-0.1.14` | Keep modal focus recovery and traversal scoped correctly. |
| `a11y-grid-readonly-sort-v1-fast` | `gpui-fast-0.1.14` | Expose grid read-only and sorting accessibility properties. |
| `gpui-macos-anchored-popup-v1-fast` | `gpui-fast-0.1.14`, `gpui-fast-macos-0.1.14` | Provide opt-in native macOS anchored popups and placement. |
| `gpui-macos-popup-child-teardown-v1-fast` | `gpui-fast-macos-0.1.14` | Dismiss nested popup children safely on parent teardown. |
| `gpui-macos-popup-defer-frame-v1-fast` | `gpui-fast-macos-0.1.14` | Move native popup frames after releasing the platform state lock. |
| `gpui-test-support-a11y-activation-v1-fast` | `gpui-fast-0.1.14` | Enable test-window accessibility activation through the platform path. |
| `gpui-test-window-a11y-default-v1-fast` | `gpui-fast-0.1.14` | Enable accessibility-tree collection in test windows. |
| `gpui-fast-test-tree-disabled-v1-fast` | `gpui-fast-0.1.14` | Include disabled state and bounds in test-tree inspection. |
| `gpui-frame-recorder-v1-fast` | `gpui-fast-0.1.14` | Record opt-in frame CPU and presentation timing. |
| `gpui-frame-phases-v1-fast` | `gpui-fast-0.1.14` | Split opt-in frame timing into layout, prepaint, paint and submit. |
| `gpui-base-scroll-bounce-oracle-v1-fast` | `rustui-gpui-base` | Expose opt-in bounce observation for the scroll capture oracle. |
| `gpui-component-vendor-manifest-normalize-v1` | `gpui-component-0.9.1` | Reproduce the final normalized Component manifest as an alternative postimage proof. |
| `owned-fast-path-wiring-v1` | owned Fast core/platform/macOS/Apple and Kit Base/Component/assets | Use the owned Fast packages consistently, including Cargo’s canonical-source requirement for the platform’s target-specific core alias. |
| `kit-assets-fast-dev-platform-v1` | owned Kit assets | Use the Fast dev platform so the unused legacy macOS chain can leave the build. |

Owned source copies are reproduced by `python3 scripts/refresh_owned_fast_sources.py --apply`, checked with `--check`, and removed/restored with `--reverse`. `owned-fast-sources.json` pins every copied source/license file; manifests are changed only by the wiring patch.
