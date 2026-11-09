# Old stash review (S11)

## Scope and method

Review baseline: `origin/main` at `04e0d69d739052d92e12ee2d78bef36e3c1dff6c`. This is a static source review, not execution of archived code. The two saved stashes were compared with their own first parents; the presync untracked third-parent tree was also inventoried. T11 was compared with its pre-integration fork, found by `git merge-base 45fb4ccb^1 archive/t11-owned-gpui`, so the review includes both earlier T11c commits and the saved tip. Only `crates/`, `apps/`, `scripts/`, `patches/` and root `Cargo.toml` were reviewed. Code includes build metadata and process cache files under those roots so nothing there is silently omitted. Documentation, images, font/license assets and other paths were counted without content review.

Exact Git blobs establish literal matches. Non-identical sources were compared with the current implementations and later replacements; Rust formatting-only differences were checked through stable rustfmt on stdin, without modifying or building archived files. Each table row has exactly one class. Every IN MAIN destination is at the baseline above unless another path is named.

The older stash was not merged as a branch, but main's recovery commit `1ea81d78579d93cbb2d5e501e4aaaec80b617541` already imported 210 of its 222 reviewed files byte-for-byte. The remaining 12 recovery differences add validation/tests or replace old candidate inventory/compiler wiring; their retained behavior and later main changes were inspected. T8/G10 moved the compiled gallery into real modules; R15/R15b, R20 and R26 supersede older control/task/reload mechanics. Historical inclusion alone was not used to classify code that was subsequently retired.

## Permanent archives

All three lightweight tags were pushed and their advertised remote commit IDs matched their source refs before this review. These tags are retained when the source branches/stash entry are removed.

| Tag | Source commit | Own base |
|---|---|---|
| `archive/stash-2026-10-26` | `7068112cb487dfb531734633c8c9f2f84912689c` | `567f933c76b90fde3fcdda4ff45dc660df1eb217` |
| `archive/presync-20261108` | `6c177fa866ad44358e8fe9e12b710a85a0960ada` | `d8d3314e8269eafd6e70cd6618b0967333635942` |
| `archive/t11-owned-gpui` | `4ffcb860de5fcdcbd0129829f512d9195c022845` | `b9cbc60e80835fb0183175170b01f1325d66948c` |

The presync stash has three parents. `archive/presync-20261108^3` is `51d8c43062d115a5d069711e7a8e8cdfaca10009`; `git ls-tree -r` lists **24 untracked files**, including **20 graft cache files**. Of these, 12 cache files are under the reviewed code roots, four are under `docs/`, and eight files are outside the reviewed roots (four other graft caches, three compiler-probe executables and `.codex/config.toml`). The untracked parent remains reachable through the permanent tag.

## Summary

Counts below are per changed path, not per hunk. The image counts are disjoint from the documentation counts in these archives; no changed image is under `docs/`.

| Archive | All changed paths | Reviewed code/metadata | IN MAIN | OBSOLETE | USEFUL | Docs paths (under docs / elsewhere) | Images | Other assets | Outside scope |
|---|---:|---:|---:|---:|---:|---|---:|---:|---:|
| stash-2026-10-26 | 1,099 | 222 | 215 | 7 | 0 | 756 (723 / 33) | 87 | 8 | 26 |
| presync-20261108, including untracked | 24 | 12 | 0 | 12 | 0 | 4 (4 / 0) | 0 | 0 | 8 |
| t11-owned-gpui | 2,635 | 735 | 735 | 0 | 0 | 15 (0 / 15) | 1,835 | 19 | 31 |

## Stash from 2026-10-26

The code deltas are already integrated or superseded. At the current baseline 77 reviewed files have exact blob matches, including proposals moved into approved paths. Eleven more Rust files are identical after stable formatting. The obsolete rows below identify the old proposal revisions, retired standalone mutation, rejected candidate tooling and retired coordinator explicitly.

| Path | Class | Reason |
|---|---|---|
| `Cargo.toml` | IN MAIN | Owned GPUI path selection and stable defaults supersede the old source/profile wiring; see C2 bfbfab3c and the same current manifest. |
| `apps/rustui-playground/Cargo.toml` | IN MAIN | Dependencies are retained behind the public RustUI facade; the same manifest now also enables test support. |
| `apps/rustui-playground/locales/ar/lab.ftl` | IN MAIN | Same runtime catalog retained; existing message IDs remain and catalog notes/UI messages were extended. |
| `apps/rustui-playground/locales/en/lab.ftl` | IN MAIN | Same runtime catalog retained; existing message IDs remain and catalog notes/UI messages were extended. |
| `apps/rustui-playground/locales/fr/lab.ftl` | IN MAIN | Same runtime catalog retained; existing message IDs remain and catalog notes/UI messages were extended. |
| `apps/rustui-playground/package-app.sh` | IN MAIN | Reference packaging retained; T6 overwrite protection and transactional resource/notice packaging supersede the old writer at the same path. |
| `apps/rustui-playground/src/catalog.rs` | IN MAIN | Catalog retained at the same path; the Tree catalog case is now outline_view_catalog_describes_the_controlled_outline_story. |
| `apps/rustui-playground/src/lib.rs` | IN MAIN | Component/story filtering, owner bindings, reference opening and startup retained; localized UI and public component_assets replace old shell literals/direct assets. |
| `apps/rustui-playground/src/main.rs` | IN MAIN | Exact Git blob in main at `apps/rustui-playground/src/main.rs`. |
| `apps/rustui-playground/src/stories/alert_gallery.rs` | IN MAIN | Same story behavior is retained at this path; current localized labels, stable IDs and later demonstrations extend the implementation imported in 1ea81d78. |
| `apps/rustui-playground/src/stories/cache_gallery.rs` | IN MAIN | Exact Git blob in main at `apps/rustui-playground/src/stories/cache_gallery.rs`. |
| `apps/rustui-playground/src/stories/canvas.rs` | IN MAIN | Same story behavior is retained at this path; current localized labels, stable IDs and later demonstrations extend the implementation imported in 1ea81d78. |
| `apps/rustui-playground/src/stories/canvas_gallery.rs` | IN MAIN | Exact Git blob in main at `apps/rustui-playground/src/stories/canvas_gallery.rs`. |
| `apps/rustui-playground/src/stories/code_preview.rs` | IN MAIN | Same story behavior is retained at this path; current localized labels, stable IDs and later demonstrations extend the implementation imported in 1ea81d78. |
| `apps/rustui-playground/src/stories/compiled_preview_gallery.rs` | IN MAIN | T8 (38d60a08) and G10 (06820889) moved the registry/renderers/tests to stories/compiled_preview_gallery/ and graph modules; all 258 archived function names remain in the current Playground. |
| `apps/rustui-playground/src/stories/component_catalog.rs` | IN MAIN | Same story behavior is retained at this path; current localized labels, stable IDs and later demonstrations extend the implementation imported in 1ea81d78. |
| `apps/rustui-playground/src/stories/dialog_gallery.rs` | IN MAIN | Same story behavior is retained at this path; current localized labels, stable IDs and later demonstrations extend the implementation imported in 1ea81d78. |
| `apps/rustui-playground/src/stories/event_inspector.rs` | IN MAIN | The same path retains bounded ordered history, totals and clear; EventEntry/view and localized messages replace concatenated summary strings. |
| `apps/rustui-playground/src/stories/form_accessories_gallery.rs` | IN MAIN | Same story behavior is retained at this path; current localized labels, stable IDs and later demonstrations extend the implementation imported in 1ea81d78. |
| `apps/rustui-playground/src/stories/foundation_demos.rs` | IN MAIN | Same story behavior is retained at this path; current localized labels, stable IDs and later demonstrations extend the implementation imported in 1ea81d78. |
| `apps/rustui-playground/src/stories/hover_gallery.rs` | IN MAIN | Same story behavior is retained at this path; current localized labels, stable IDs and later demonstrations extend the implementation imported in 1ea81d78. |
| `apps/rustui-playground/src/stories/keyboard_gallery.rs` | IN MAIN | Same story behavior is retained at this path; current localized labels, stable IDs and later demonstrations extend the implementation imported in 1ea81d78. |
| `apps/rustui-playground/src/stories/locale_selector.rs` | IN MAIN | Same story behavior is retained at this path; current localized labels, stable IDs and later demonstrations extend the implementation imported in 1ea81d78. |
| `apps/rustui-playground/src/stories/localization_gallery.rs` | IN MAIN | Same story behavior is retained at this path; current localized labels, stable IDs and later demonstrations extend the implementation imported in 1ea81d78. |
| `apps/rustui-playground/src/stories/menu_bar_reference_gallery.rs` | IN MAIN | Exact Git blob in main at `apps/rustui-playground/src/stories/menu_bar_reference_gallery.rs`. |
| `apps/rustui-playground/src/stories/mod.rs` | IN MAIN | Same story behavior is retained at this path; current localized labels, stable IDs and later demonstrations extend the implementation imported in 1ea81d78. |
| `apps/rustui-playground/src/stories/navigation_split_gallery.rs` | IN MAIN | Exact Git blob in main at `apps/rustui-playground/src/stories/navigation_split_gallery.rs`. |
| `apps/rustui-playground/src/stories/offset_gallery.rs` | IN MAIN | Same source at this path after stable rustfmt; differences are formatting only. |
| `apps/rustui-playground/src/stories/owner_tasks.rs` | IN MAIN | Same story behavior is retained at this path; current localized labels, stable IDs and later demonstrations extend the implementation imported in 1ea81d78. |
| `apps/rustui-playground/src/stories/primitive_gallery.rs` | IN MAIN | Exact Git blob in main at `apps/rustui-playground/src/stories/primitive_gallery.rs`. |
| `apps/rustui-playground/src/stories/semantics_gallery.rs` | IN MAIN | Exact Git blob in main at `apps/rustui-playground/src/stories/semantics_gallery.rs`. |
| `apps/rustui-playground/src/stories/sheet_gallery.rs` | IN MAIN | Same story behavior is retained at this path; current localized labels, stable IDs and later demonstrations extend the implementation imported in 1ea81d78. |
| `apps/rustui-playground/src/stories/sidebar_gallery.rs` | IN MAIN | Exact Git blob in main at `apps/rustui-playground/src/stories/sidebar_gallery.rs`. |
| `apps/rustui-playground/src/stories/supplementary_gallery.rs` | IN MAIN | Same story behavior is retained at this path; current localized labels, stable IDs and later demonstrations extend the implementation imported in 1ea81d78. |
| `apps/rustui-playground/src/stories/tab_bar_reference_gallery.rs` | IN MAIN | Exact Git blob in main at `apps/rustui-playground/src/stories/tab_bar_reference_gallery.rs`. |
| `apps/rustui-playground/src/stories/task_release.rs` | IN MAIN | Same story behavior is retained at this path; current localized labels, stable IDs and later demonstrations extend the implementation imported in 1ea81d78. |
| `apps/rustui-playground/src/stories/text_specimen_gallery.rs` | IN MAIN | Exact Git blob in main at `apps/rustui-playground/src/stories/text_specimen_gallery.rs`. |
| `apps/rustui-playground/src/stories/text_style_gallery.rs` | IN MAIN | Exact Git blob in main at `apps/rustui-playground/src/stories/text_style_gallery.rs`. |
| `apps/rustui-playground/src/stories/toolbar_gallery.rs` | IN MAIN | Same story behavior is retained at this path; current localized labels, stable IDs and later demonstrations extend the implementation imported in 1ea81d78. |
| `apps/rustui-playground/src/stories/tree_reconciliation_gallery.rs` | IN MAIN | Same story behavior is retained at this path; current localized labels, stable IDs and later demonstrations extend the implementation imported in 1ea81d78. |
| `apps/rustui-playground/src/stories/utility_window_gallery.rs` | IN MAIN | Same story behavior is retained at this path; current localized labels, stable IDs and later demonstrations extend the implementation imported in 1ea81d78. |
| `apps/rustui-playground/src/stories/window_chrome_gallery.rs` | IN MAIN | Exact Git blob in main at `apps/rustui-playground/src/stories/window_chrome_gallery.rs`. |
| `apps/rustui-playground/src/stories/window_controls_gallery.rs` | IN MAIN | Same source at this path after stable rustfmt; differences are formatting only. |
| `apps/rustui-playground/web/index.html` | IN MAIN | Exact Git blob in main at `apps/rustui-playground/web/index.html`. |
| `crates/rustui-core/src/actions.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-core/src/actions.rs`. |
| `crates/rustui-core/src/binding.rs` | IN MAIN | The same Binding implementation remains; current comments clarify captured snapshots versus owner updates. |
| `crates/rustui-i18n/src/catalog.rs` | IN MAIN | Same catalog/locale/formatting contract remains; later fallback and validation regressions extend the implementation imported in 1ea81d78. |
| `crates/rustui-i18n/src/direction.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-i18n/src/direction.rs`. |
| `crates/rustui-i18n/src/fallback.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-i18n/src/fallback.rs`. |
| `crates/rustui-i18n/src/formatting.rs` | IN MAIN | Same catalog/locale/formatting contract remains; later fallback and validation regressions extend the implementation imported in 1ea81d78. |
| `crates/rustui-i18n/src/lib.rs` | IN MAIN | Same catalog/locale/formatting contract remains; later fallback and validation regressions extend the implementation imported in 1ea81d78. |
| `crates/rustui-i18n/src/locale.rs` | IN MAIN | Same catalog/locale/formatting contract remains; later fallback and validation regressions extend the implementation imported in 1ea81d78. |
| `crates/rustui-i18n/src/message.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-i18n/src/message.rs`. |
| `crates/rustui-i18n/src/validation.rs` | IN MAIN | Same catalog/locale/formatting contract remains; later fallback and validation regressions extend the implementation imported in 1ea81d78. |
| `crates/rustui-i18n/tests/catalog_contract.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-i18n/tests/catalog_contract.rs`. |
| `crates/rustui-macros/src/component.rs` | IN MAIN | Conditional defaults retained; the same macro now uses associated field types to handle cfg-removed and Self-referential fields. |
| `crates/rustui-macros/src/view_builder.rs` | IN MAIN | Same source at this path after stable rustfmt; differences are formatting only. |
| `crates/rustui-macros/tests/conditional_default.rs` | IN MAIN | Existing conditional-default cases retained at the same path, with additional unavailable-type/Self regressions. |
| `crates/rustui/examples/cache_comparison.rs` | IN MAIN | Same source at this path after stable rustfmt; differences are formatting only. |
| `crates/rustui/examples/m49_runtime_action_dispatch_probe.rs` | IN MAIN | Exact Git blob in main at `crates/rustui/examples/m49_runtime_action_dispatch_probe.rs`. |
| `crates/rustui/examples/m532_component_host_root_probe.rs` | IN MAIN | Exact Git blob in main at `crates/rustui/examples/m532_component_host_root_probe.rs`. |
| `crates/rustui/examples/support/spring_probe.rs` | IN MAIN | Archived source lines are retained at this path; current main only adds code or reformats them. |
| `crates/rustui/examples/view_composition.rs` | IN MAIN | Exact Git blob in main at `crates/rustui/examples/view_composition.rs`. |
| `crates/rustui/src/accessibility/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui/src/accessibility/mod.rs`. |
| `crates/rustui/src/accessibility/semantics.rs` | IN MAIN | Same source at this path after stable rustfmt; differences are formatting only. |
| `crates/rustui/src/adapters/themes.rs` | IN MAIN | Exact Git blob in main at `crates/rustui/src/adapters/themes.rs`. |
| `crates/rustui/src/animation/offset.rs` | IN MAIN | Exact Git blob in main at `crates/rustui/src/animation/offset.rs`. |
| `crates/rustui/src/animation/spring.rs` | IN MAIN | Exact Git blob in main at `crates/rustui/src/animation/spring.rs`. |
| `crates/rustui/src/components/advanced/code_editor.rs` | IN MAIN | The same retained EditorState, owner writer, language and line-number configuration remain; later focus/disabled synchronization extends them. |
| `crates/rustui/src/components/advanced/property_inspector.rs` | IN MAIN | The same keyed property editors remain; integer/choice owner subscriptions and disabled recovery were strengthened. |
| `crates/rustui/src/components/basic/avatar.rs` | IN MAIN | Exact Git blob in main at `crates/rustui/src/components/basic/avatar.rs`. |
| `crates/rustui/src/components/basic/badge.rs` | IN MAIN | Same source at this path after stable rustfmt; differences are formatting only. |
| `crates/rustui/src/components/basic/button.rs` | IN MAIN | Archived source lines are retained at this path; current main only adds code or reformats them. |
| `crates/rustui/src/components/basic/divider.rs` | IN MAIN | Same source at this path after stable rustfmt; differences are formatting only. |
| `crates/rustui/src/components/basic/icon.rs` | IN MAIN | Exact Git blob in main at `crates/rustui/src/components/basic/icon.rs`. |
| `crates/rustui/src/components/basic/image.rs` | IN MAIN | Exact Git blob in main at `crates/rustui/src/components/basic/image.rs`. |
| `crates/rustui/src/components/basic/label.rs` | IN MAIN | Exact Git blob in main at `crates/rustui/src/components/basic/label.rs`. |
| `crates/rustui/src/components/basic/link.rs` | IN MAIN | Archived source lines are retained at this path; current main only adds code or reformats them. |
| `crates/rustui/src/components/basic/text.rs` | IN MAIN | Archived source lines are retained at this path; current main only adds code or reformats them. |
| `crates/rustui/src/components/data/collection.rs` | IN MAIN | Collection selection/virtualization retained; the same path now guards deferred clearing if the missing item is restored. |
| `crates/rustui/src/components/data/data_grid.rs` | IN MAIN | Table semantics and explicit grid/table IDs retained; stable implicit IDs replace label/data-derived IDs at the same path. |
| `crates/rustui/src/components/data/keyed_list.rs` | IN MAIN | Stable-ID reconciliation, filtering and selection retained; the same path adds current owner/focus/task guards. |
| `crates/rustui/src/components/data/list.rs` | IN MAIN | List ownership/filtering retained; stable implicit IDs, localized labels and disabled/focus handling extend the same path. |
| `crates/rustui/src/components/data/mod.rs` | IN MAIN | Live data exports retained; T10 removed only the old outline planning scaffold; the actual hierarchy lives in tree.rs. |
| `crates/rustui/src/components/data/table.rs` | IN MAIN | Retained table/delegate, row selection and semantic rows remain at the same path; identity and deferred owner writes were strengthened. |
| `crates/rustui/src/components/data/tree.rs` | IN MAIN | Stable hierarchy, expansion and disabled-item navigation remain; enabled_tree_index replaces next_enabled_tree_index and R15 adds live focus/owner reconciliation. |
| `crates/rustui/src/components/feedback/empty_state.rs` | IN MAIN | Archived source lines are retained at this path; current main only adds code or reformats them. |
| `crates/rustui/src/components/feedback/error_state.rs` | IN MAIN | Archived source lines are retained at this path; current main only adds code or reformats them. |
| `crates/rustui/src/components/feedback/mod.rs` | IN MAIN | Archived source lines are retained at this path; current main only adds code or reformats them. |
| `crates/rustui/src/components/feedback/progress.rs` | IN MAIN | Exact Git blob in main at `crates/rustui/src/components/feedback/progress.rs`. |
| `crates/rustui/src/components/feedback/skeleton.rs` | IN MAIN | Exact Git blob in main at `crates/rustui/src/components/feedback/skeleton.rs`. |
| `crates/rustui/src/components/feedback/spinner.rs` | IN MAIN | Exact Git blob in main at `crates/rustui/src/components/feedback/spinner.rs`. |
| `crates/rustui/src/components/input/checkbox.rs` | IN MAIN | Same controlled input and owner bridge remain; R15/R15b disabled/focus/subscription handling extends the implementation at this path. |
| `crates/rustui/src/components/input/color_picker.rs` | IN MAIN | Same controlled input and owner bridge remain; R15/R15b disabled/focus/subscription handling extends the implementation at this path. |
| `crates/rustui/src/components/input/combo_box.rs` | IN MAIN | Same controlled input and owner bridge remain; R15/R15b disabled/focus/subscription handling extends the implementation at this path. |
| `crates/rustui/src/components/input/date_picker.rs` | IN MAIN | Same controlled input and owner bridge remain; R15/R15b disabled/focus/subscription handling extends the implementation at this path. |
| `crates/rustui/src/components/input/mod.rs` | IN MAIN | Same controlled input and owner bridge remain; R15/R15b disabled/focus/subscription handling extends the implementation at this path. |
| `crates/rustui/src/components/input/picker.rs` | IN MAIN | Same controlled input and owner bridge remain; R15/R15b disabled/focus/subscription handling extends the implementation at this path. |
| `crates/rustui/src/components/input/radio.rs` | IN MAIN | Same controlled input and owner bridge remain; R15/R15b disabled/focus/subscription handling extends the implementation at this path. |
| `crates/rustui/src/components/input/search_field.rs` | IN MAIN | Exact Git blob in main at `crates/rustui/src/components/input/search_field.rs`. |
| `crates/rustui/src/components/input/slider.rs` | IN MAIN | Slider state/range/step and accessible actions retained; handle_slider_accessibility_action centralizes the handlers under P2b (4a627cd3). |
| `crates/rustui/src/components/input/stepper.rs` | IN MAIN | Same controlled input and owner bridge remain; R15/R15b disabled/focus/subscription handling extends the implementation at this path. |
| `crates/rustui/src/components/input/text_editor.rs` | IN MAIN | Same controlled input and owner bridge remain; R15/R15b disabled/focus/subscription handling extends the implementation at this path. |
| `crates/rustui/src/components/input/text_field.rs` | IN MAIN | Same controlled input and owner bridge remain; R15/R15b disabled/focus/subscription handling extends the implementation at this path. |
| `crates/rustui/src/components/input/toggle.rs` | IN MAIN | Same controlled input and owner bridge remain; R15/R15b disabled/focus/subscription handling extends the implementation at this path. |
| `crates/rustui/src/components/input/token_field.rs` | IN MAIN | Token editing retained; remove_matching_token with binding_updater replaces stale snapshot removal, preserving current owner edits. |
| `crates/rustui/src/components/mod.rs` | IN MAIN | Archived source lines are retained at this path; current main only adds code or reformats them. |
| `crates/rustui/src/components/navigation/breadcrumb.rs` | IN MAIN | Exact Git blob in main at `crates/rustui/src/components/navigation/breadcrumb.rs`. |
| `crates/rustui/src/components/navigation/disclosure.rs` | IN MAIN | Controlled disclosure remains at the same path; current localized naming and disabled focus handling extend it. |
| `crates/rustui/src/components/navigation/navigation_split.rs` | IN MAIN | Narrow/detail routing retained at the same path; shared label resolution and localized titles replace duplicate fallback code. |
| `crates/rustui/src/components/navigation/navigation_stack.rs` | IN MAIN | Navigation path/back handling remains at the same path; current localized messages and labels extend it. |
| `crates/rustui/src/components/navigation/sidebar.rs` | IN MAIN | Sidebar retention and navigation retained; R15 makes the same focus handle leave disabled rows and applies the active theme surface. |
| `crates/rustui/src/components/navigation/tab_view.rs` | IN MAIN | Controlled tab selection/roving focus retained; stable implicit IDs and R15 disabled-tab recovery extend the same path. |
| `crates/rustui/src/components/navigation/toolbar.rs` | IN MAIN | Toolbar geometry/actions remain at the same path; active theme surfaces and disabled focus handling extend it. |
| `crates/rustui/src/components/overlay/alert.rs` | IN MAIN | Exact Git blob in main at `crates/rustui/src/components/overlay/alert.rs`. |
| `crates/rustui/src/components/overlay/dialog.rs` | IN MAIN | Dialog ownership and semantic content retained; native title forwarding, current content props and focus recovery extend the same path. |
| `crates/rustui/src/components/overlay/menu.rs` | IN MAIN | Menu actions/semantics retained; current popup item descriptors and R15b disabled/live selection recovery replace stale closures. |
| `crates/rustui/src/components/overlay/popover.rs` | IN MAIN | Same source at this path after stable rustfmt; differences are formatting only. |
| `crates/rustui/src/components/overlay/sheet.rs` | IN MAIN | Sheet content ownership and labels retained; current props and semantic surfaces extend the same path. |
| `crates/rustui/src/components/overlay/toast.rs` | IN MAIN | In-app notification actions/live announcements retained; current localized payload construction extends the same path. |
| `crates/rustui/src/components/overlay/tooltip.rs` | IN MAIN | Exact Git blob in main at `crates/rustui/src/components/overlay/tooltip.rs`. |
| `crates/rustui/src/components/theme.rs` | IN MAIN | Archived source lines are retained at this path; current main only adds code or reformats them. |
| `crates/rustui/src/interaction/focus.rs` | IN MAIN | Exact Git blob in main at `crates/rustui/src/interaction/focus.rs`. |
| `crates/rustui/src/layout/grid.rs` | IN MAIN | Same source at this path after stable rustfmt; differences are formatting only. |
| `crates/rustui/src/layout/hstack.rs` | IN MAIN | Horizontal layout remains at the same path; children now receive explicit retention scopes. |
| `crates/rustui/src/layout/lazy_grid.rs` | IN MAIN | Same source at this path after stable rustfmt; differences are formatting only. |
| `crates/rustui/src/layout/mod.rs` | IN MAIN | Live layout exports retained; T10 removed only planning scaffolds. |
| `crates/rustui/src/layout/scroll_view.rs` | IN MAIN | Native scroll ownership, axes and diagnostics retained; stable scoped IDs and offset restoration strengthen the same implementation. |
| `crates/rustui/src/layout/split_view.rs` | IN MAIN | Archived source lines are retained at this path; current main only adds code or reformats them. |
| `crates/rustui/src/layout/vstack.rs` | IN MAIN | Vertical layout remains at the same path; children now receive explicit retention scopes. |
| `crates/rustui/src/lib.rs` | IN MAIN | Public RustUI modules retained; owned GPUI facade and current test declarations extend exports. |
| `crates/rustui/src/localization/mod.rs` | IN MAIN | Archived source lines are retained at this path; current main only adds code or reformats them. |
| `crates/rustui/src/modifiers/opacity.rs` | IN MAIN | Native opacity animation retained at the same path; inherited environment motion preferences extend configuration. |
| `crates/rustui/src/prelude.rs` | IN MAIN | Existing public controls remain exported; newer controls and owned facade additions extend the prelude. |
| `crates/rustui/src/preview/codec.rs` | IN MAIN | Validated preview codec and integer normalization retained; current checked encoding and composition codec extend the same path. |
| `crates/rustui/src/preview/host.rs` | IN MAIN | Exact Git blob in main at `crates/rustui/src/preview/host.rs`. |
| `crates/rustui/src/preview/model.rs` | IN MAIN | Existing preview contracts remain at the same path; composition-specific errors were added. |
| `crates/rustui/src/preview/schema.rs` | IN MAIN | Schema validation retained; current registration regressions and validated-catalog reuse extend the same path. |
| `crates/rustui/src/runtime/application.rs` | IN MAIN | Runtime/font/assets/window startup retained; public component_assets and later window/focus defaults extend the same path. |
| `crates/rustui/src/runtime/component_host.rs` | IN MAIN | Retained entities/scopes/subscriptions remain; current replaceable subscriptions and lifecycle tests extend the same path. |
| `crates/rustui/src/runtime/mod.rs` | IN MAIN | Live runtime exports remain; T10 removed the unused invalidation scaffold and later native popup exports were added. |
| `crates/rustui/src/runtime/scoped_element.rs` | IN MAIN | Exact Git blob in main at `crates/rustui/src/runtime/scoped_element.rs`. |
| `crates/rustui/src/runtime/tasks.rs` | IN MAIN | OwnerTask retained at the same path; R20 adds cancellation/generation/error and owner-release coverage. |
| `crates/rustui/src/theme/loader.rs` | IN MAIN | Validated loading remains; the same parser now rejects duplicate keys before typed deserialization. |
| `crates/rustui/src/theme/mod.rs` | IN MAIN | Theme functionality retained; T16 implements theme tokens in tokens.rs in place of the old planning scaffold/public-module export. |
| `crates/rustui/src/theme/registry.rs` | IN MAIN | Activation and component styles retained; current revision/cache handling and opaque popover fallback supersede uncached resolution. |
| `crates/rustui/src/theme/resolver.rs` | IN MAIN | Component rules/capabilities retained; record reuse and capability reporting extend the same resolver. |
| `crates/rustui/src/theme/schema.rs` | IN MAIN | Same source at this path after stable rustfmt; differences are formatting only. |
| `crates/rustui/src/theme/watcher.rs` | IN MAIN | Reload/last-good theme behavior retained; current validated activation and R26 recovery/unmount coverage extend the same path. |
| `crates/rustui/src/view.rs` | IN MAIN | Binding, keyed retention and render context remain; current scoped identities/owner tasks/localized messages extend the same path. |
| `crates/rustui/tests/component_state.rs` | IN MAIN | Exact Git blob in main at `crates/rustui/tests/component_state.rs`. |
| `crates/rustui/tests/hstack_macro_turbofish_optional.rs` | IN MAIN | Exact Git blob in main at `crates/rustui/tests/hstack_macro_turbofish_optional.rs`. |
| `crates/rustui/tests/menu_preview_host_integration.rs` | IN MAIN | Exact Git blob in main at `crates/rustui/tests/menu_preview_host_integration.rs`. |
| `crates/rustui/tests/preview_host_render_lifecycle.rs` | IN MAIN | Archived source lines are retained at this path; current main only adds code or reformats them. |
| `crates/rustui/tests/theme_profile_appearance.rs` | IN MAIN | Exact Git blob in main at `crates/rustui/tests/theme_profile_appearance.rs`. |
| `patches/approved/gpui-component-date-picker-accessibility-v1.patch` | IN MAIN | Exact Git blob in main at `patches/approved/gpui-component-date-picker-accessibility-v1.patch`. |
| `patches/approved/gpui-pre-web-keyboard-focus-activation-v3.patch` | IN MAIN | Exact Git blob in main at `patches/approved/gpui-pre-web-keyboard-focus-activation-v3.patch`. |
| `patches/approved/gpui-tree-aria-disabled-v1.patch` | IN MAIN | Exact Git blob in main at `patches/approved/gpui-tree-aria-disabled-v1.patch`. |
| `patches/proposed/gpui-component-sheet-close-accessibility-v1.patch` | OBSOLETE | P2 replaced this proposal with patches/approved/gpui-component-sheet-close-accessibility-v2.patch (b5658fdd). |
| `patches/proposed/gpui-component-slider-accessibility-label.patch` | OBSOLETE | P2b replaced this proposal with patches/approved/gpui-component-slider-accessibility-label-v2.patch (4a627cd3). |
| `patches/proposed/gpui-component-tree-accessible-name-v1.patch` | IN MAIN | Exact Git blob in main at `patches/approved/gpui-component-tree-accessible-name-v1.patch`. |
| `patches/proposed/gpui-pre-web-keyboard-focus-activation-v1.patch` | IN MAIN | Exact Git blob in main at `patches/proposed/gpui-pre-web-keyboard-focus-activation-v1.patch`. |
| `patches/proposed/gpui-pre-web-keyboard-focus-activation-v2.patch` | IN MAIN | Exact Git blob in main at `patches/proposed/gpui-pre-web-keyboard-focus-activation-v2.patch`. |
| `patches/proposed/gpui-pre-web-keyboard-focus-activation-v3.patch` | IN MAIN | Exact Git blob in main at `patches/proposed/gpui-pre-web-keyboard-focus-activation-v3.patch`. |
| `patches/proposed/gpui-tree-aria-disabled.patch` | IN MAIN | Exact Git blob in main at `patches/approved/gpui-tree-aria-disabled-v1.patch`. |
| `patches/proposed/gpui-tree-aria-level-v1.patch` | IN MAIN | Promoted to patches/approved/gpui-tree-aria-level-v1.patch; the only patch-body adjustment fixes the test-tree map shape; replay belongs to the three-source series (P0). |
| `patches/proposed/gpui-tree-disabled-keyboard-v1.patch` | OBSOLETE | P2 replaced this proposal with patches/approved/gpui-tree-disabled-keyboard-v2.patch (b1e01110). |
| `scripts/aggregate_candidate_evidence.py` | IN MAIN | Exact Git blob in main at `scripts/aggregate_candidate_evidence.py`. |
| `scripts/apply_candidate_patch.py` | IN MAIN | Same patch application retained; PinnedDir.inventory replaces duplicate recursive inventory with bounded descriptor-based traversal. |
| `scripts/apply_gpui_component_date_picker_patch.py` | IN MAIN | Same apply entry point retained; approved date-picker-localization-v2 and composed source images supersede v1-only validation. |
| `scripts/apply_gpui_tree_aria_disabled_patch.py` | IN MAIN | Same baseline checker retained; current checks compose with the approved Tree accessibility series. |
| `scripts/apply_gpui_tree_aria_level_patch.py` | OBSOLETE | Standalone mutation was retired by P0 (7194bb90); the same path is now a read-only compatibility entry point for apply_gpui_tree_accessibility_series.py. |
| `scripts/apply_gpui_web_keyboard_focus_activation_patch.py` | IN MAIN | Same entry point and archived behavior remain; later source-stack/artifact integration extends this path (static comparison only). |
| `scripts/build-playground-web.sh` | IN MAIN | Same entry point and archived behavior remain; later source-stack/artifact integration extends this path (static comparison only). |
| `scripts/calibrate_cache_workload.py` | IN MAIN | Exact Git blob in main at `scripts/calibrate_cache_workload.py`. |
| `scripts/cargo-headstart` | IN MAIN | Optional compiler wrapper retained at the same path; explicit compiler/remap/popup support extend it. Stable daily aliases are governed by G11 (da64f0b9). |
| `scripts/check_gpui_component_slider_accessibility_label_patch.py` | IN MAIN | Same check entry point now delegates to apply_gpui_component_slider_accessibility_label_v2_patch.py (P2b); equivalent purpose retained. |
| `scripts/check_gpui_tree_disabled_keyboard_patch.py` | IN MAIN | Same check entry point now delegates to the approved v2 manifest/replay helper; equivalent purpose retained. |
| `scripts/check_gpui_web_keyboard_focus_activation_patch.py` | IN MAIN | Exact Git blob in main at `scripts/check_gpui_web_keyboard_focus_activation_patch.py`. |
| `scripts/compare_cache_workload.py` | IN MAIN | Same comparison retained; current provenance accepts both rooted and legacy absolute binary names while validating the same artifact. |
| `scripts/compare_candidate_sources.py` | IN MAIN | Exact Git blob in main at `scripts/compare_candidate_sources.py`. |
| `scripts/p3_candidate_fs.py` | IN MAIN | Same pinned-filesystem implementation retained; explicit hard-link policy and race handling extend it. |
| `scripts/prepare_dialog_focus_rebase_candidate.py` | IN MAIN | Exact Git blob in main at `scripts/prepare_dialog_focus_rebase_candidate.py`. |
| `scripts/prepare_quad_sort_candidate.py` | OBSOLETE | Rejected candidate generator removed by P1 (7fc7aff2); T20 later closed the experiment without promotion (14215cf6). |
| `scripts/record_build_provenance.py` | IN MAIN | Same recorder retained; compiler lookup is now skipped for a missing binary. |
| `scripts/report_candidate_sources.py` | IN MAIN | Exact Git blob in main at `scripts/report_candidate_sources.py`. |
| `scripts/roadmap_session_coordinator.py` | OBSOLETE | Retired session/claim coordination, removed by H0 (e1ef2373); AGENTS.md and docs/TICKETS.md replace that process. |
| `scripts/run_candidate_pipeline.py` | IN MAIN | Exact Git blob in main at `scripts/run_candidate_pipeline.py`. |
| `scripts/tests/test_aggregate_candidate_evidence.py` | IN MAIN | Exact Git blob in main at `scripts/tests/test_aggregate_candidate_evidence.py`. |
| `scripts/tests/test_apply_candidate_patch.py` | IN MAIN | Same regressions retained; the two renamed fresh-copy/reused-ID tests additionally assert patch-cache cleanup; cleanup-failure coverage remains. |
| `scripts/tests/test_build_playground_web.sh` | IN MAIN | Archived regression cases remain at this path; current fixtures and additional cases track the stronger current implementation. |
| `scripts/tests/test_calibrate_cache_workload.py` | IN MAIN | Exact Git blob in main at `scripts/tests/test_calibrate_cache_workload.py`. |
| `scripts/tests/test_cargo_headstart_candidate_target.py` | IN MAIN | Exact Git blob in main at `scripts/tests/test_cargo_headstart_candidate_target.py`. |
| `scripts/tests/test_compare_cache_workload.py` | IN MAIN | Archived regression cases remain at this path; current fixtures and additional cases track the stronger current implementation. |
| `scripts/tests/test_compare_candidate_sources.py` | IN MAIN | Exact Git blob in main at `scripts/tests/test_compare_candidate_sources.py`. |
| `scripts/tests/test_p3_candidate_fs.py` | IN MAIN | Exact Git blob in main at `scripts/tests/test_p3_candidate_fs.py`. |
| `scripts/tests/test_package_playground_app.sh` | IN MAIN | Same bundle/resource regressions retained; T6 overwrite protection, rollback and notice cases extend the suite. |
| `scripts/tests/test_prepare_dialog_focus_rebase_candidate.py` | IN MAIN | Exact Git blob in main at `scripts/tests/test_prepare_dialog_focus_rebase_candidate.py`. |
| `scripts/tests/test_prepare_quad_sort_candidate.py` | OBSOLETE | Tests for the rejected generator removed by P1 (7fc7aff2); no active generator remains to recover. |
| `scripts/tests/test_record_build_provenance.py` | IN MAIN | Archived regression cases remain at this path; current fixtures and additional cases track the stronger current implementation. |
| `scripts/tests/test_report_candidate_sources.py` | IN MAIN | Exact Git blob in main at `scripts/tests/test_report_candidate_sources.py`. |
| `scripts/tests/test_run_candidate_pipeline.py` | IN MAIN | Archived regression cases remain at this path; current fixtures and additional cases track the stronger current implementation. |
| `scripts/tests/test_validate_candidate_evidence.py` | IN MAIN | Exact Git blob in main at `scripts/tests/test_validate_candidate_evidence.py`. |
| `scripts/tests/test_validate_candidate_snapshot.py` | IN MAIN | Archived regression cases remain at this path; current fixtures and additional cases track the stronger current implementation. |
| `scripts/tests/test_validate_locales.py` | IN MAIN | Archived regression cases remain at this path; current fixtures and additional cases track the stronger current implementation. |
| `scripts/tests/test_validate_p3d_visual_evidence.py` | IN MAIN | Exact Git blob in main at `scripts/tests/test_validate_p3d_visual_evidence.py`. |
| `scripts/tests/test_validate_patch_plan.py` | IN MAIN | Exact Git blob in main at `scripts/tests/test_validate_patch_plan.py`. |
| `scripts/tests/test_validate_quad_sort_performance.py` | IN MAIN | Exact Git blob in main at `scripts/tests/test_validate_quad_sort_performance.py`. |
| `scripts/tests/test_validate_themes.py` | IN MAIN | Exact Git blob in main at `scripts/tests/test_validate_themes.py`. |
| `scripts/tests/test_verify_dependency_boundary.py` | IN MAIN | Archived regression cases remain at this path; current fixtures and additional cases track the stronger current implementation. |
| `scripts/tests/test_verify_dependency_reuse.py` | IN MAIN | Exact Git blob in main at `scripts/tests/test_verify_dependency_reuse.py`. |
| `scripts/tests/test_verify_theme_adapter_coverage.py` | IN MAIN | Archived regression cases remain at this path; current fixtures and additional cases track the stronger current implementation. |
| `scripts/validate_candidate_evidence.py` | IN MAIN | Exact Git blob in main at `scripts/validate_candidate_evidence.py`. |
| `scripts/validate_candidate_snapshot.py` | IN MAIN | Same validator retained; hard-link/change diagnostics are ordered more precisely. |
| `scripts/validate_locales.py` | IN MAIN | Same catalog metadata validation retained; current optional named-runtime validation extends it. |
| `scripts/validate_p3d_visual_evidence.py` | IN MAIN | Exact Git blob in main at `scripts/validate_p3d_visual_evidence.py`. |
| `scripts/validate_patch_plan.py` | IN MAIN | Exact Git blob in main at `scripts/validate_patch_plan.py`. |
| `scripts/validate_quad_sort_performance.py` | IN MAIN | Exact Git blob in main at `scripts/validate_quad_sort_performance.py`. |
| `scripts/validate_themes.py` | IN MAIN | Exact Git blob in main at `scripts/validate_themes.py`. |
| `scripts/verify_dependency_boundary.py` | IN MAIN | Same dependency guard retained; locked/offline metadata and public-facade boundaries extend it. |
| `scripts/verify_dependency_reuse.py` | IN MAIN | Exact Git blob in main at `scripts/verify_dependency_reuse.py`. |
| `scripts/verify_theme_adapter_coverage.py` | IN MAIN | Same source-consumption guard retained; menu activation-to-popover and slider contrast tracing extend it. |

## Owner presync stash and backups

The tracked worktree and index trees equal the stash's own base: `git diff archive/presync-20261108^1 archive/presync-20261108` is empty. Its only reviewed entries are the 12 generated cache files in the third parent.

The backup directory `~/.codex/backups/owner-source-cleanup-20261108-oamqjnqa` is retained unchanged. Its recorded repo HEAD equals the stash base, both `repo-index.diff` and `repo-worktree.diff` are zero bytes, and `repo-untracked.tar.gz` contains exactly the same 24 file paths as the tagged untracked parent. All 20 non-documentation file blobs match; the four docs paths were counted without content review. The bundle and all other backup files are retained.

| Path | Class | Reason |
|---|---|---|
| `apps/rustui-playground/graft/.cache/telemetry-repo-id.json` | OBSOLETE | Generated graft telemetry/branch cache, not RustUI runtime code; retired process data with no current tracked module or build consumer. |
| `apps/rustui-playground/graft/.cache/trail-branches.json` | OBSOLETE | Generated graft telemetry/branch cache, not RustUI runtime code; retired process data with no current tracked module or build consumer. |
| `crates/graft/.cache/telemetry-repo-id.json` | OBSOLETE | Generated graft telemetry/branch cache, not RustUI runtime code; retired process data with no current tracked module or build consumer. |
| `crates/graft/.cache/trail-branches.json` | OBSOLETE | Generated graft telemetry/branch cache, not RustUI runtime code; retired process data with no current tracked module or build consumer. |
| `crates/rustui-macros/graft/.cache/telemetry-repo-id.json` | OBSOLETE | Generated graft telemetry/branch cache, not RustUI runtime code; retired process data with no current tracked module or build consumer. |
| `crates/rustui-macros/graft/.cache/trail-branches.json` | OBSOLETE | Generated graft telemetry/branch cache, not RustUI runtime code; retired process data with no current tracked module or build consumer. |
| `crates/rustui/graft/.cache/telemetry-repo-id.json` | OBSOLETE | Generated graft telemetry/branch cache, not RustUI runtime code; retired process data with no current tracked module or build consumer. |
| `crates/rustui/graft/.cache/trail-branches.json` | OBSOLETE | Generated graft telemetry/branch cache, not RustUI runtime code; retired process data with no current tracked module or build consumer. |
| `scripts/graft/.cache/telemetry-repo-id.json` | OBSOLETE | Generated graft telemetry/branch cache, not RustUI runtime code; retired process data with no current tracked module or build consumer. |
| `scripts/graft/.cache/trail-branches.json` | OBSOLETE | Generated graft telemetry/branch cache, not RustUI runtime code; retired process data with no current tracked module or build consumer. |
| `scripts/tests/graft/.cache/telemetry-repo-id.json` | OBSOLETE | Generated graft telemetry/branch cache, not RustUI runtime code; retired process data with no current tracked module or build consumer. |
| `scripts/tests/graft/.cache/trail-branches.json` | OBSOLETE | Generated graft telemetry/branch cache, not RustUI runtime code; retired process data with no current tracked module or build consumer. |

## Already-merged T11 branch

T11 was incorporated as the second parent of mainline merge `45fb4ccbad02ffac1a4b3d3ad09365e6e1ee0edd`; C2's `bfbfab3c` completed the owned dependency/consumer integration. The archived tip is an ancestor of the review baseline. Its saved tip has parent `473e960b515311c7e40dc094fcf923fe25858f34`, but the full unmerged branch delta starts at `b9cbc60e80835fb0183175170b01f1325d66948c`, the common ancestor with C2's pre-merge mainline. This includes T11c commits `3b0303e1` and `473e960b`, rather than reviewing only the final checkpoint.

Of 735 reviewed code/metadata files, 710 still match exact blobs. The other 25 differ through current manifests, verifier composition, and approved focus/accessibility patches; none contains a separate feature to recover. Existing modal/input/list/menu behavior is retained, with the current patches addressing disabled actions, stale focus/close requests and accessibility propagation.

| Path | Class | Reason |
|---|---|---|
| `Cargo.toml` | IN MAIN | C2 (45fb4ccb, bfbfab3c) integrated the owned dependency chain; current exclusions replace incomplete workspace membership and remove the block patch override. |
| `crates/rustui-block/.cargo-checksum.json` | IN MAIN | Exact Git blob in main at `crates/rustui-block/.cargo-checksum.json`. |
| `crates/rustui-block/Cargo.toml` | IN MAIN | Exact Git blob in main at `crates/rustui-block/Cargo.toml`. |
| `crates/rustui-block/src/lib.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-block/src/lib.rs`. |
| `crates/rustui-block/src/test_utils.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-block/src/test_utils.rs`. |
| `crates/rustui-cocoa-foundation/.cargo-checksum.json` | IN MAIN | Exact Git blob in main at `crates/rustui-cocoa-foundation/.cargo-checksum.json`. |
| `crates/rustui-cocoa-foundation/.cargo_vcs_info.json` | IN MAIN | Exact Git blob in main at `crates/rustui-cocoa-foundation/.cargo_vcs_info.json`. |
| `crates/rustui-cocoa-foundation/.travis.yml` | IN MAIN | Exact Git blob in main at `crates/rustui-cocoa-foundation/.travis.yml`. |
| `crates/rustui-cocoa-foundation/Cargo.toml` | IN MAIN | Exact Git blob in main at `crates/rustui-cocoa-foundation/Cargo.toml`. |
| `crates/rustui-cocoa-foundation/Cargo.toml.orig` | IN MAIN | Exact Git blob in main at `crates/rustui-cocoa-foundation/Cargo.toml.orig`. |
| `crates/rustui-cocoa-foundation/src/base.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-cocoa-foundation/src/base.rs`. |
| `crates/rustui-cocoa-foundation/src/foundation.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-cocoa-foundation/src/foundation.rs`. |
| `crates/rustui-cocoa-foundation/src/lib.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-cocoa-foundation/src/lib.rs`. |
| `crates/rustui-cocoa-foundation/tests/foundation.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-cocoa-foundation/tests/foundation.rs`. |
| `crates/rustui-cocoa/.cargo-checksum.json` | IN MAIN | Exact Git blob in main at `crates/rustui-cocoa/.cargo-checksum.json`. |
| `crates/rustui-cocoa/.cargo_vcs_info.json` | IN MAIN | Exact Git blob in main at `crates/rustui-cocoa/.cargo_vcs_info.json`. |
| `crates/rustui-cocoa/.travis.yml` | IN MAIN | Exact Git blob in main at `crates/rustui-cocoa/.travis.yml`. |
| `crates/rustui-cocoa/COPYRIGHT` | IN MAIN | Exact Git blob in main at `crates/rustui-cocoa/COPYRIGHT`. |
| `crates/rustui-cocoa/Cargo.lock` | IN MAIN | Exact Git blob in main at `crates/rustui-cocoa/Cargo.lock`. |
| `crates/rustui-cocoa/Cargo.toml` | IN MAIN | Exact Git blob in main at `crates/rustui-cocoa/Cargo.toml`. |
| `crates/rustui-cocoa/Cargo.toml.orig` | IN MAIN | Exact Git blob in main at `crates/rustui-cocoa/Cargo.toml.orig`. |
| `crates/rustui-cocoa/examples/color.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-cocoa/examples/color.rs`. |
| `crates/rustui-cocoa/examples/fullscreen.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-cocoa/examples/fullscreen.rs`. |
| `crates/rustui-cocoa/examples/hello_world.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-cocoa/examples/hello_world.rs`. |
| `crates/rustui-cocoa/examples/nsvisualeffectview_blur.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-cocoa/examples/nsvisualeffectview_blur.rs`. |
| `crates/rustui-cocoa/examples/tab_view.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-cocoa/examples/tab_view.rs`. |
| `crates/rustui-cocoa/src/appkit.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-cocoa/src/appkit.rs`. |
| `crates/rustui-cocoa/src/lib.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-cocoa/src/lib.rs`. |
| `crates/rustui-cocoa/src/macros.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-cocoa/src/macros.rs`. |
| `crates/rustui-cocoa/src/quartzcore.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-cocoa/src/quartzcore.rs`. |
| `crates/rustui-gpui-apple/Cargo.toml` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-apple/Cargo.toml`. |
| `crates/rustui-gpui-base/.cargo-checksum.json` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/.cargo-checksum.json`. |
| `crates/rustui-gpui-base/.cargo_vcs_info.json` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/.cargo_vcs_info.json`. |
| `crates/rustui-gpui-base/Cargo.lock` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/Cargo.lock`. |
| `crates/rustui-gpui-base/Cargo.toml` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/Cargo.toml`. |
| `crates/rustui-gpui-base/Cargo.toml.orig` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/Cargo.toml.orig`. |
| `crates/rustui-gpui-base/benches/motion.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/benches/motion.rs`. |
| `crates/rustui-gpui-base/benches/text_view_scroll.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/benches/text_view_scroll.rs`. |
| `crates/rustui-gpui-base/src/accordion.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/accordion.rs`. |
| `crates/rustui-gpui-base/src/actions.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/actions.rs`. |
| `crates/rustui-gpui-base/src/alert_dialog.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/alert_dialog.rs`. |
| `crates/rustui-gpui-base/src/animation.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/animation.rs`. |
| `crates/rustui-gpui-base/src/async_util.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/async_util.rs`. |
| `crates/rustui-gpui-base/src/auto_scroll.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/auto_scroll.rs`. |
| `crates/rustui-gpui-base/src/avatar.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/avatar.rs`. |
| `crates/rustui-gpui-base/src/button.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/button.rs`. |
| `crates/rustui-gpui-base/src/calendar.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/calendar.rs`. |
| `crates/rustui-gpui-base/src/checkbox.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/checkbox.rs`. |
| `crates/rustui-gpui-base/src/collapsible.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/collapsible.rs`. |
| `crates/rustui-gpui-base/src/color_picker.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/color_picker.rs`. |
| `crates/rustui-gpui-base/src/combobox.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/combobox.rs`. |
| `crates/rustui-gpui-base/src/component_traits.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/component_traits.rs`. |
| `crates/rustui-gpui-base/src/date_picker.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/date_picker.rs`. |
| `crates/rustui-gpui-base/src/dialog.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/dialog.rs`. |
| `crates/rustui-gpui-base/src/dock/active.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/dock/active.rs`. |
| `crates/rustui-gpui-base/src/dock/dock_area.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/dock/dock_area.rs`. |
| `crates/rustui-gpui-base/src/dock/dock_placement.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/dock/dock_placement.rs`. |
| `crates/rustui-gpui-base/src/dock/drag.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/dock/drag.rs`. |
| `crates/rustui-gpui-base/src/dock/fixtures/bare_tab_panel_root.json` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/dock/fixtures/bare_tab_panel_root.json`. |
| `crates/rustui-gpui-base/src/dock/fixtures/layout.json` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/dock/fixtures/layout.json`. |
| `crates/rustui-gpui-base/src/dock/fixtures/legacy_empty_tab_group.json` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/dock/fixtures/legacy_empty_tab_group.json`. |
| `crates/rustui-gpui-base/src/dock/fixtures/nested_splits.json` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/dock/fixtures/nested_splits.json`. |
| `crates/rustui-gpui-base/src/dock/fixtures/unregistered_panel.json` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/dock/fixtures/unregistered_panel.json`. |
| `crates/rustui-gpui-base/src/dock/fixtures/zero_size_sentinel.json` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/dock/fixtures/zero_size_sentinel.json`. |
| `crates/rustui-gpui-base/src/dock/layout/builder.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/dock/layout/builder.rs`. |
| `crates/rustui-gpui-base/src/dock/layout/edit.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/dock/layout/edit.rs`. |
| `crates/rustui-gpui-base/src/dock/layout/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/dock/layout/mod.rs`. |
| `crates/rustui-gpui-base/src/dock/layout/node.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/dock/layout/node.rs`. |
| `crates/rustui-gpui-base/src/dock/layout/normalize.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/dock/layout/normalize.rs`. |
| `crates/rustui-gpui-base/src/dock/layout/tree.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/dock/layout/tree.rs`. |
| `crates/rustui-gpui-base/src/dock/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/dock/mod.rs`. |
| `crates/rustui-gpui-base/src/dock/panel.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/dock/panel.rs`. |
| `crates/rustui-gpui-base/src/dock/registry.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/dock/registry.rs`. |
| `crates/rustui-gpui-base/src/dock/state.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/dock/state.rs`. |
| `crates/rustui-gpui-base/src/dock/state_convert.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/dock/state_convert.rs`. |
| `crates/rustui-gpui-base/src/dock/tab_group.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/dock/tab_group.rs`. |
| `crates/rustui-gpui-base/src/dock/test_support.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/dock/test_support.rs`. |
| `crates/rustui-gpui-base/src/element_ext.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/element_ext.rs`. |
| `crates/rustui-gpui-base/src/event.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/event.rs`. |
| `crates/rustui-gpui-base/src/focus_trap.rs` | IN MAIN | Same focus trap retained; approved P2 forwards accessibility role/info/synthetic children through the wrapper. |
| `crates/rustui-gpui-base/src/geometry.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/geometry.rs`. |
| `crates/rustui-gpui-base/src/global_state.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/global_state.rs`. |
| `crates/rustui-gpui-base/src/history.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/history.rs`. |
| `crates/rustui-gpui-base/src/hover_card.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/hover_card.rs`. |
| `crates/rustui-gpui-base/src/index_path.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/index_path.rs`. |
| `crates/rustui-gpui-base/src/input/base/blink_cursor.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/base/blink_cursor.rs`. |
| `crates/rustui-gpui-base/src/input/base/change.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/base/change.rs`. |
| `crates/rustui-gpui-base/src/input/base/cursor.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/base/cursor.rs`. |
| `crates/rustui-gpui-base/src/input/base/element.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/base/element.rs`. |
| `crates/rustui-gpui-base/src/input/base/inline_tokens.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/base/inline_tokens.rs`. |
| `crates/rustui-gpui-base/src/input/base/kind.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/base/kind.rs`. |
| `crates/rustui-gpui-base/src/input/base/layout.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/base/layout.rs`. |
| `crates/rustui-gpui-base/src/input/base/mask_pattern.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/base/mask_pattern.rs`. |
| `crates/rustui-gpui-base/src/input/base/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/base/mod.rs`. |
| `crates/rustui-gpui-base/src/input/base/mode.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/base/mode.rs`. |
| `crates/rustui-gpui-base/src/input/base/movement.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/base/movement.rs`. |
| `crates/rustui-gpui-base/src/input/base/native.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/base/native.rs`. |
| `crates/rustui-gpui-base/src/input/base/rope_ext.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/base/rope_ext.rs`. |
| `crates/rustui-gpui-base/src/input/base/selection.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/base/selection.rs`. |
| `crates/rustui-gpui-base/src/input/base/state.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/base/state.rs`. |
| `crates/rustui-gpui-base/src/input/base/token_presentation.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/base/token_presentation.rs`. |
| `crates/rustui-gpui-base/src/input/base/touch.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/base/touch.rs`. |
| `crates/rustui-gpui-base/src/input/base/undo_manager.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/base/undo_manager.rs`. |
| `crates/rustui-gpui-base/src/input/editor/auto_close.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/editor/auto_close.rs`. |
| `crates/rustui-gpui-base/src/input/editor/decorations.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/editor/decorations.rs`. |
| `crates/rustui-gpui-base/src/input/editor/diagnostics.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/editor/diagnostics.rs`. |
| `crates/rustui-gpui-base/src/input/editor/display_map/display_map.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/editor/display_map/display_map.rs`. |
| `crates/rustui-gpui-base/src/input/editor/display_map/fold_map.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/editor/display_map/fold_map.rs`. |
| `crates/rustui-gpui-base/src/input/editor/display_map/folding.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/editor/display_map/folding.rs`. |
| `crates/rustui-gpui-base/src/input/editor/display_map/inline_line.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/editor/display_map/inline_line.rs`. |
| `crates/rustui-gpui-base/src/input/editor/display_map/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/editor/display_map/mod.rs`. |
| `crates/rustui-gpui-base/src/input/editor/display_map/text_wrapper.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/editor/display_map/text_wrapper.rs`. |
| `crates/rustui-gpui-base/src/input/editor/display_map/wrap_map.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/editor/display_map/wrap_map.rs`. |
| `crates/rustui-gpui-base/src/input/editor/highlighting.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/editor/highlighting.rs`. |
| `crates/rustui-gpui-base/src/input/editor/indent.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/editor/indent.rs`. |
| `crates/rustui-gpui-base/src/input/editor/language.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/editor/language.rs`. |
| `crates/rustui-gpui-base/src/input/editor/language_config.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/editor/language_config.rs`. |
| `crates/rustui-gpui-base/src/input/editor/lsp/code_actions.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/editor/lsp/code_actions.rs`. |
| `crates/rustui-gpui-base/src/input/editor/lsp/completions.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/editor/lsp/completions.rs`. |
| `crates/rustui-gpui-base/src/input/editor/lsp/definitions.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/editor/lsp/definitions.rs`. |
| `crates/rustui-gpui-base/src/input/editor/lsp/document_colors.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/editor/lsp/document_colors.rs`. |
| `crates/rustui-gpui-base/src/input/editor/lsp/hover.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/editor/lsp/hover.rs`. |
| `crates/rustui-gpui-base/src/input/editor/lsp/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/editor/lsp/mod.rs`. |
| `crates/rustui-gpui-base/src/input/editor/lsp/overlay.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/editor/lsp/overlay.rs`. |
| `crates/rustui-gpui-base/src/input/editor/lsp/semantic_tokens.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/editor/lsp/semantic_tokens.rs`. |
| `crates/rustui-gpui-base/src/input/editor/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/editor/mod.rs`. |
| `crates/rustui-gpui-base/src/input/editor/search.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/editor/search.rs`. |
| `crates/rustui-gpui-base/src/input/input/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/input/mod.rs`. |
| `crates/rustui-gpui-base/src/input/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/mod.rs`. |
| `crates/rustui-gpui-base/src/input/textarea/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/input/textarea/mod.rs`. |
| `crates/rustui-gpui-base/src/lib.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/lib.rs`. |
| `crates/rustui-gpui-base/src/link.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/link.rs`. |
| `crates/rustui-gpui-base/src/list_settings.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/list_settings.rs`. |
| `crates/rustui-gpui-base/src/macos_accessibility.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/macos_accessibility.rs`. |
| `crates/rustui-gpui-base/src/measure.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/measure.rs`. |
| `crates/rustui-gpui-base/src/motion.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/motion.rs`. |
| `crates/rustui-gpui-base/src/motion/easing.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/motion/easing.rs`. |
| `crates/rustui-gpui-base/src/motion/keyframes.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/motion/keyframes.rs`. |
| `crates/rustui-gpui-base/src/motion/presence.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/motion/presence.rs`. |
| `crates/rustui-gpui-base/src/motion/reveal.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/motion/reveal.rs`. |
| `crates/rustui-gpui-base/src/motion/sequence.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/motion/sequence.rs`. |
| `crates/rustui-gpui-base/src/motion/stagger.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/motion/stagger.rs`. |
| `crates/rustui-gpui-base/src/motion/timing.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/motion/timing.rs`. |
| `crates/rustui-gpui-base/src/nav_stack.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/nav_stack.rs`. |
| `crates/rustui-gpui-base/src/number_input.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/number_input.rs`. |
| `crates/rustui-gpui-base/src/observe.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/observe.rs`. |
| `crates/rustui-gpui-base/src/otp_input.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/otp_input.rs`. |
| `crates/rustui-gpui-base/src/pagination.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/pagination.rs`. |
| `crates/rustui-gpui-base/src/plot/appear.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/plot/appear.rs`. |
| `crates/rustui-gpui-base/src/plot/axis.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/plot/axis.rs`. |
| `crates/rustui-gpui-base/src/plot/element.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/plot/element.rs`. |
| `crates/rustui-gpui-base/src/plot/grid.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/plot/grid.rs`. |
| `crates/rustui-gpui-base/src/plot/hover.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/plot/hover.rs`. |
| `crates/rustui-gpui-base/src/plot/label.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/plot/label.rs`. |
| `crates/rustui-gpui-base/src/plot/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/plot/mod.rs`. |
| `crates/rustui-gpui-base/src/plot/path_cache.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/plot/path_cache.rs`. |
| `crates/rustui-gpui-base/src/plot/scale.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/plot/scale.rs`. |
| `crates/rustui-gpui-base/src/plot/scale/band.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/plot/scale/band.rs`. |
| `crates/rustui-gpui-base/src/plot/scale/linear.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/plot/scale/linear.rs`. |
| `crates/rustui-gpui-base/src/plot/scale/ordinal.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/plot/scale/ordinal.rs`. |
| `crates/rustui-gpui-base/src/plot/scale/point.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/plot/scale/point.rs`. |
| `crates/rustui-gpui-base/src/plot/scale/sealed.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/plot/scale/sealed.rs`. |
| `crates/rustui-gpui-base/src/plot/shape.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/plot/shape.rs`. |
| `crates/rustui-gpui-base/src/plot/shape/arc.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/plot/shape/arc.rs`. |
| `crates/rustui-gpui-base/src/plot/shape/area.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/plot/shape/area.rs`. |
| `crates/rustui-gpui-base/src/plot/shape/bar.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/plot/shape/bar.rs`. |
| `crates/rustui-gpui-base/src/plot/shape/line.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/plot/shape/line.rs`. |
| `crates/rustui-gpui-base/src/plot/shape/pie.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/plot/shape/pie.rs`. |
| `crates/rustui-gpui-base/src/plot/shape/radial_line.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/plot/shape/radial_line.rs`. |
| `crates/rustui-gpui-base/src/plot/shape/sankey.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/plot/shape/sankey.rs`. |
| `crates/rustui-gpui-base/src/plot/shape/stack.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/plot/shape/stack.rs`. |
| `crates/rustui-gpui-base/src/popover.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/popover.rs`. |
| `crates/rustui-gpui-base/src/popup.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/popup.rs`. |
| `crates/rustui-gpui-base/src/positioner.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/positioner.rs`. |
| `crates/rustui-gpui-base/src/progress.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/progress.rs`. |
| `crates/rustui-gpui-base/src/questionnaire/control.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/questionnaire/control.rs`. |
| `crates/rustui-gpui-base/src/questionnaire/keyboard.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/questionnaire/keyboard.rs`. |
| `crates/rustui-gpui-base/src/questionnaire/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/questionnaire/mod.rs`. |
| `crates/rustui-gpui-base/src/questionnaire/state.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/questionnaire/state.rs`. |
| `crates/rustui-gpui-base/src/questionnaire/types.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/questionnaire/types.rs`. |
| `crates/rustui-gpui-base/src/radio.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/radio.rs`. |
| `crates/rustui-gpui-base/src/radio_group.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/radio_group.rs`. |
| `crates/rustui-gpui-base/src/reduce_motion.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/reduce_motion.rs`. |
| `crates/rustui-gpui-base/src/resizable/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/resizable/mod.rs`. |
| `crates/rustui-gpui-base/src/resizable/panel.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/resizable/panel.rs`. |
| `crates/rustui-gpui-base/src/resizable/resize_handle.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/resizable/resize_handle.rs`. |
| `crates/rustui-gpui-base/src/root.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/root.rs`. |
| `crates/rustui-gpui-base/src/scroll_bounce.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/scroll_bounce.rs`. |
| `crates/rustui-gpui-base/src/scrollable_mask.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/scrollable_mask.rs`. |
| `crates/rustui-gpui-base/src/scrollbar.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/scrollbar.rs`. |
| `crates/rustui-gpui-base/src/select.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/select.rs`. |
| `crates/rustui-gpui-base/src/selectable_text.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/selectable_text.rs`. |
| `crates/rustui-gpui-base/src/sheet.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/sheet.rs`. |
| `crates/rustui-gpui-base/src/slider.rs` | IN MAIN | Same state/render implementation retained; approved P2b adds labelled SetValue/Increment/Decrement handling with event emission and step/bounds normalization. |
| `crates/rustui-gpui-base/src/state_style.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/state_style.rs`. |
| `crates/rustui-gpui-base/src/styled.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/styled.rs`. |
| `crates/rustui-gpui-base/src/switch.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/switch.rs`. |
| `crates/rustui-gpui-base/src/table.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/table.rs`. |
| `crates/rustui-gpui-base/src/tabs.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/tabs.rs`. |
| `crates/rustui-gpui-base/src/test_support.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/test_support.rs`. |
| `crates/rustui-gpui-base/src/text/document.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/text/document.rs`. |
| `crates/rustui-gpui-base/src/text/format/html.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/text/format/html.rs`. |
| `crates/rustui-gpui-base/src/text/format/html5minify/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/text/format/html5minify/mod.rs`. |
| `crates/rustui-gpui-base/src/text/format/markdown.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/text/format/markdown.rs`. |
| `crates/rustui-gpui-base/src/text/format/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/text/format/mod.rs`. |
| `crates/rustui-gpui-base/src/text/inline.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/text/inline.rs`. |
| `crates/rustui-gpui-base/src/text/inline_element.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/text/inline_element.rs`. |
| `crates/rustui-gpui-base/src/text/inline_flow.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/text/inline_flow.rs`. |
| `crates/rustui-gpui-base/src/text/inline_object.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/text/inline_object.rs`. |
| `crates/rustui-gpui-base/src/text/inline_virtual_tests.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/text/inline_virtual_tests.rs`. |
| `crates/rustui-gpui-base/src/text/markdown_ext.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/text/markdown_ext.rs`. |
| `crates/rustui-gpui-base/src/text/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/text/mod.rs`. |
| `crates/rustui-gpui-base/src/text/node.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/text/node.rs`. |
| `crates/rustui-gpui-base/src/text/range_highlight.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/text/range_highlight.rs`. |
| `crates/rustui-gpui-base/src/text/selection.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/text/selection.rs`. |
| `crates/rustui-gpui-base/src/text/selection_adapter.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/text/selection_adapter.rs`. |
| `crates/rustui-gpui-base/src/text/state.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/text/state.rs`. |
| `crates/rustui-gpui-base/src/text/stream_fade.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/text/stream_fade.rs`. |
| `crates/rustui-gpui-base/src/text/style.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/text/style.rs`. |
| `crates/rustui-gpui-base/src/text/text_view.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/text/text_view.rs`. |
| `crates/rustui-gpui-base/src/text/utils.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/text/utils.rs`. |
| `crates/rustui-gpui-base/src/text_boundary.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/text_boundary.rs`. |
| `crates/rustui-gpui-base/src/text_selection.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/text_selection.rs`. |
| `crates/rustui-gpui-base/src/theme.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/theme.rs`. |
| `crates/rustui-gpui-base/src/theme_tokens.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/theme_tokens.rs`. |
| `crates/rustui-gpui-base/src/time_field.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/time_field.rs`. |
| `crates/rustui-gpui-base/src/toast.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/toast.rs`. |
| `crates/rustui-gpui-base/src/toggle.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/toggle.rs`. |
| `crates/rustui-gpui-base/src/toggle_group.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/toggle_group.rs`. |
| `crates/rustui-gpui-base/src/toolbar.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/toolbar.rs`. |
| `crates/rustui-gpui-base/src/tooltip.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/tooltip.rs`. |
| `crates/rustui-gpui-base/src/touch_selection.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/touch_selection.rs`. |
| `crates/rustui-gpui-base/src/tree.rs` | IN MAIN | Same hierarchy retained; approved P2/R15 patches reject disabled expansion and recover focus when all entries become disabled. |
| `crates/rustui-gpui-base/src/undo_history.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/undo_history.rs`. |
| `crates/rustui-gpui-base/src/virtual_list.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/src/virtual_list.rs`. |
| `crates/rustui-gpui-base/tests/element_ext.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-base/tests/element_ext.rs`. |
| `crates/rustui-gpui-component/.cargo-checksum.json` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/.cargo-checksum.json`. |
| `crates/rustui-gpui-component/.cargo_vcs_info.json` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/.cargo_vcs_info.json`. |
| `crates/rustui-gpui-component/Cargo.lock` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/Cargo.lock`. |
| `crates/rustui-gpui-component/Cargo.toml` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/Cargo.toml`. |
| `crates/rustui-gpui-component/Cargo.toml.orig` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/Cargo.toml.orig`. |
| `crates/rustui-gpui-component/build.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/build.rs`. |
| `crates/rustui-gpui-component/locales/ui.yml` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/locales/ui.yml`. |
| `crates/rustui-gpui-component/src/accordion.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/accordion.rs`. |
| `crates/rustui-gpui-component/src/alert.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/alert.rs`. |
| `crates/rustui-gpui-component/src/attachment.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/attachment.rs`. |
| `crates/rustui-gpui-component/src/avatar/avatar.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/avatar/avatar.rs`. |
| `crates/rustui-gpui-component/src/avatar/avatar_group.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/avatar/avatar_group.rs`. |
| `crates/rustui-gpui-component/src/avatar/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/avatar/mod.rs`. |
| `crates/rustui-gpui-component/src/badge.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/badge.rs`. |
| `crates/rustui-gpui-component/src/breadcrumb.rs` | IN MAIN | Same item renderer retained; approved R15 adds retained focus, disabled semantics and live focus recovery. |
| `crates/rustui-gpui-component/src/bubble.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/bubble.rs`. |
| `crates/rustui-gpui-component/src/button/button.rs` | IN MAIN | Same button renderer retained; approved R15 moves focus away when a focused button becomes disabled. |
| `crates/rustui-gpui-component/src/button/button_group.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/button/button_group.rs`. |
| `crates/rustui-gpui-component/src/button/button_icon.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/button/button_icon.rs`. |
| `crates/rustui-gpui-component/src/button/dropdown_button.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/button/dropdown_button.rs`. |
| `crates/rustui-gpui-component/src/button/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/button/mod.rs`. |
| `crates/rustui-gpui-component/src/button/toggle.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/button/toggle.rs`. |
| `crates/rustui-gpui-component/src/carousel/carousel.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/carousel/carousel.rs`. |
| `crates/rustui-gpui-component/src/carousel/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/carousel/mod.rs`. |
| `crates/rustui-gpui-component/src/carousel/scroll_mask.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/carousel/scroll_mask.rs`. |
| `crates/rustui-gpui-component/src/carousel/state.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/carousel/state.rs`. |
| `crates/rustui-gpui-component/src/chart/area_chart.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/chart/area_chart.rs`. |
| `crates/rustui-gpui-component/src/chart/bar_chart.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/chart/bar_chart.rs`. |
| `crates/rustui-gpui-component/src/chart/candlestick_chart.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/chart/candlestick_chart.rs`. |
| `crates/rustui-gpui-component/src/chart/line_chart.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/chart/line_chart.rs`. |
| `crates/rustui-gpui-component/src/chart/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/chart/mod.rs`. |
| `crates/rustui-gpui-component/src/chart/pie_chart.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/chart/pie_chart.rs`. |
| `crates/rustui-gpui-component/src/chart/radar_chart.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/chart/radar_chart.rs`. |
| `crates/rustui-gpui-component/src/chart/sankey_chart.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/chart/sankey_chart.rs`. |
| `crates/rustui-gpui-component/src/checkbox.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/checkbox.rs`. |
| `crates/rustui-gpui-component/src/clipboard.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/clipboard.rs`. |
| `crates/rustui-gpui-component/src/collapsible.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/collapsible.rs`. |
| `crates/rustui-gpui-component/src/color_picker.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/color_picker.rs`. |
| `crates/rustui-gpui-component/src/combobox.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/combobox.rs`. |
| `crates/rustui-gpui-component/src/command/command.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/command/command.rs`. |
| `crates/rustui-gpui-component/src/command/item.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/command/item.rs`. |
| `crates/rustui-gpui-component/src/command/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/command/mod.rs`. |
| `crates/rustui-gpui-component/src/command/state.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/command/state.rs`. |
| `crates/rustui-gpui-component/src/component_traits.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/component_traits.rs`. |
| `crates/rustui-gpui-component/src/description_list.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/description_list.rs`. |
| `crates/rustui-gpui-component/src/dialog/alert_dialog.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/dialog/alert_dialog.rs`. |
| `crates/rustui-gpui-component/src/dialog/content.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/dialog/content.rs`. |
| `crates/rustui-gpui-component/src/dialog/description.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/dialog/description.rs`. |
| `crates/rustui-gpui-component/src/dialog/dialog.rs` | IN MAIN | Same dialog remains; approved tracked-dialog identity handles close requests without closing a successor. |
| `crates/rustui-gpui-component/src/dialog/dispatch_anchor.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/dialog/dispatch_anchor.rs`. |
| `crates/rustui-gpui-component/src/dialog/entrance.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/dialog/entrance.rs`. |
| `crates/rustui-gpui-component/src/dialog/footer.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/dialog/footer.rs`. |
| `crates/rustui-gpui-component/src/dialog/header.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/dialog/header.rs`. |
| `crates/rustui-gpui-component/src/dialog/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/dialog/mod.rs`. |
| `crates/rustui-gpui-component/src/dialog/title.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/dialog/title.rs`. |
| `crates/rustui-gpui-component/src/diff/conflict.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/diff/conflict.rs`. |
| `crates/rustui-gpui-component/src/diff/copy_tests.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/diff/copy_tests.rs`. |
| `crates/rustui-gpui-component/src/diff/document.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/diff/document.rs`. |
| `crates/rustui-gpui-component/src/diff/header_tests.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/diff/header_tests.rs`. |
| `crates/rustui-gpui-component/src/diff/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/diff/mod.rs`. |
| `crates/rustui-gpui-component/src/diff/newline_tests.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/diff/newline_tests.rs`. |
| `crates/rustui-gpui-component/src/diff/parser.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/diff/parser.rs`. |
| `crates/rustui-gpui-component/src/diff/parser_tests.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/diff/parser_tests.rs`. |
| `crates/rustui-gpui-component/src/diff/presentation.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/diff/presentation.rs`. |
| `crates/rustui-gpui-component/src/diff/review_tests.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/diff/review_tests.rs`. |
| `crates/rustui-gpui-component/src/diff/selection.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/diff/selection.rs`. |
| `crates/rustui-gpui-component/src/diff/selection_tests.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/diff/selection_tests.rs`. |
| `crates/rustui-gpui-component/src/diff/state.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/diff/state.rs`. |
| `crates/rustui-gpui-component/src/diff/tests.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/diff/tests.rs`. |
| `crates/rustui-gpui-component/src/dock/dock.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/dock/dock.rs`. |
| `crates/rustui-gpui-component/src/dock/invalid_panel.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/dock/invalid_panel.rs`. |
| `crates/rustui-gpui-component/src/dock/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/dock/mod.rs`. |
| `crates/rustui-gpui-component/src/dock/panel.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/dock/panel.rs`. |
| `crates/rustui-gpui-component/src/dock/tab_panel.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/dock/tab_panel.rs`. |
| `crates/rustui-gpui-component/src/dock/test_support.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/dock/test_support.rs`. |
| `crates/rustui-gpui-component/src/element_ext.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/element_ext.rs`. |
| `crates/rustui-gpui-component/src/empty.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/empty.rs`. |
| `crates/rustui-gpui-component/src/form/field.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/form/field.rs`. |
| `crates/rustui-gpui-component/src/form/form.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/form/form.rs`. |
| `crates/rustui-gpui-component/src/form/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/form/mod.rs`. |
| `crates/rustui-gpui-component/src/form/tests.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/form/tests.rs`. |
| `crates/rustui-gpui-component/src/global_state.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/global_state.rs`. |
| `crates/rustui-gpui-component/src/group_box.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/group_box.rs`. |
| `crates/rustui-gpui-component/src/highlighter/diagnostic_styles.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/highlighter/diagnostic_styles.rs`. |
| `crates/rustui-gpui-component/src/highlighter/highlighter.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/highlighter/highlighter.rs`. |
| `crates/rustui-gpui-component/src/highlighter/input_adapter.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/highlighter/input_adapter.rs`. |
| `crates/rustui-gpui-component/src/highlighter/language_name.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/highlighter/language_name.rs`. |
| `crates/rustui-gpui-component/src/highlighter/languages.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/highlighter/languages.rs`. |
| `crates/rustui-gpui-component/src/highlighter/languages/go/highlights.scm` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/highlighter/languages/go/highlights.scm`. |
| `crates/rustui-gpui-component/src/highlighter/languages/html/highlights.scm` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/highlighter/languages/html/highlights.scm`. |
| `crates/rustui-gpui-component/src/highlighter/languages/html/injections.scm` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/highlighter/languages/html/injections.scm`. |
| `crates/rustui-gpui-component/src/highlighter/languages/javascript/highlights.scm` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/highlighter/languages/javascript/highlights.scm`. |
| `crates/rustui-gpui-component/src/highlighter/languages/javascript/injections.scm` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/highlighter/languages/javascript/injections.scm`. |
| `crates/rustui-gpui-component/src/highlighter/languages/json/highlights.scm` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/highlighter/languages/json/highlights.scm`. |
| `crates/rustui-gpui-component/src/highlighter/languages/kotlin/highlights.scm` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/highlighter/languages/kotlin/highlights.scm`. |
| `crates/rustui-gpui-component/src/highlighter/languages/lua/highlights.scm` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/highlighter/languages/lua/highlights.scm`. |
| `crates/rustui-gpui-component/src/highlighter/languages/markdown/highlights.scm` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/highlighter/languages/markdown/highlights.scm`. |
| `crates/rustui-gpui-component/src/highlighter/languages/markdown/injections.scm` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/highlighter/languages/markdown/injections.scm`. |
| `crates/rustui-gpui-component/src/highlighter/languages/markdown_inline/highlights.scm` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/highlighter/languages/markdown_inline/highlights.scm`. |
| `crates/rustui-gpui-component/src/highlighter/languages/php/injections.scm` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/highlighter/languages/php/injections.scm`. |
| `crates/rustui-gpui-component/src/highlighter/languages/rust/highlights.scm` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/highlighter/languages/rust/highlights.scm`. |
| `crates/rustui-gpui-component/src/highlighter/languages/rust/injections.scm` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/highlighter/languages/rust/injections.scm`. |
| `crates/rustui-gpui-component/src/highlighter/languages/typescript/highlights.scm` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/highlighter/languages/typescript/highlights.scm`. |
| `crates/rustui-gpui-component/src/highlighter/languages/zig/highlights.scm` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/highlighter/languages/zig/highlights.scm`. |
| `crates/rustui-gpui-component/src/highlighter/languages/zig/injections.scm` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/highlighter/languages/zig/injections.scm`. |
| `crates/rustui-gpui-component/src/highlighter/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/highlighter/mod.rs`. |
| `crates/rustui-gpui-component/src/highlighter/registry.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/highlighter/registry.rs`. |
| `crates/rustui-gpui-component/src/highlighter/wasm_stub.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/highlighter/wasm_stub.rs`. |
| `crates/rustui-gpui-component/src/history.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/history.rs`. |
| `crates/rustui-gpui-component/src/hover_card.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/hover_card.rs`. |
| `crates/rustui-gpui-component/src/icon.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/icon.rs`. |
| `crates/rustui-gpui-component/src/index_path.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/index_path.rs`. |
| `crates/rustui-gpui-component/src/input/clear_button.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/input/clear_button.rs`. |
| `crates/rustui-gpui-component/src/input/content_type.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/input/content_type.rs`. |
| `crates/rustui-gpui-component/src/input/editor.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/input/editor.rs`. |
| `crates/rustui-gpui-component/src/input/group.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/input/group.rs`. |
| `crates/rustui-gpui-component/src/input/group/button_element.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/input/group/button_element.rs`. |
| `crates/rustui-gpui-component/src/input/group/tests.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/input/group/tests.rs`. |
| `crates/rustui-gpui-component/src/input/input.rs` | IN MAIN | Same input retained; approved validation patch forwards supplementary aria_description to the input node. |
| `crates/rustui-gpui-component/src/input/language_config.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/input/language_config.rs`. |
| `crates/rustui-gpui-component/src/input/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/input/mod.rs`. |
| `crates/rustui-gpui-component/src/input/number_input.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/input/number_input.rs`. |
| `crates/rustui-gpui-component/src/input/otp_input.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/input/otp_input.rs`. |
| `crates/rustui-gpui-component/src/input/overlay.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/input/overlay.rs`. |
| `crates/rustui-gpui-component/src/input/popovers/caret_anchored.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/input/popovers/caret_anchored.rs`. |
| `crates/rustui-gpui-component/src/input/popovers/code_action_menu.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/input/popovers/code_action_menu.rs`. |
| `crates/rustui-gpui-component/src/input/popovers/completion_menu.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/input/popovers/completion_menu.rs`. |
| `crates/rustui-gpui-component/src/input/popovers/diagnostic_popover.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/input/popovers/diagnostic_popover.rs`. |
| `crates/rustui-gpui-component/src/input/popovers/hover_popover.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/input/popovers/hover_popover.rs`. |
| `crates/rustui-gpui-component/src/input/popovers/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/input/popovers/mod.rs`. |
| `crates/rustui-gpui-component/src/input/search.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/input/search.rs`. |
| `crates/rustui-gpui-component/src/input/state.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/input/state.rs`. |
| `crates/rustui-gpui-component/src/input/syntax_context.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/input/syntax_context.rs`. |
| `crates/rustui-gpui-component/src/input/textarea.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/input/textarea.rs`. |
| `crates/rustui-gpui-component/src/input/token.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/input/token.rs`. |
| `crates/rustui-gpui-component/src/inspector.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/inspector.rs`. |
| `crates/rustui-gpui-component/src/kbd.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/kbd.rs`. |
| `crates/rustui-gpui-component/src/label.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/label.rs`. |
| `crates/rustui-gpui-component/src/lib.rs` | IN MAIN | Existing exports retained; approved dialog identity patch additionally exports DialogId and TrackedDialogExt. |
| `crates/rustui-gpui-component/src/link.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/link.rs`. |
| `crates/rustui-gpui-component/src/list/cache.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/list/cache.rs`. |
| `crates/rustui-gpui-component/src/list/delegate.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/list/delegate.rs`. |
| `crates/rustui-gpui-component/src/list/list.rs` | IN MAIN | Same list state/rendering retained; approved selectable-list semantics chooses listbox/option roles and emits selected only for selectable rows. |
| `crates/rustui-gpui-component/src/list/list_item.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/list/list_item.rs`. |
| `crates/rustui-gpui-component/src/list/loading.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/list/loading.rs`. |
| `crates/rustui-gpui-component/src/list/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/list/mod.rs`. |
| `crates/rustui-gpui-component/src/list/separator_item.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/list/separator_item.rs`. |
| `crates/rustui-gpui-component/src/marker.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/marker.rs`. |
| `crates/rustui-gpui-component/src/menu/app_menu_bar.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/menu/app_menu_bar.rs`. |
| `crates/rustui-gpui-component/src/menu/context_menu.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/menu/context_menu.rs`. |
| `crates/rustui-gpui-component/src/menu/dropdown_menu.rs` | IN MAIN | Same retained menu remains; approved R15b adds opt-in refresh_on_render while keeping menu/focus identity. |
| `crates/rustui-gpui-component/src/menu/menu_item.rs` | IN MAIN | Same item renderer retained; approved R15b forwards the disabled property to accessibility nodes. |
| `crates/rustui-gpui-component/src/menu/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/menu/mod.rs`. |
| `crates/rustui-gpui-component/src/menu/popup_menu.rs` | IN MAIN | Same menu construction/activation retained; approved P2/R15b rejects disabled confirmation, skips disabled selection/hover and refreshes live selection. |
| `crates/rustui-gpui-component/src/message.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/message.rs`. |
| `crates/rustui-gpui-component/src/message_scroller.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/message_scroller.rs`. |
| `crates/rustui-gpui-component/src/native_menu/fallback.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/native_menu/fallback.rs`. |
| `crates/rustui-gpui-component/src/native_menu/macos.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/native_menu/macos.rs`. |
| `crates/rustui-gpui-component/src/native_menu/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/native_menu/mod.rs`. |
| `crates/rustui-gpui-component/src/native_menu/windows.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/native_menu/windows.rs`. |
| `crates/rustui-gpui-component/src/notification.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/notification.rs`. |
| `crates/rustui-gpui-component/src/pagination.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/pagination.rs`. |
| `crates/rustui-gpui-component/src/plot/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/plot/mod.rs`. |
| `crates/rustui-gpui-component/src/plot/tooltip.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/plot/tooltip.rs`. |
| `crates/rustui-gpui-component/src/popover.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/popover.rs`. |
| `crates/rustui-gpui-component/src/progress/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/progress/mod.rs`. |
| `crates/rustui-gpui-component/src/progress/progress.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/progress/progress.rs`. |
| `crates/rustui-gpui-component/src/progress/progress_circle.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/progress/progress_circle.rs`. |
| `crates/rustui-gpui-component/src/questionnaire/components.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/questionnaire/components.rs`. |
| `crates/rustui-gpui-component/src/questionnaire/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/questionnaire/mod.rs`. |
| `crates/rustui-gpui-component/src/radio.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/radio.rs`. |
| `crates/rustui-gpui-component/src/rating.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/rating.rs`. |
| `crates/rustui-gpui-component/src/resizable.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/resizable.rs`. |
| `crates/rustui-gpui-component/src/root.rs` | IN MAIN | Same modal stack retained; approved focus/identity patches use epochs and opening IDs to preserve the top modal and prevent stale deferred closes/restores. |
| `crates/rustui-gpui-component/src/scroll/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/scroll/mod.rs`. |
| `crates/rustui-gpui-component/src/scroll/scrollable.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/scroll/scrollable.rs`. |
| `crates/rustui-gpui-component/src/searchable_list.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/searchable_list.rs`. |
| `crates/rustui-gpui-component/src/searchable_list/adapter.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/searchable_list/adapter.rs`. |
| `crates/rustui-gpui-component/src/searchable_list/change.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/searchable_list/change.rs`. |
| `crates/rustui-gpui-component/src/searchable_list/delegate.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/searchable_list/delegate.rs`. |
| `crates/rustui-gpui-component/src/searchable_list/item.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/searchable_list/item.rs`. |
| `crates/rustui-gpui-component/src/searchable_list/state.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/searchable_list/state.rs`. |
| `crates/rustui-gpui-component/src/searchable_list/vec.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/searchable_list/vec.rs`. |
| `crates/rustui-gpui-component/src/select.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/select.rs`. |
| `crates/rustui-gpui-component/src/separator.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/separator.rs`. |
| `crates/rustui-gpui-component/src/setting/fields/bool.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/setting/fields/bool.rs`. |
| `crates/rustui-gpui-component/src/setting/fields/dropdown.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/setting/fields/dropdown.rs`. |
| `crates/rustui-gpui-component/src/setting/fields/element.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/setting/fields/element.rs`. |
| `crates/rustui-gpui-component/src/setting/fields/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/setting/fields/mod.rs`. |
| `crates/rustui-gpui-component/src/setting/fields/number.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/setting/fields/number.rs`. |
| `crates/rustui-gpui-component/src/setting/fields/string.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/setting/fields/string.rs`. |
| `crates/rustui-gpui-component/src/setting/group.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/setting/group.rs`. |
| `crates/rustui-gpui-component/src/setting/item.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/setting/item.rs`. |
| `crates/rustui-gpui-component/src/setting/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/setting/mod.rs`. |
| `crates/rustui-gpui-component/src/setting/page.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/setting/page.rs`. |
| `crates/rustui-gpui-component/src/setting/settings.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/setting/settings.rs`. |
| `crates/rustui-gpui-component/src/setting/tests.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/setting/tests.rs`. |
| `crates/rustui-gpui-component/src/sheet.rs` | IN MAIN | Same sheet renderer retained; approved P2 forwards a close-button accessibility label. |
| `crates/rustui-gpui-component/src/shimmer.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/shimmer.rs`. |
| `crates/rustui-gpui-component/src/sidebar/footer.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/sidebar/footer.rs`. |
| `crates/rustui-gpui-component/src/sidebar/group.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/sidebar/group.rs`. |
| `crates/rustui-gpui-component/src/sidebar/header.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/sidebar/header.rs`. |
| `crates/rustui-gpui-component/src/sidebar/menu.rs` | IN MAIN | Same sidebar item retained; approved R15 forwards its disabled state to the accessibility node. |
| `crates/rustui-gpui-component/src/sidebar/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/sidebar/mod.rs`. |
| `crates/rustui-gpui-component/src/sizing.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/sizing.rs`. |
| `crates/rustui-gpui-component/src/skeleton.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/skeleton.rs`. |
| `crates/rustui-gpui-component/src/slider.rs` | IN MAIN | Same slider adapter retained; approved P2b forwards the optional accessibility label to the base slider. |
| `crates/rustui-gpui-component/src/speech/button.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/speech/button.rs`. |
| `crates/rustui-gpui-component/src/speech/level.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/speech/level.rs`. |
| `crates/rustui-gpui-component/src/speech/microphone.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/speech/microphone.rs`. |
| `crates/rustui-gpui-component/src/speech/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/speech/mod.rs`. |
| `crates/rustui-gpui-component/src/speech/recognizer.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/speech/recognizer.rs`. |
| `crates/rustui-gpui-component/src/speech/state.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/speech/state.rs`. |
| `crates/rustui-gpui-component/src/speech/system/macos.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/speech/system/macos.rs`. |
| `crates/rustui-gpui-component/src/speech/system/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/speech/system/mod.rs`. |
| `crates/rustui-gpui-component/src/speech/system/unsupported.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/speech/system/unsupported.rs`. |
| `crates/rustui-gpui-component/src/speech/system/winrt.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/speech/system/winrt.rs`. |
| `crates/rustui-gpui-component/src/speech/waveform.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/speech/waveform.rs`. |
| `crates/rustui-gpui-component/src/spinner.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/spinner.rs`. |
| `crates/rustui-gpui-component/src/status_bar.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/status_bar.rs`. |
| `crates/rustui-gpui-component/src/stepper/item.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/stepper/item.rs`. |
| `crates/rustui-gpui-component/src/stepper/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/stepper/mod.rs`. |
| `crates/rustui-gpui-component/src/stepper/stepper.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/stepper/stepper.rs`. |
| `crates/rustui-gpui-component/src/stepper/trigger.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/stepper/trigger.rs`. |
| `crates/rustui-gpui-component/src/styled.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/styled.rs`. |
| `crates/rustui-gpui-component/src/switch.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/switch.rs`. |
| `crates/rustui-gpui-component/src/tab/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/tab/mod.rs`. |
| `crates/rustui-gpui-component/src/tab/tab.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/tab/tab.rs`. |
| `crates/rustui-gpui-component/src/tab/tab_bar.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/tab/tab_bar.rs`. |
| `crates/rustui-gpui-component/src/table/column.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/table/column.rs`. |
| `crates/rustui-gpui-component/src/table/data_table.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/table/data_table.rs`. |
| `crates/rustui-gpui-component/src/table/delegate.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/table/delegate.rs`. |
| `crates/rustui-gpui-component/src/table/loading.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/table/loading.rs`. |
| `crates/rustui-gpui-component/src/table/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/table/mod.rs`. |
| `crates/rustui-gpui-component/src/table/state.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/table/state.rs`. |
| `crates/rustui-gpui-component/src/table/table.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/table/table.rs`. |
| `crates/rustui-gpui-component/src/tag.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/tag.rs`. |
| `crates/rustui-gpui-component/src/text/compat.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/text/compat.rs`. |
| `crates/rustui-gpui-component/src/text/frontmatter.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/text/frontmatter.rs`. |
| `crates/rustui-gpui-component/src/text/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/text/mod.rs`. |
| `crates/rustui-gpui-component/src/text/style.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/text/style.rs`. |
| `crates/rustui-gpui-component/src/text/window_selection.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/text/window_selection.rs`. |
| `crates/rustui-gpui-component/src/theme/color.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/theme/color.rs`. |
| `crates/rustui-gpui-component/src/theme/default-colors.json` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/theme/default-colors.json`. |
| `crates/rustui-gpui-component/src/theme/default-theme.json` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/theme/default-theme.json`. |
| `crates/rustui-gpui-component/src/theme/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/theme/mod.rs`. |
| `crates/rustui-gpui-component/src/theme/mono_font.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/theme/mono_font.rs`. |
| `crates/rustui-gpui-component/src/theme/motion.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/theme/motion.rs`. |
| `crates/rustui-gpui-component/src/theme/registry.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/theme/registry.rs`. |
| `crates/rustui-gpui-component/src/theme/schema.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/theme/schema.rs`. |
| `crates/rustui-gpui-component/src/theme/system_font.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/theme/system_font.rs`. |
| `crates/rustui-gpui-component/src/theme/theme_color.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/theme/theme_color.rs`. |
| `crates/rustui-gpui-component/src/time/calendar.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/time/calendar.rs`. |
| `crates/rustui-gpui-component/src/time/date_picker.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/time/date_picker.rs`. |
| `crates/rustui-gpui-component/src/time/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/time/mod.rs`. |
| `crates/rustui-gpui-component/src/time/time_field.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/time/time_field.rs`. |
| `crates/rustui-gpui-component/src/time/utils.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/time/utils.rs`. |
| `crates/rustui-gpui-component/src/title_bar.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/title_bar.rs`. |
| `crates/rustui-gpui-component/src/toolbar.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/toolbar.rs`. |
| `crates/rustui-gpui-component/src/tooltip.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/tooltip.rs`. |
| `crates/rustui-gpui-component/src/touch_selection/edit_menu.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/touch_selection/edit_menu.rs`. |
| `crates/rustui-gpui-component/src/touch_selection/handle.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/touch_selection/handle.rs`. |
| `crates/rustui-gpui-component/src/touch_selection/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/touch_selection/mod.rs`. |
| `crates/rustui-gpui-component/src/touch_selection/window_overlay.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/touch_selection/window_overlay.rs`. |
| `crates/rustui-gpui-component/src/tree.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/tree.rs`. |
| `crates/rustui-gpui-component/src/virtual_list.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/virtual_list.rs`. |
| `crates/rustui-gpui-component/src/window_border.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/src/window_border.rs`. |
| `crates/rustui-gpui-component/src/window_ext.rs` | IN MAIN | WindowExt remains source compatible; approved tracked-dialog extension adds opaque opening identities and targeted close/query operations. |
| `crates/rustui-gpui-component/tests/base_compat.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/tests/base_compat.rs`. |
| `crates/rustui-gpui-component/tests/controlled_change_callbacks.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/tests/controlled_change_callbacks.rs`. |
| `crates/rustui-gpui-component/tests/legacy_button_compat.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/tests/legacy_button_compat.rs`. |
| `crates/rustui-gpui-component/tests/legacy_controls_compat.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/tests/legacy_controls_compat.rs`. |
| `crates/rustui-gpui-component/tests/legacy_language_config_compat.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/tests/legacy_language_config_compat.rs`. |
| `crates/rustui-gpui-component/tests/table_dump_range.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/tests/table_dump_range.rs`. |
| `crates/rustui-gpui-component/tests/theme_compat.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-component/tests/theme_compat.rs`. |
| `crates/rustui-gpui-kit-assets/.cargo-checksum.json` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-kit-assets/.cargo-checksum.json`. |
| `crates/rustui-gpui-kit-assets/.cargo_vcs_info.json` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-kit-assets/.cargo_vcs_info.json`. |
| `crates/rustui-gpui-kit-assets/Cargo.lock` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-kit-assets/Cargo.lock`. |
| `crates/rustui-gpui-kit-assets/Cargo.toml` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-kit-assets/Cargo.toml`. |
| `crates/rustui-gpui-kit-assets/Cargo.toml.orig` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-kit-assets/Cargo.toml.orig`. |
| `crates/rustui-gpui-kit-assets/assets/.gitkeep` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-kit-assets/assets/.gitkeep`. |
| `crates/rustui-gpui-kit-assets/build.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-kit-assets/build.rs`. |
| `crates/rustui-gpui-kit-assets/examples/extra_assets.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-kit-assets/examples/extra_assets.rs`. |
| `crates/rustui-gpui-kit-assets/examples/full_assets.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-kit-assets/examples/full_assets.rs`. |
| `crates/rustui-gpui-kit-assets/examples/selected_assets.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-kit-assets/examples/selected_assets.rs`. |
| `crates/rustui-gpui-kit-assets/lucide.json` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-kit-assets/lucide.json`. |
| `crates/rustui-gpui-kit-assets/src/icon.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-kit-assets/src/icon.rs`. |
| `crates/rustui-gpui-kit-assets/src/lib.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-kit-assets/src/lib.rs`. |
| `crates/rustui-gpui-kit-assets/src/native_assets.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-kit-assets/src/native_assets.rs`. |
| `crates/rustui-gpui-kit-assets/src/wasm_assets.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-kit-assets/src/wasm_assets.rs`. |
| `crates/rustui-gpui-kit-assets/tests/icons.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-kit-assets/tests/icons.rs`. |
| `crates/rustui-gpui-linux/.cargo-checksum.json` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/.cargo-checksum.json`. |
| `crates/rustui-gpui-linux/Cargo.lock` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/Cargo.lock`. |
| `crates/rustui-gpui-linux/Cargo.toml` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/Cargo.toml`. |
| `crates/rustui-gpui-linux/Cargo.toml.orig` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/Cargo.toml.orig`. |
| `crates/rustui-gpui-linux/src/gpui_linux.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/src/gpui_linux.rs`. |
| `crates/rustui-gpui-linux/src/linux.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/src/linux.rs`. |
| `crates/rustui-gpui-linux/src/linux/dispatcher.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/src/linux/dispatcher.rs`. |
| `crates/rustui-gpui-linux/src/linux/display_connection.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/src/linux/display_connection.rs`. |
| `crates/rustui-gpui-linux/src/linux/headless.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/src/linux/headless.rs`. |
| `crates/rustui-gpui-linux/src/linux/headless/window.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/src/linux/headless/window.rs`. |
| `crates/rustui-gpui-linux/src/linux/keyboard.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/src/linux/keyboard.rs`. |
| `crates/rustui-gpui-linux/src/linux/platform.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/src/linux/platform.rs`. |
| `crates/rustui-gpui-linux/src/linux/system_notifications.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/src/linux/system_notifications.rs`. |
| `crates/rustui-gpui-linux/src/linux/text_system.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/src/linux/text_system.rs`. |
| `crates/rustui-gpui-linux/src/linux/wayland.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/src/linux/wayland.rs`. |
| `crates/rustui-gpui-linux/src/linux/wayland/client.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/src/linux/wayland/client.rs`. |
| `crates/rustui-gpui-linux/src/linux/wayland/clipboard.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/src/linux/wayland/clipboard.rs`. |
| `crates/rustui-gpui-linux/src/linux/wayland/cursor.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/src/linux/wayland/cursor.rs`. |
| `crates/rustui-gpui-linux/src/linux/wayland/display.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/src/linux/wayland/display.rs`. |
| `crates/rustui-gpui-linux/src/linux/wayland/layer_shell.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/src/linux/wayland/layer_shell.rs`. |
| `crates/rustui-gpui-linux/src/linux/wayland/popup.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/src/linux/wayland/popup.rs`. |
| `crates/rustui-gpui-linux/src/linux/wayland/serial.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/src/linux/wayland/serial.rs`. |
| `crates/rustui-gpui-linux/src/linux/wayland/window.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/src/linux/wayland/window.rs`. |
| `crates/rustui-gpui-linux/src/linux/x11.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/src/linux/x11.rs`. |
| `crates/rustui-gpui-linux/src/linux/x11/client.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/src/linux/x11/client.rs`. |
| `crates/rustui-gpui-linux/src/linux/x11/clipboard.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/src/linux/x11/clipboard.rs`. |
| `crates/rustui-gpui-linux/src/linux/x11/display.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/src/linux/x11/display.rs`. |
| `crates/rustui-gpui-linux/src/linux/x11/event.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/src/linux/x11/event.rs`. |
| `crates/rustui-gpui-linux/src/linux/x11/window.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/src/linux/x11/window.rs`. |
| `crates/rustui-gpui-linux/src/linux/x11/xim_handler.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/src/linux/x11/xim_handler.rs`. |
| `crates/rustui-gpui-linux/src/linux/xdg_desktop_portal.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-linux/src/linux/xdg_desktop_portal.rs`. |
| `crates/rustui-gpui-macos/Cargo.toml` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-macos/Cargo.toml`. |
| `crates/rustui-gpui-platform/Cargo.toml` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-platform/Cargo.toml`. |
| `crates/rustui-gpui-web/.cargo-checksum.json` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-web/.cargo-checksum.json`. |
| `crates/rustui-gpui-web/Cargo.lock` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-web/Cargo.lock`. |
| `crates/rustui-gpui-web/Cargo.toml` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-web/Cargo.toml`. |
| `crates/rustui-gpui-web/Cargo.toml.orig` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-web/Cargo.toml.orig`. |
| `crates/rustui-gpui-web/src/a11y.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-web/src/a11y.rs`. |
| `crates/rustui-gpui-web/src/a11y_dom.js` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-web/src/a11y_dom.js`. |
| `crates/rustui-gpui-web/src/a11y_dom.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-web/src/a11y_dom.rs`. |
| `crates/rustui-gpui-web/src/canvas_fallback.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-web/src/canvas_fallback.rs`. |
| `crates/rustui-gpui-web/src/canvas_text.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-web/src/canvas_text.rs`. |
| `crates/rustui-gpui-web/src/dispatcher.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-web/src/dispatcher.rs`. |
| `crates/rustui-gpui-web/src/display.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-web/src/display.rs`. |
| `crates/rustui-gpui-web/src/events.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-web/src/events.rs`. |
| `crates/rustui-gpui-web/src/glyph_cache.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-web/src/glyph_cache.rs`. |
| `crates/rustui-gpui-web/src/gpui_web.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-web/src/gpui_web.rs`. |
| `crates/rustui-gpui-web/src/http_client.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-web/src/http_client.rs`. |
| `crates/rustui-gpui-web/src/ime_edit.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-web/src/ime_edit.rs`. |
| `crates/rustui-gpui-web/src/ime_mirror.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-web/src/ime_mirror.rs`. |
| `crates/rustui-gpui-web/src/keyboard.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-web/src/keyboard.rs`. |
| `crates/rustui-gpui-web/src/logging.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-web/src/logging.rs`. |
| `crates/rustui-gpui-web/src/platform.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-web/src/platform.rs`. |
| `crates/rustui-gpui-web/src/run_replacements.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-web/src/run_replacements.rs`. |
| `crates/rustui-gpui-web/src/text_system.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-web/src/text_system.rs`. |
| `crates/rustui-gpui-web/src/viewport.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-web/src/viewport.rs`. |
| `crates/rustui-gpui-web/src/window.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-web/src/window.rs`. |
| `crates/rustui-gpui-wgpu/.cargo-checksum.json` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-wgpu/.cargo-checksum.json`. |
| `crates/rustui-gpui-wgpu/Cargo.lock` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-wgpu/Cargo.lock`. |
| `crates/rustui-gpui-wgpu/Cargo.toml` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-wgpu/Cargo.toml`. |
| `crates/rustui-gpui-wgpu/Cargo.toml.orig` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-wgpu/Cargo.toml.orig`. |
| `crates/rustui-gpui-wgpu/benches/layout_line.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-wgpu/benches/layout_line.rs`. |
| `crates/rustui-gpui-wgpu/src/cosmic_text_system.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-wgpu/src/cosmic_text_system.rs`. |
| `crates/rustui-gpui-wgpu/src/gpui_wgpu.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-wgpu/src/gpui_wgpu.rs`. |
| `crates/rustui-gpui-wgpu/src/shaders.wgsl` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-wgpu/src/shaders.wgsl`. |
| `crates/rustui-gpui-wgpu/src/shaders_storage.wgsl` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-wgpu/src/shaders_storage.wgsl`. |
| `crates/rustui-gpui-wgpu/src/shaders_subpixel.wgsl` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-wgpu/src/shaders_subpixel.wgsl`. |
| `crates/rustui-gpui-wgpu/src/shaders_webgl.wgsl` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-wgpu/src/shaders_webgl.wgsl`. |
| `crates/rustui-gpui-wgpu/src/wgpu_atlas.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-wgpu/src/wgpu_atlas.rs`. |
| `crates/rustui-gpui-wgpu/src/wgpu_context.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-wgpu/src/wgpu_context.rs`. |
| `crates/rustui-gpui-wgpu/src/wgpu_renderer.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-wgpu/src/wgpu_renderer.rs`. |
| `crates/rustui-gpui-windows/.cargo-checksum.json` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/.cargo-checksum.json`. |
| `crates/rustui-gpui-windows/Cargo.lock` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/Cargo.lock`. |
| `crates/rustui-gpui-windows/Cargo.toml` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/Cargo.toml`. |
| `crates/rustui-gpui-windows/Cargo.toml.orig` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/Cargo.toml.orig`. |
| `crates/rustui-gpui-windows/build.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/build.rs`. |
| `crates/rustui-gpui-windows/src/alpha_correction.hlsl` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/src/alpha_correction.hlsl`. |
| `crates/rustui-gpui-windows/src/clipboard.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/src/clipboard.rs`. |
| `crates/rustui-gpui-windows/src/color_text_raster.hlsl` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/src/color_text_raster.hlsl`. |
| `crates/rustui-gpui-windows/src/destination_list.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/src/destination_list.rs`. |
| `crates/rustui-gpui-windows/src/dialog.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/src/dialog.rs`. |
| `crates/rustui-gpui-windows/src/direct_manipulation.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/src/direct_manipulation.rs`. |
| `crates/rustui-gpui-windows/src/direct_write.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/src/direct_write.rs`. |
| `crates/rustui-gpui-windows/src/directx_atlas.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/src/directx_atlas.rs`. |
| `crates/rustui-gpui-windows/src/directx_devices.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/src/directx_devices.rs`. |
| `crates/rustui-gpui-windows/src/directx_renderer.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/src/directx_renderer.rs`. |
| `crates/rustui-gpui-windows/src/dispatcher.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/src/dispatcher.rs`. |
| `crates/rustui-gpui-windows/src/display.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/src/display.rs`. |
| `crates/rustui-gpui-windows/src/events.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/src/events.rs`. |
| `crates/rustui-gpui-windows/src/gpui_windows.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/src/gpui_windows.rs`. |
| `crates/rustui-gpui-windows/src/keyboard.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/src/keyboard.rs`. |
| `crates/rustui-gpui-windows/src/platform.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/src/platform.rs`. |
| `crates/rustui-gpui-windows/src/shaders.hlsl` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/src/shaders.hlsl`. |
| `crates/rustui-gpui-windows/src/system_notifications.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/src/system_notifications.rs`. |
| `crates/rustui-gpui-windows/src/system_settings.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/src/system_settings.rs`. |
| `crates/rustui-gpui-windows/src/util.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/src/util.rs`. |
| `crates/rustui-gpui-windows/src/vsync.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/src/vsync.rs`. |
| `crates/rustui-gpui-windows/src/window.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/src/window.rs`. |
| `crates/rustui-gpui-windows/src/wrapper.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui-windows/src/wrapper.rs`. |
| `crates/rustui-gpui/Cargo.toml` | IN MAIN | Exact Git blob in main at `crates/rustui-gpui/Cargo.toml`. |
| `crates/rustui-metal/.cargo-checksum.json` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/.cargo-checksum.json`. |
| `crates/rustui-metal/.cargo_vcs_info.json` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/.cargo_vcs_info.json`. |
| `crates/rustui-metal/.github/workflows/ci.yml` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/.github/workflows/ci.yml`. |
| `crates/rustui-metal/Cargo.lock` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/Cargo.lock`. |
| `crates/rustui-metal/Cargo.toml` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/Cargo.toml`. |
| `crates/rustui-metal/Cargo.toml.orig` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/Cargo.toml.orig`. |
| `crates/rustui-metal/Makefile` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/Makefile`. |
| `crates/rustui-metal/bors.toml` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/bors.toml`. |
| `crates/rustui-metal/examples/argument-buffer/main.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/argument-buffer/main.rs`. |
| `crates/rustui-metal/examples/bind/main.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/bind/main.rs`. |
| `crates/rustui-metal/examples/bindless/main.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/bindless/main.rs`. |
| `crates/rustui-metal/examples/caps/main.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/caps/main.rs`. |
| `crates/rustui-metal/examples/circle/main.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/circle/main.rs`. |
| `crates/rustui-metal/examples/circle/shaders.metal` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/circle/shaders.metal`. |
| `crates/rustui-metal/examples/compute/compute-argument-buffer.metal` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/compute/compute-argument-buffer.metal`. |
| `crates/rustui-metal/examples/compute/compute-argument-buffer.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/compute/compute-argument-buffer.rs`. |
| `crates/rustui-metal/examples/compute/embedded-lib.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/compute/embedded-lib.rs`. |
| `crates/rustui-metal/examples/compute/main.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/compute/main.rs`. |
| `crates/rustui-metal/examples/compute/shaders.metal` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/compute/shaders.metal`. |
| `crates/rustui-metal/examples/events/main.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/events/main.rs`. |
| `crates/rustui-metal/examples/fence/main.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/fence/main.rs`. |
| `crates/rustui-metal/examples/headless-render/main.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/headless-render/main.rs`. |
| `crates/rustui-metal/examples/library/main.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/library/main.rs`. |
| `crates/rustui-metal/examples/mesh-shader/main.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/mesh-shader/main.rs`. |
| `crates/rustui-metal/examples/mesh-shader/shaders.metal` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/mesh-shader/shaders.metal`. |
| `crates/rustui-metal/examples/mps/main.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/mps/main.rs`. |
| `crates/rustui-metal/examples/mps/shaders.metal` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/mps/shaders.metal`. |
| `crates/rustui-metal/examples/raytracing/camera.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/raytracing/camera.rs`. |
| `crates/rustui-metal/examples/raytracing/geometry.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/raytracing/geometry.rs`. |
| `crates/rustui-metal/examples/raytracing/main.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/raytracing/main.rs`. |
| `crates/rustui-metal/examples/raytracing/renderer.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/raytracing/renderer.rs`. |
| `crates/rustui-metal/examples/raytracing/scene.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/raytracing/scene.rs`. |
| `crates/rustui-metal/examples/raytracing/shaders.metal` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/raytracing/shaders.metal`. |
| `crates/rustui-metal/examples/reflection/main.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/reflection/main.rs`. |
| `crates/rustui-metal/examples/shader-dylib/main.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/shader-dylib/main.rs`. |
| `crates/rustui-metal/examples/shader-dylib/test_dylib.metal` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/shader-dylib/test_dylib.metal`. |
| `crates/rustui-metal/examples/shader-dylib/test_shader.metal` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/shader-dylib/test_shader.metal`. |
| `crates/rustui-metal/examples/window/main.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/window/main.rs`. |
| `crates/rustui-metal/examples/window/shaders.metal` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/examples/window/shaders.metal`. |
| `crates/rustui-metal/renovate.json` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/renovate.json`. |
| `crates/rustui-metal/src/acceleration_structure.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/acceleration_structure.rs`. |
| `crates/rustui-metal/src/acceleration_structure_pass.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/acceleration_structure_pass.rs`. |
| `crates/rustui-metal/src/argument.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/argument.rs`. |
| `crates/rustui-metal/src/blitpass.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/blitpass.rs`. |
| `crates/rustui-metal/src/buffer.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/buffer.rs`. |
| `crates/rustui-metal/src/capturedescriptor.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/capturedescriptor.rs`. |
| `crates/rustui-metal/src/capturemanager.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/capturemanager.rs`. |
| `crates/rustui-metal/src/commandbuffer.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/commandbuffer.rs`. |
| `crates/rustui-metal/src/commandqueue.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/commandqueue.rs`. |
| `crates/rustui-metal/src/computepass.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/computepass.rs`. |
| `crates/rustui-metal/src/constants.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/constants.rs`. |
| `crates/rustui-metal/src/counters.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/counters.rs`. |
| `crates/rustui-metal/src/depthstencil.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/depthstencil.rs`. |
| `crates/rustui-metal/src/device.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/device.rs`. |
| `crates/rustui-metal/src/drawable.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/drawable.rs`. |
| `crates/rustui-metal/src/encoder.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/encoder.rs`. |
| `crates/rustui-metal/src/heap.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/heap.rs`. |
| `crates/rustui-metal/src/indirect_encoder.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/indirect_encoder.rs`. |
| `crates/rustui-metal/src/lib.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/lib.rs`. |
| `crates/rustui-metal/src/library.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/library.rs`. |
| `crates/rustui-metal/src/mps.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/mps.rs`. |
| `crates/rustui-metal/src/pipeline/compute.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/pipeline/compute.rs`. |
| `crates/rustui-metal/src/pipeline/mod.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/pipeline/mod.rs`. |
| `crates/rustui-metal/src/pipeline/render.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/pipeline/render.rs`. |
| `crates/rustui-metal/src/renderpass.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/renderpass.rs`. |
| `crates/rustui-metal/src/resource.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/resource.rs`. |
| `crates/rustui-metal/src/sampler.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/sampler.rs`. |
| `crates/rustui-metal/src/sync.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/sync.rs`. |
| `crates/rustui-metal/src/texture.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/texture.rs`. |
| `crates/rustui-metal/src/types.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/types.rs`. |
| `crates/rustui-metal/src/vertexdescriptor.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-metal/src/vertexdescriptor.rs`. |
| `crates/rustui-objc-foundation/.cargo-checksum.json` | IN MAIN | Exact Git blob in main at `crates/rustui-objc-foundation/.cargo-checksum.json`. |
| `crates/rustui-objc-foundation/Cargo.toml` | IN MAIN | Exact Git blob in main at `crates/rustui-objc-foundation/Cargo.toml`. |
| `crates/rustui-objc-foundation/examples/custom_class.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-objc-foundation/examples/custom_class.rs`. |
| `crates/rustui-objc-foundation/examples/example.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-objc-foundation/examples/example.rs`. |
| `crates/rustui-objc-foundation/src/array.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-objc-foundation/src/array.rs`. |
| `crates/rustui-objc-foundation/src/data.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-objc-foundation/src/data.rs`. |
| `crates/rustui-objc-foundation/src/dictionary.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-objc-foundation/src/dictionary.rs`. |
| `crates/rustui-objc-foundation/src/enumerator.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-objc-foundation/src/enumerator.rs`. |
| `crates/rustui-objc-foundation/src/lib.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-objc-foundation/src/lib.rs`. |
| `crates/rustui-objc-foundation/src/macros.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-objc-foundation/src/macros.rs`. |
| `crates/rustui-objc-foundation/src/object.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-objc-foundation/src/object.rs`. |
| `crates/rustui-objc-foundation/src/string.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-objc-foundation/src/string.rs`. |
| `crates/rustui-objc-foundation/src/value.rs` | IN MAIN | Exact Git blob in main at `crates/rustui-objc-foundation/src/value.rs`. |
| `crates/rustui/Cargo.toml` | IN MAIN | Owned dependencies retained at the same path; current manifest adds test-support. |
| `crates/rustui/src/lib.rs` | IN MAIN | Owned public re-exports retained at the same path; current formatting/test modules extend it. |
| `scripts/apply_block_stack_symbol_patch.py` | IN MAIN | Owned block detection retained; current loader handles C2 removal of the block override. |
| `scripts/apply_gpui_base_calendar_date_limits.py` | IN MAIN | Exact Git blob in main at `scripts/apply_gpui_base_calendar_date_limits.py`. |
| `scripts/apply_gpui_base_disabled_semantic_roots.py` | IN MAIN | Owned package detection retained at the same path; current verifier composes later accessibility layers. |
| `scripts/apply_gpui_base_resize_first_move.py` | IN MAIN | Exact Git blob in main at `scripts/apply_gpui_base_resize_first_move.py`. |
| `scripts/apply_gpui_base_slider_endpoint.py` | IN MAIN | Exact Git blob in main at `scripts/apply_gpui_base_slider_endpoint.py`. |
| `scripts/apply_gpui_base_slider_first_move.py` | IN MAIN | Exact Git blob in main at `scripts/apply_gpui_base_slider_first_move.py`. |
| `scripts/apply_gpui_base_slider_snap_position.py` | IN MAIN | Exact Git blob in main at `scripts/apply_gpui_base_slider_snap_position.py`. |
| `scripts/apply_gpui_base_slider_zero_track.py` | IN MAIN | Exact Git blob in main at `scripts/apply_gpui_base_slider_zero_track.py`. |
| `scripts/apply_gpui_calendar_heading_patch.py` | IN MAIN | Exact Git blob in main at `scripts/apply_gpui_calendar_heading_patch.py`. |
| `scripts/apply_gpui_calendar_keyboard_patch.py` | IN MAIN | Exact Git blob in main at `scripts/apply_gpui_calendar_keyboard_patch.py`. |
| `scripts/apply_gpui_calendar_rtl_patch.py` | IN MAIN | Exact Git blob in main at `scripts/apply_gpui_calendar_rtl_patch.py`. |
| `scripts/apply_gpui_date_picker_rtl_patch.py` | IN MAIN | Exact Git blob in main at `scripts/apply_gpui_date_picker_rtl_patch.py`. |
| `scripts/apply_gpui_macos_focused_element_patch.py` | IN MAIN | Owned manifest support retained; current lookup recognizes excluded crates as well as members. |
| `scripts/apply_gpui_select_popup_rtl_patch.py` | IN MAIN | Owned selection support retained; current source stack also accounts for the later selectable-list layer. |
| `scripts/apply_gpui_select_rtl_patch.py` | IN MAIN | Exact Git blob in main at `scripts/apply_gpui_select_rtl_patch.py`. |
| `scripts/apply_gpui_toolbar_rtl.py` | IN MAIN | Exact Git blob in main at `scripts/apply_gpui_toolbar_rtl.py`. |
| `scripts/apply_gpui_tree_accessibility_series.py` | IN MAIN | Owned package support retained; current verifier composes the later validation/disabled Tree layers. |
| `scripts/tests/test_external_consumer_patches.py` | IN MAIN | Exact Git blob in main at `scripts/tests/test_external_consumer_patches.py`. |
| `scripts/tests/test_external_consumer_setup.py` | IN MAIN | Exact Git blob in main at `scripts/tests/test_external_consumer_setup.py`. |
| `scripts/tests/test_gpui_macos_focused_element_patch.py` | IN MAIN | Exact Git blob in main at `scripts/tests/test_gpui_macos_focused_element_patch.py`. |
| `scripts/verify_gpui_calendar_patch_stack.py` | IN MAIN | Exact Git blob in main at `scripts/verify_gpui_calendar_patch_stack.py`. |
| `scripts/verify_owned_gpui_sources.py` | IN MAIN | Exact Git blob in main at `scripts/verify_owned_gpui_sources.py`. |

## USEFUL → proposed tickets

None. No reviewed archived fix or feature is absent from main without an identified retirement or superseding implementation. There are no S11 recovery tickets to add under roadmap item 11. The separate `wip/native-macos-popups` review remains scheduled as item 11b; this ticket does not inspect or delete that branch.

## Retention and deletion boundary

Only the two named remote branches and the single local stash entry named `owner: pre-sync 20261108` are authorized for deletion, after this review PR is merged and the remote archive targets are rechecked. Permanent tags, all backup files, every other stash entry and `wip/native-macos-popups` are retained.
