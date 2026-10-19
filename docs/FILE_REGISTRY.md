# File registry

## Delegated roadmap delivery — 2026-10-18

`docs/IMPLEMENTATION_ROADMAP.md` owns task dependencies, assigned agents, model
choices, acceptance criteria and the distinction between dispatched and queued
work. Task notes extend this registry without creating speculative source modules.

| File | Responsibility / change | Dependencies | API / output | Order / priority / risk | Validation |
| --- | --- | --- | --- | --- | --- |
| `docs/tasks/R1_BINDING_UPDATES.md` | New: navigation binding semantics, limitations and follow-ups | Binding, RenderContext, NavigationStack | R1 implementation record | Wave 1 / foundation / medium | Source review; no behavioral acceptance claimed |
| `docs/tasks/L1_LAB_ACCEPTANCE.md` | New: Lab evidence and acceptance task breakdown | Existing Lab stories and public API | L1 acceptance roadmap | Wave 1 / foundation / low | Source comparison; native checks pending |
| `docs/tasks/P1_PERFORMANCE_PLAN.md` | New: RGPE and candidate-update task plan | Upstream audit, build scripts, patch policy | P1 plan and approval gates | Wave 1 planning / advanced implementation / low planning risk | Source references; no measurements claimed |
| `crates/rustui-core/src/binding.rs` | Modified: mutation starts from current owner value | Existing typed read/write closures | Hidden `Binding::update_on` | R1 / foundation / medium | Formatting and workspace check; repeated mutation checks pending |
| `crates/rustui/src/view.rs` | Modified: bind owner-current mutations through GPUI | Binding and host updater | Internal `RenderContext::binding_updater` | R1 / foundation / medium | Formatting and workspace check; release behavior checks pending |
| `crates/rustui/src/components/navigation/navigation_stack.rs` | Modified: push/pop use current owner path | Binding updater | Public constructors unchanged | R1 / core / medium | Formatting and workspace check; interaction checks pending |
| `apps/rustui-lab/src/code_preview.rs` | Modified: Breadcrumb examples use actual Action fields | StoryCanvas | Displayed source fragment | L1 / core / low | Compared to canvas; strings are not compiled examples |

Exact integration commands and output location are recorded in the roadmap.

## Existing implementation inventory

Dated delivery rows above retain the paths and status that were current when those
milestones were recorded. The current inventory below reflects the story migration:
Lab story modules now live under `apps/rustui-playground/src/stories/`, locale
catalogs under `apps/rustui-playground/locales/`, and `apps/rustui-lab/src/main.rs`
is only the compatibility launcher.

### Roadmap milestone 3 additions

| File | Responsibility / change | Status and evidence |
| --- | --- | --- |
| `crates/rustui/src/components/navigation/navigation_stack.rs` | Modify: RustUI Button presentation, localized Back, explicit missing destination | Compiled; interaction/visual checks pending |
| `locales/{en,fr,ar}/rustui.ftl` | Modify: Back and unavailable-destination messages | Catalog keys added consistently; rendered language acceptance pending |
| `apps/rustui-lab/src/theme_selector.rs` and `main.rs` | Implement: shared macos27 document and Light/Dark activation controls | Compiled; errors go to stderr; GUI pending |
| `crates/rustui/src/components/data/{list,table,tree}.rs` | Modify: current-owner selection guards; Table stale-event filtering | Compiled; value-only selection has documented limitations; GUI pending |
| `docs/tasks/N1_NAVIGATION_POLICY.md` | New: navigation presentation and route fallback contract | Implementation note |
| `docs/tasks/L3A_APPEARANCE_CONTROLS.md` | New: single-pack appearance controls and limitations | Implementation note |
| `docs/tasks/R3A_COLLECTION_SYNC.md` | New: upstream event evidence, selection guards and remaining contracts | Implementation note |

### Roadmap milestone 2 additions

| File | Responsibility / change | Status and evidence |
| --- | --- | --- |
| `crates/rustui/src/components/input/stepper.rs` | Modify: owner-current bounded increment/decrement callbacks | Compiled; interaction checks pending |
| `apps/rustui-lab/src/main.rs` | Modify: searchable, scrollable story browser using RustUI List | Compiled; source-index selection preserved; GUI pending |
| `scripts/record_build_provenance.py` | New: current source/configuration hashes alongside supplied build log | Help/syntax checks and actual evidence recording completed; not a benchmark or reuse verifier |
| `docs/tasks/R1A_STEPPER_UPDATES.md` | New: Stepper semantics and correction record | Implementation note |
| `docs/tasks/L2_STORY_BROWSER.md` | New: story browser contract and pending acceptance | Implementation note |
| `docs/tasks/P2A_BUILD_PROVENANCE.md` | New: provenance CLI usage and limits | Implementation note |

Task dependencies, model assignments and actual integration commands are in
`docs/IMPLEMENTATION_ROADMAP.md`. The statuses below remain subsystem-specific;
new files do not establish full framework acceptance.

This is the implementation status ledger for the standalone RustUI workspace.
Statuses describe code that exists now; a path in this registry is not evidence that
its planned behavior works. RustUI has no Allui dependency or compatibility layer.

Status meanings:

- **Implemented**: the stated contract has working source code.
- **Partial**: useful code exists, but the planned contract is incomplete.
- **Scaffold**: the file or module path exists without the planned runtime behavior.
- **Planning metadata**: data records intent and is not loaded as a runtime pack.
- **Build/configuration utility**: operational workspace support rather than framework API.

The current implementation order is runtime ownership → state and identity → RustUI
Playground stories → initial component hardening → typed themes → localization →
component expansion → advanced framework work.

| File | Purpose | Phase | Prerequisites | Current status |
| --- | --- | --- | --- | --- |
| `.cargo/config.toml` | `playground`, `dev`, `lab`, `warm`, and `audit` aliases; persistent Headstart cache. | bootstrap | Workspace packages | Configuration; `lab` is a compatibility alias |
| `.gitignore` | Ignore generated build, capture, and Python artifacts. | bootstrap | — | Configuration/documentation |
| `AGENTS.md` | Persistent project boundaries and authorized capability-scoped workflow. | bootstrap | — | Configuration/documentation |
| `apps/rustui-playground/Cargo.toml` | Canonical app package using the public RustUI crate. | application | `rustui` | Implementation in progress; build pending |
| `apps/rustui-playground/src/main.rs` | Playground shell, catalog selection, appearance, and reference panel using RustUI APIs. | application | RustUI public API | Implementation in progress; GUI checks pending |
| `apps/rustui-playground/src/catalog.rs` | Exact 41-key macOS 27 coverage and reference ledger. | application | `themes/macos27.json` | Ledger retained; baseline adapter status recorded separately from platform acceptance |
| `apps/rustui-playground/src/stories/canvas.rs` | Live preview composition for the ten story routes. | application | RustUI public API | Migrated source; interaction and GUI checks pending |
| `apps/rustui-playground/src/stories/component_catalog.rs` | Canonical typed registry for Button, Text, Layout, Input, List, Table, Tree, Tabs, Breadcrumb, and Navigation. | application | — | Migrated source; acceptance pending |
| `apps/rustui-playground/src/stories/{child_component,code_preview,event_inspector,layout_inspector,locale_selector,properties_panel,test_runner,mod}.rs` | Child demo, source display, event history, layout facts, locale controls, properties, verification state, and story wiring. | application | RustUI public API | Migrated source; inspectors remain partial and GUI checks pending |
| `apps/rustui-playground/src/stories/menu_gallery.rs` | Public API action menu story with increment, reset, and disabled callback counter. | application | RustUI public API | M30 bounded native interaction accepted; see menu GUI evidence |
| `apps/rustui-playground/src/stories/toolbar_gallery.rs` | Public API toolbar actions, order, disabled state, and locale control. | application | RustUI Toolbar API | Implemented; native acceptance pending |
| `apps/rustui-playground/src/stories/alert_gallery.rs` | Public API inline Alert visibility, close, variant, and locale controls. | application | RustUI Alert API | M41 bounded native acceptance recorded; remaining visual, keyboard, and full screen-reader checks pending |
| `apps/rustui-playground/src/stories/tooltip_gallery.rs` | Public API Tooltip examples on text and buttons, including a live child action. | application | RustUI public API | M31 native layout and child action observed; tooltip display and dismissal unverified |
| `apps/rustui-playground/src/stories/offset_gallery.rs` | Signed X/Y Offset controls with a stationary sibling layout slot and moved button/input. | application | RustUI `Offset` API | M79 workspace tests/all-targets check and Playground build passed; native interaction, visual, and accessibility acceptance pending |
| `apps/rustui-playground/src/stories/collection_gallery.rs` | 1,000 stable-ID arbitrary-view rows with controlled selection/confirmation, reorder, removal/restoration, caller-owned row values, and lifecycle counters. | application | RustUI `Collection` API | M80 workspace Collection tests (8 passed), all-targets check, Playground build, and bounded native far-row scroll/state-retention acceptance passed; reorder/removal/restore, keyboard/focus, full lifecycle, and accessibility acceptance pending |
| `apps/rustui-playground/locales/{en,fr,ar}/lab.ftl` | Embedded English, French, and Arabic application catalogs. | application | RustUI localization API | Runtime records present; GUI acceptance pending |
| `apps/rustui-lab/Cargo.toml` and `apps/rustui-lab/src/main.rs` | Compatibility package that delegates to `rustui_playground::run()` without a second UI implementation. | application | `rustui-playground` | Compatibility source; build pending |
| `apps/rustui-playground/references/` | Imported macOS 27 SVG assets for side-by-side visual comparison. | application | Catalog source paths | Reference assets, not runtime components or acceptance evidence |
| `docs/RUSTUI_PLAYGROUND.md` | Canonical Playground launch guide, story inventory, and acceptance limits. | documentation | Current application scope | Configuration/documentation |
| `docs/RUSTUI_LAB.md` | Compatibility redirect to the Playground guide. | documentation | Compatibility package | Configuration/documentation |
| `benchmarks/layout/README.md` | Sizing, repeated content, remeasurement, and layout retention; benchmark design boundary. | performance | — | Configuration/documentation |
| `benchmarks/rendering/README.md` | Idle, dragging, scrolling, text, GPU work, and frame-time distributions; benchmark design boundary. | performance | — | Configuration/documentation |
| `benchmarks/state/README.md` | Notification propagation, derived values, and update cost; benchmark design boundary. | performance | — | Configuration/documentation |
| `Cargo.lock` | Cargo-generated dependency resolution; never hand-authored. | bootstrap | All workspace manifests and approved upstream pins | Generated by Cargo |
| `Cargo.toml` | Workspace membership, shared metadata, and local dependency boundaries. | bootstrap | — | Configuration/documentation |
| `crates/rustui-core/Cargo.toml` | rustui-core package boundary. | core | Workspace manifest | Configuration/documentation |
| `crates/rustui-core/src/actions.rs` | Type-erased component actions with always-notify and change-reporting invalidation. | core | GPUI host binding | Implemented baseline |
| `crates/rustui-core/src/async_state.rs` | Loading, success, and error state contracts without a second scheduler. | core | Approved API and upstream integration | Documentation only |
| `crates/rustui-core/src/binding.rs` | Typed owner lens carrying the current body value for controlled component synchronization. | core | Component owner | Implemented baseline |
| `crates/rustui-core/src/environment.rs` | Cloneable type-indexed environment values scoped to a component host. | core | Standard library | Implemented baseline |
| `crates/rustui-core/src/identity.rs` | Stable caller-owned semantic control and collection identity. | core | Standard library | Implemented baseline |
| `crates/rustui-core/src/lib.rs` | Public core module and contract re-exports. | core | Core modules | Partial facade |
| `crates/rustui-core/src/lifecycle.rs` | Reserved for shared lifecycle types; active root hooks live on View and ComponentHost. | core | View runtime | Scaffold |
| `crates/rustui-core/src/observation.rs` | Invalidation decision consumed by GPUI notification. | core | GPUI host binding | Implemented baseline |
| `crates/rustui-core/src/state.rs` | Borrowed field state paired with its typed binding; component retains storage. | core | Binding | Implemented baseline |
| `crates/rustui-i18n/Cargo.toml` | rustui-i18n package boundary. | localization | Workspace manifest | Configuration/documentation |
| `crates/rustui-i18n/src/catalog.rs` | Initial Fluent-compatible catalog parser, registry, and exact/primary/fallback lookup. | localization | Locale and MessageId | Implemented baseline |
| `crates/rustui-i18n/src/direction.rs` | Locale-derived logical left-to-right/right-to-left direction. | localization | Locale | Implemented baseline |
| `crates/rustui-i18n/src/fallback.rs` | Documents the implemented catalog fallback order. | localization | Catalogs | Implemented documentation |
| `crates/rustui-i18n/src/formatting.rs` | Explicit boundary for unsupported interpolation, plural, date, and number formatting. | localization | Future approved formatter | Deferred |
| `crates/rustui-i18n/src/lib.rs` | Public locale, catalog, direction, message, fallback, and validation facade. | localization | i18n modules | Implemented facade |
| `crates/rustui-i18n/src/locale.rs` | Validated and normalized BCP 47 style locale identifiers. | localization | Standard library | Implemented baseline |
| `crates/rustui-i18n/src/message.rs` | Validated message identifiers for the supported catalog subset. | localization | Standard library | Implemented baseline |
| `crates/rustui-i18n/src/validation.rs` | Missing/extra message validation against the fallback catalog. | localization | Catalogs | Implemented baseline |
| `crates/rustui-macros/Cargo.toml` | rustui-macros package boundary. | deferred | Workspace manifest | Configuration/documentation |
| `crates/rustui-macros/src/component.rs` | Generates state accessors, current-value bindings, and typed component actions. | declarative API | rustui-core Action and Binding | Partial |
| `crates/rustui-macros/src/lib.rs` | Exports `#[component]`, `VStack!`, and `HStack!`. | declarative API | component and view-builder expansion | Implemented facade |
| `crates/rustui-macros/src/state.rs` | Optional state macro experiment; no reactive runtime is approved. | deferred | Approved API and upstream integration | Documentation only |
| `crates/rustui-macros/src/view_builder.rs` | Restricted adjacent-child stack grammar, chained builders, nested stacks, and shorthand Button actions. | declarative API | RustUI stack and action APIs | Implemented baseline |
| `crates/rustui-rgpe/Cargo.toml` | rustui-rgpe package boundary. | performance | Workspace manifest | Configuration/documentation |
| `crates/rustui-rgpe/src/compatibility.rs` | Pinned-upstream compatibility and patch prerequisites. | performance | Approved API and upstream integration | Documentation only |
| `crates/rustui-rgpe/src/config.rs` | Explicit optimization capability/configuration contracts. | performance | Approved API and upstream integration | Documentation only |
| `crates/rustui-rgpe/src/diagnostics.rs` | Performance counters and reproducible workload reporting. | performance | Approved API and upstream integration | Documentation only |
| `crates/rustui-rgpe/src/lib.rs` | rustui-rgpe module root. | performance | crates/rustui-rgpe/src/config.rs, crates/rustui-rgpe/src/diagnostics.rs, crates/rustui-rgpe/src/compatibility.rs | Module declarations only |
| `crates/rustui/Cargo.toml` | Public framework package with the approved GPUI snapshot/platform and component dependencies. | bootstrap | Workspace manifest, rustui-core, rustui-macros | Configuration/documentation |
| `crates/rustui/src/accessibility/mod.rs` | accessibility integration. | framework | crates/rustui/src/accessibility/semantics.rs, crates/rustui/src/accessibility/navigation.rs | Module declarations only |
| `crates/rustui/src/accessibility/navigation.rs` | Accessible focus and navigation behavior. | framework | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/accessibility/semantics.rs` | Accessibility roles, labels, values, and actions. | framework | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/adapters/gpui_components.rs` | Pinned GPUI Components integration boundary. | framework | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/adapters/gpui.rs` | Official GPUI integration boundary. | framework | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/adapters/localization.rs` | Compatibility re-export for RustUI localization integration. | framework | RustUI localization module | Implemented facade |
| `crates/rustui/src/adapters/mod.rs` | adapters integration. | framework | crates/rustui/src/adapters/gpui.rs, crates/rustui/src/adapters/gpui_components.rs, crates/rustui/src/adapters/themes.rs, crates/rustui/src/adapters/localization.rs | Module declarations only |
| `crates/rustui/src/adapters/themes.rs` | Maps the active platform palette into all pinned GPUI Components color roles, plus shared font and radius settings. | themes | Parsed ThemeProfile | All 139 `ThemeColor` roles mapped; verified by `scripts/verify_theme_adapter_coverage.py`; native visual acceptance pending |
| `crates/rustui/src/animation/animator.rs` | Animation coordination using GPUI scheduling rather than a second runtime. | deferred | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/animation/mod.rs` | animation integration. | deferred | crates/rustui/src/animation/timing.rs, crates/rustui/src/animation/spring.rs, crates/rustui/src/animation/transition.rs, crates/rustui/src/animation/animator.rs | Module declarations only |
| `crates/rustui/src/animation/spring.rs` | Validated `SpringConfig`, `SpringOpacity`, and `SpringOpacityError` using GPUI retained spring updates. | framework | GPUI `with_spring`; `docs/tasks/SPRING_OPACITY.md` | M57 bounded native child-identity isolation accepted; M60 normal/reduced-motion host-lowering probe passed; no GPUI-frame, alpha, velocity-continuity, or performance claim |
| `crates/rustui/src/animation/timing.rs` | Timing functions using existing GPUI facilities where possible. | deferred | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/animation/transition.rs` | View transition semantics. | deferred | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/components/advanced/canvas.rs` | M61 bounded native Canvas leaf with validated fill/stroke rectangles and straight-line scenes, GPUI clipping, and immutable shared commands. | components | GPUI native canvas; `docs/tasks/CANVAS.md` | Bounded M61 visual, clipping, clip-toggle, and group-label acceptance recorded in `target/headstart/milestone61-canvas-gui.json`; exact-pixel, scale-factor, speech and performance limits remain |
| `crates/rustui/src/components/advanced/code_editor.rs` | Controlled retained GPUI Components EditorState adapter. | components | Stable ID and owner Binding<String> | Implemented; M53 bounded native editing accepted; parser feature approval, syntax/LSP and full accessibility pending |
| `crates/rustui/src/components/advanced/graph_view.rs` | Graph visualization; deferred until requirements and workloads are approved. | deferred | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/components/advanced/mod.rs` | Advanced component exports; Canvas and CodeEditor are implemented while other entries remain separately scoped. | framework | Canvas, ResizablePanel, PropertyInspector, CodeEditor, GraphView, NodeEditor | Partial facade; bounded M61 Canvas drawing/clipping/group-label acceptance recorded; broader visual/accessibility checks remain open; CodeEditor M53 native editing acceptance recorded |
| `crates/rustui/src/components/advanced/node_editor.rs` | Node editing; deferred until requirements and workloads are approved. | deferred | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/components/advanced/property_inspector.rs` | Typed property editor composition. | components | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/components/advanced/resizable_panel.rs` | Resizable panel adapter. | components | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/components/basic/avatar.rs` | GPUI Components avatar adapter. | components | Approved API and upstream integration | Implemented; tests passed; native acceptance pending |
| `crates/rustui/src/components/basic/badge.rs` | GPUI Components badge adapter. | components | Approved API and upstream integration | Implemented; tests passed; native acceptance pending |
| `crates/rustui/src/components/basic/button.rs` | RustUI Button adapter with stable identity, actions, styles, disabled state, accessibility label/help, tab order, focus traversal, and GPUI Components rendering. | components | View lowering, Action, active theme | Implemented baseline; acceptance pending |
| `crates/rustui/src/components/basic/divider.rs` | Semantic separators. | components | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/components/basic/icon.rs` | GPUI Components icon adapter. | components | Approved API and upstream integration | Implemented; tests passed; icon pixels pending asset registration |
| `crates/rustui/src/components/basic/image.rs` | GPUI image loading, sizing, and error states. | components | Approved API and upstream integration | Implemented; tests and native file-image check passed; custom error UI pending |
| `crates/rustui/src/components/basic/label.rs` | Text and icon label composition. | components | Approved API and upstream integration | Implemented; text/secondary native check passed; icon pixels pending assets |
| `crates/rustui/src/components/basic/link.rs` | Accessible link activation. | components | Approved API and upstream integration | Implemented; tests passed; native acceptance pending |
| `crates/rustui/src/components/basic/mod.rs` | basic component family. | framework | crates/rustui/src/components/basic/button.rs, crates/rustui/src/components/basic/text.rs, crates/rustui/src/components/basic/label.rs, crates/rustui/src/components/basic/image.rs, crates/rustui/src/components/basic/icon.rs, crates/rustui/src/components/basic/divider.rs, crates/rustui/src/components/basic/link.rs, crates/rustui/src/components/basic/avatar.rs, crates/rustui/src/components/basic/badge.rs | Module declarations only |
| `crates/rustui/src/components/basic/text.rs` | Literal and catalog-resolved Text with GPUI lowering, size, weight, caller-supplied fallback policy, and accessible label naming. | components | View lowering, localization context, FontFallbackPolicy | M42 bounded multilingual sample rendered; M48 native accessibility-tree names and distinct duplicate labels observed; OS font selection provenance, native announcements, and full keyboard acceptance pending |
| `crates/rustui/src/components/data/collection.rs` | Typed fixed-row-height virtualization over caller-supplied `CollectionItem<T>` snapshots and arbitrary RustUI row views; optional stable-ID selection and confirmation. | components | GPUI uniform-list virtualization, retained row owners, caller bindings | Source implemented; M80 far-index native scrolling and state retention passed; reorder/removal/restore, keyboard/focus, full lifecycle, and accessibility acceptance pending |
| `crates/rustui/src/components/data/data_grid.rs` | Table-based data grid with explicitly reviewed editing behavior. | components | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/components/data/list.rs` | Controlled virtualized string rows with search and source-index selection. | components | Retained entities and GPUI Components List | Implemented baseline; acceptance pending |
| `crates/rustui/src/components/data/mod.rs` | Data component family and public List/Table/Tree/Collection/CollectionItem exports. | framework | crates/rustui/src/components/data/list.rs, crates/rustui/src/components/data/table.rs, crates/rustui/src/components/data/tree.rs, crates/rustui/src/components/data/outline.rs, crates/rustui/src/components/data/data_grid.rs, crates/rustui/src/components/data/collection.rs | Partial facade |
| `crates/rustui/src/components/data/outline.rs` | Tree-based outline composition. | components | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/components/data/table.rs` | Controlled virtualized string cells with retained row selection. | components | Retained entities and GPUI Components DataTable | M76 bounded native source-index selection/normalization checks; broader keyboard/AX/platform/performance acceptance pending |
| `crates/rustui/src/components/data/tree.rs` | Controlled stable-ID hierarchy with retained expansion and virtual scrolling. | components | Retained entities and GPUI Components Tree | M75 bounded native replacement/expansion/stable-selection checks; broader keyboard/AX/platform/performance acceptance pending |
| `crates/rustui/src/components/feedback/empty_state.rs` | Empty-content composition. | components | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/components/feedback/error_state.rs` | Error presentation and retry composition. | components | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/components/feedback/mod.rs` | feedback component family. | framework | crates/rustui/src/components/feedback/progress.rs, crates/rustui/src/components/feedback/spinner.rs, crates/rustui/src/components/feedback/skeleton.rs, crates/rustui/src/components/feedback/empty_state.rs, crates/rustui/src/components/feedback/error_state.rs | Module declarations only |
| `crates/rustui/src/components/feedback/progress.rs` | Determinate and indeterminate Progress, with `.indeterminate(true)` delegating to pinned GPUI Components loading mode (default false). | components | Approved API and upstream integration | M73: 3 focused tests passed; native moving-fill screenshots and 25→50 determinate progression with both labels observed. Reduced motion verified only in upstream source; visual/platform/accessibility limits remain. |
| `docs/tasks/C5_PROGRESS_SPINNER.md` | Progress and Spinner API examples and acceptance boundaries. | M73 bounded native mode acceptance and evidence links. |
| `target/headstart/milestone73-progress-gui.json` | M73 native Progress determinate/indeterminate observations. | Different fill positions observed across two screenshots; determinate 25→50 and both labels present. Reduced-motion OS/UI toggle, exact pixels, cross-platform, timing/performance, keyboard-only, and full screen-reader checks not done. |
| `target/headstart/milestone73-progress-tests.log`, `target/headstart/milestone73-workspace-tests.log` | M73 focused Progress tests and final expanded workspace suite. | 3 focused tests passed; workspace 237 passed, 0 failed across 12 binaries, Cargo exit 0. |
| `target/headstart/milestone73-all-targets-check.log` | M73 locked/offline all-targets check. | Exit 0. |
| `crates/rustui/src/components/feedback/skeleton.rs` | Skeleton placeholder adapter. | components | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/components/feedback/spinner.rs` | Spinner adapter. | components | Approved API and upstream integration | Implemented; AX node present; spinner pixels pending assets |
| `crates/rustui/src/components/input/checkbox.rs` | Controlled checkbox with identity, focus, accessibility, and disabled state. | components | Binding<bool> and GPUI Components Checkbox | Implemented baseline; acceptance pending |
| `crates/rustui/src/components/input/color_picker.rs` | Controlled optional HSLA color selection backed by retained GPUI ColorPickerState. | components | Binding<Option<Hsla>> and GPUI Components ColorSelect | Implemented baseline; acceptance pending |
| `crates/rustui/src/components/input/combo_box.rs` | Controlled searchable single selection backed by retained GPUI SelectState; reconciles native popup selection with the controlled value after option replacement. | components | Binding<String>, retained entities, GPUI Components Select | M78 popup replacement regression fixed and bounded native cases observed; Picker native behavior not separately tested; full keyboard/focus/accessibility acceptance pending |
| `crates/rustui/src/components/input/date_picker.rs` | Controlled optional single-date selection backed by retained GPUI DatePickerState. | components | Binding<Option<NaiveDate>> and GPUI Components DatePicker | Implemented baseline; acceptance pending |
| `crates/rustui/src/components/input/mod.rs` | input component family. | framework | crates/rustui/src/components/input/text_field.rs, crates/rustui/src/components/input/secure_field.rs, crates/rustui/src/components/input/text_editor.rs, crates/rustui/src/components/input/search_field.rs, crates/rustui/src/components/input/checkbox.rs, crates/rustui/src/components/input/toggle.rs, crates/rustui/src/components/input/radio.rs, crates/rustui/src/components/input/slider.rs, crates/rustui/src/components/input/stepper.rs, crates/rustui/src/components/input/picker.rs, crates/rustui/src/components/input/combo_box.rs, crates/rustui/src/components/input/date_picker.rs, crates/rustui/src/components/input/color_picker.rs | Module declarations only |
| `crates/rustui/src/components/input/picker.rs` | Controlled string selection backed by retained GPUI SelectState. | components | Retained entities and GPUI Components Select | Implemented baseline; acceptance pending |
| `crates/rustui/src/components/input/radio.rs` | Controlled Boolean radio with identity, focus, accessibility, and disabled state. | components | Binding<bool> and GPUI Components Radio | Implemented baseline; acceptance pending |
| `crates/rustui/src/components/input/search_field.rs` | Search text field with upstream clear affordance. | components | TextField retained path | Implemented baseline; acceptance pending |
| `crates/rustui/src/components/input/secure_field.rs` | Password-semantic masked text field with reveal control. | components | TextField retained path | Implemented baseline; acceptance pending |
| `crates/rustui/src/components/input/slider.rs` | Controlled single-value slider with retained GPUI SliderState and Binding<f32>. | components | Retained entities and GPUI Components Slider | Implemented baseline; acceptance pending |
| `crates/rustui/src/components/input/stepper.rs` | Bounded numeric Binding<i32> control composed from GPUI Components buttons. | components | Binding<i32> and GPUI Components Button | Implemented baseline; acceptance pending |
| `crates/rustui/src/components/input/text_editor.rs` | Controlled multiline editor using retained GPUI TextareaState and Binding<String>. | components | Retained entities and GPUI Components input | Implemented baseline; acceptance pending |
| `crates/rustui/src/components/input/text_field.rs` | Controlled single-line input using retained GPUI InputState and Binding<String>. | components | Retained entities and GPUI Components input | Implemented baseline; acceptance pending |
| `crates/rustui/src/components/input/toggle.rs` | Controlled Boolean switch with identity, focus, accessibility, and disabled state. | components | Binding<bool> and GPUI Components Switch | Implemented baseline; acceptance pending |
| `crates/rustui/src/components/mod.rs` | Public component families. | framework | crates/rustui/src/components/basic/mod.rs, crates/rustui/src/components/input/mod.rs, crates/rustui/src/components/data/mod.rs, crates/rustui/src/components/navigation/mod.rs, crates/rustui/src/components/overlay/mod.rs, crates/rustui/src/components/feedback/mod.rs, crates/rustui/src/components/advanced/mod.rs | Module declarations only |
| `crates/rustui/src/components/navigation/breadcrumb.rs` | Ordered optional component actions over GPUI Components Breadcrumb. | components | RustUI actions and GPUI Components Breadcrumb | Implemented baseline; acceptance pending |
| `crates/rustui/src/components/navigation/mod.rs` | Navigation component family and public Breadcrumb, Sidebar, TabView, and NavigationStack exports. | framework | Navigation adapters | Implemented facade |
| `crates/rustui/src/components/navigation/navigation_split.rs` | Controlled stable-ID destination selection and selected-detail composition; disabled sidebar input, empty detail for absent/unknown selection, inactive detail native controls released; horizontal GPUI Components resizable split, no adaptive collapse. | components | `Binding<Option<String>>`, `Sidebar`, `SplitView` | M78 bounded native routing checks passed in the Playground; resize drag, keyboard, locale, full accessibility, and direct allocation-release instrumentation remain pending |
| `crates/rustui/src/components/navigation/navigation_stack.rs` | Bound string route path, heterogeneous destinations, links, and back navigation. | components | Binding<Vec<String>>, AnyView, GPUI Components Button | Implemented baseline; acceptance pending |
| `crates/rustui/src/components/navigation/sidebar.rs` | Controlled flat semantic-ID selection, labels/localization, disabled guards, and current-owner Up/Down/Home/End/Enter handling over GPUI Components Sidebar. The wrapper fills parent width and exposes no width setter; selected-row active-descendant is not emitted. | components | Binding<Option<String>>, stable RustUI IDs, GPUI focus | M79 workspace tests/all-targets check and Playground build passed. Earlier M37 native selected/reorder/relabel/disabled/remove-restore checks remain bounded; new keyboard/focus and parent-width behavior lack native acceptance; full screenreader acceptance pending |
| `crates/rustui/src/components/navigation/tab_view.rs` | Controlled stable-ID tabs with selected RustUI child composition. | components | Binding<String> and GPUI Components TabBar | Implemented baseline; acceptance pending |
| `crates/rustui/src/components/navigation/toolbar.rs` | Ordered stable-ID RustUI action buttons over GPUI Components Toolbar. | components | ControlAction, themed Button, pinned upstream Toolbar | Implemented; keyboard/focus/accessibility and visual acceptance pending |
| `crates/rustui/src/components/overlay/alert.rs` | Inline owner-controlled alert with title/message localization and semantic variants. | components | Binding<bool>, pinned upstream Alert | M41 bounded native close, locale, visibility, and variant appearance observations recorded; remaining checks pending |
| `crates/rustui/src/components/overlay/context_menu.rs` | Right-click/long-press action menu around a RustUI child. | components | Pinned GPUI Components ContextMenuExt and shared MenuItem validation | Implemented; bounded M43 native action, disabled guard, reorder, and in-menu owner-removal observations recorded |
| `crates/rustui/src/components/overlay/dialog.rs` | Dialog and modal presentation. | components | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/components/overlay/drawer.rs` | Drawer presentation composed from existing panel or sheet behavior. | components | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/components/overlay/menu.rs` | Action menu adapter over upstream GPUI Components PopupMenu. | components | RustUI owner actions and upstream popup | Implemented; M30 bounded native interaction accepted; native row IDs remain positional |
| `crates/rustui/src/components/overlay/mod.rs` | overlay component family. | framework | crates/rustui/src/components/overlay/alert.rs, crates/rustui/src/components/overlay/dialog.rs, crates/rustui/src/components/overlay/sheet.rs, crates/rustui/src/components/overlay/popover.rs, crates/rustui/src/components/overlay/tooltip.rs, crates/rustui/src/components/overlay/menu.rs, crates/rustui/src/components/overlay/context_menu.rs, crates/rustui/src/components/overlay/toast.rs, crates/rustui/src/components/overlay/drawer.rs | Module declarations only |
| `crates/rustui/src/components/overlay/popover.rs` | Controlled open state and same-owner RustUI content over GPUI Components Popover. | components | Binding<bool>, themed Button trigger, popover retention scope | Implemented; M36 native editing, owner/child lifecycle, locale/environment, Escape/outside click, and trigger width observed; trigger keyboard/full focus/screenreader acceptance pending |
| `crates/rustui/src/components/overlay/sheet.rs` | Sheet presentation. | components | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/components/overlay/toast.rs` | Window-scoped in-app notification builder with semantic kind, replacement ID, autohide, body action, and native action button. | components | AppAction, pinned upstream Notification/WindowExt | Implemented; newer action-button naming build/test and native acceptance pending |
| `crates/rustui/src/components/overlay/tooltip.rs` | Localized visual tooltip wrapper over GPUI Div.tooltip and GPUI Components Tooltip. | components | RustUI View lowering and upstream tooltip APIs | Implemented; explicit wrapper ID required; native hover/dismissal unverified |
| `crates/rustui/src/constructors.rs` | Ergonomic constructors introduced only after their component APIs are approved. | framework | Approved API and upstream integration | Implemented as reexports of available function constructors |
| `crates/rustui/src/interaction/drag_drop.rs` | M66 typed same-process `DragSource`, `DropTarget`, `DropEvent<T>`, and Copy/Transfer intent over native GPUI drag/drop. | framework | Pinned GPUI drag APIs, owner binding, stable IDs | Build and 221-test M66–M68 suite passed; bounded native pointer flow recorded in `target/headstart/milestone66-drag-drop-gui.json`; keyboard-only and in-flight owner-destruction cases pending |
| `crates/rustui/src/interaction/focus.rs` | Retained GPUI FocusHandle scope, focus binding, Group label, and optional scope tab stop. | framework | Owner Binding<bool>, stable scope ID | Implemented; M52 bounded native acceptance; removal binding policy, optional tab-stop and full screen-reader acceptance pending |
| `crates/rustui/src/interaction/gestures.rs` | Native GPUI enter/exit callbacks wrapped in public `HoverRegion`; validates callback compatibility and retains the child scope. | framework | GPUI pointer events; `docs/tasks/HOVER_REGION.md` | M59 bounded native acceptance recorded; pointer-only dwell, AppAction, nested regions, stale-owner, and full platform/accessibility coverage remain unverified |
| `crates/rustui/src/interaction/keyboard.rs` | Focus-descendant single-stroke shortcut wrapper and action dispatch. | framework | GPUI key events; keyboard-compatible `Action` / `AppAction`; `docs/tasks/KEYBOARD_SHORTCUTS.md` | M56 bounded native acceptance recorded; conflict precedence, AppAction, nested scopes, repeat suppression, real IME, sibling state isolation, and full keyboard/screen-reader coverage remain open |
| `crates/rustui/src/interaction/mod.rs` | Interaction exports, including FocusScope, KeyboardShortcuts, and HoverRegion. | framework | FocusScope, KeyboardShortcuts, HoverRegion, gestures, drag/drop | Partial facade; FocusScope M52, KeyboardShortcuts M56, and HoverRegion M59 bounded native acceptance recorded; other interaction modules remain scoped separately |
| `crates/rustui/src/layout/alignment.rs` | Logical alignment including leading and trailing edges. | framework | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/layout/card.rs` | M68 themed Card composed from Panel Container and Shadow; private `Rc` forwarding adapter preserves borrowed `View` lowering. | framework | Existing Container, Shadow, Card retention scope | Build and seven focused Card tests included in the 221-test M66–M68 suite; native visual/input acceptance pending |
| `crates/rustui/src/layout/container.rs` | General styled layout box, including validated static opacity. | framework | GPUI Styled::opacity and existing Container builders | Implemented; bounded M45 visual comparison recorded; pixel alpha/hit testing unverified |
| `crates/rustui/src/layout/geometry.rs` | Public `GeometryReader` and `GeometrySize` supply resolved container dimensions to a RustUI child factory. | framework | GPUI container query and retained child scope; `docs/tasks/GEOMETRY_READER.md` | M58 bounded native acceptance at explicit 280/540×240 sizes; intrinsic/unbounded/zero GUI behavior and listed isolation/accessibility limits remain unverified |
| `crates/rustui/src/layout/grid.rs` | Grid layout. | framework | Approved API and upstream integration | Implemented; tests and native English/Arabic placement passed |
| `crates/rustui/src/layout/group.rs` | M63 ordered child-list builder; nested groups flatten and receiving layouts splice children directly, without Group implementing View. | framework | `HStack`, `VStack`, `Grid`, `ZStack`; `docs/tasks/GROUP.md` | Implemented; bounded native placement and keyed reorder/removal/restoration accepted; listed visual/RTL/accessibility limits remain |
| `crates/rustui/src/layout/hstack.rs` | Ordered horizontal composition with spacing, vertical alignment, and RTL flow. | declarative API | View lowering and layout direction | Implemented baseline |
| `crates/rustui/src/layout/lazy_grid.rs` | Fixed-column grid with GPUI uniform_list row virtualization and retained cell owners. | framework | Stable snapshot IDs, owner RenderSeed, GPUI list callbacks | Implemented; M54 corrected-paint/state acceptance; counts are not benchmark evidence; positional IDs and keyboard/screen-reader limits remain |
| `crates/rustui/src/layout/mod.rs` | Layout exports, including M63 Group, M62 ScrollAxis/ScrollView, GeometryReader, eager Grid, and virtualized LazyGrid. | framework | VStack, HStack, Group, ScrollView, GeometryReader, Grid, LazyGrid, Spacer, SplitView, ZStack | Partial facade; M63 Group bounded native acceptance recorded; other layout status remains per row |
| `crates/rustui/src/layout/scroll_view.rs` | M62 native GPUI viewport with vertical/horizontal/both axes, retained handle, bounded dimension builders, and optional upstream overlay scrollbars. | framework | GPUI div overflow/ScrollHandle, upstream Scrollbar; `docs/tasks/SCROLL_AXES.md` | Implemented; 191 tests passed; bounded GUI observations recorded; horizontal input, shell regressions, diagonal input and full accessibility remain unverified |
| `crates/rustui/src/layout/spacer.rs` | Flexible layout space. | framework | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/layout/split_view.rs` | Resizable layout primitive independent of navigation history. | framework | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/layout/vstack.rs` | Ordered vertical composition with spacing and logical start/end cross-axis alignment. | declarative API | View lowering and layout direction | Implemented baseline |
| `crates/rustui/src/layout/zstack.rs` | Overlay stack layout. | framework | Approved API and upstream integration | Implemented; tests and centered overlay native check passed |
| `crates/rustui/src/lib.rs` | Public module root, macro re-exports, and GPUI `App` re-export for launch and View lifecycle callbacks. | bootstrap | Workspace modules, rustui-macros, GPUI App | Implemented facade; M25 release acceptance recorded |
| `crates/rustui/src/localization/mod.rs` | Active catalog/locale/direction GPUI global and runtime switching API. | localization | rustui-i18n, GPUI Components | Implemented baseline |
| `crates/rustui/src/localization/gpui_adapter.rs` | Synchronizes the active RustUI locale with GPUI Components. | localization | GPUI Components | Implemented baseline |
| `crates/rustui/src/localization/invalidation.rs` | Observes locale globals and notifies mounted component hosts. | localization | GPUI global observation | Implemented baseline |
| `crates/rustui/src/modifiers/background.rs` | M64 retained identity-scoped native semantic surface box for arbitrary RustUI views. | framework | `Surface`, `RetentionScope::Background`; `docs/tasks/VIEW_DECORATION.md` | Implemented; bounded native independent-state, theme and RTL acceptance recorded; listed visual/accessibility limits remain |
| `crates/rustui/src/modifiers/border.rs` | M64 retained identity-scoped native border box with validated width/radius and active palette colors. | framework | `BorderColor`, `RetentionScope::Border`; `docs/tasks/VIEW_DECORATION.md` | Implemented; bounded native independent-state, theme and RTL acceptance recorded; listed visual/accessibility limits remain |
| `crates/rustui/src/modifiers/frame.rs` | Frame fixed/min/max size constraints and child alignment mapped to GPUI native layout. | framework | GPUI Styled sizes and flex alignment | Implemented; M74 bounded native Center/Stretch/Baseline, retained text/Undo, and action evidence. No exact pixels, peer-baseline placement, full keyboard/AX, RTL, or cross-platform claim. |
| `crates/rustui/src/modifiers/interaction.rs` | Interaction modifiers that preserve control semantics. | framework | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/modifiers/mod.rs` | Modifier module declarations; public Background/Border exports are provided through the prelude. | framework | Frame, Background, Border, padding, shadow, transform, opacity, interaction | Background/Border wrappers implemented; M64 bounded native acceptance recorded; other modules vary by implementation status |
| `crates/rustui/src/modifiers/opacity.rs` | Opacity and composition semantics. | framework | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/modifiers/padding.rs` | Logical padding mapped to GPUI style. | framework | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/modifiers/shadow.rs` | M67 theme-backed elevation and validated explicit native outer shadow. | framework | Pinned GPUI BoxShadow and GPUI Components theme shadow tokens | Build and six focused Shadow tests included in the 221-test M66–M68 suite; bounded Dark-mode CUA observations recorded in `target/headstart/milestone67-shadow-gui.json`; Light appearance and remaining acceptance pending |
| `crates/rustui/src/modifiers/transform.rs` | `Offset`/`OffsetView` apply finite signed logical-pixel relative positioning; layout slot stays in place and GPUI supplies translated hit-test/accessibility geometry. No scale or rotation. | framework | GPUI relative positioning | Source implemented; native visual, interaction, hit-testing, and accessibility acceptance pending |
| `crates/rustui/src/prelude.rs` | Curated public exports, including M63 Group, M64 Background/Border, M66–M68 drag/drop, Shadow, Card, plus Offset and typed Collection. | declarative API | Current public APIs | M79 workspace tests/all-targets check and Playground build passed; M80 bounded native Collection scroll/state-retention acceptance passed; Offset and broader Collection acceptance remain pending |
| `crates/rustui/src/runtime/application.rs` | `RustUI` bootstrap for GPUI Components initialization, launch setup, root window mounting, and activation. | runtime | GPUI platform, window bootstrap | Implemented |
| `crates/rustui/src/runtime/component_host.rs` | Root ownership, lifecycle, locale observation, and keyed retained GPUI entity/subscription storage. | runtime | GPUI entity context, rustui-core | Implemented baseline |
| `crates/rustui/src/runtime/events.rs` | Cloneable application actions update GPUI globals; context-only actions are keyboard-compatible and click-only actions are rejected in shortcut scopes. | runtime | GPUI App | M56 keyboard-compatible action contract; 167 tests passed across 12 binaries; native coverage of AppAction remains pending |
| `crates/rustui/src/runtime/invalidation.rs` | GPUI notification integration; no independent reactive scheduler. | framework | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/runtime/mod.rs` | Runtime module root and public `RustUI`, `ComponentHost`, `ViewHost`, and `Action` exports. | runtime | application, component host, rustui-core | Partial facade |
| `crates/rustui/src/runtime/tasks.rs` | Foreground/background tasks, cancellation, and weak entity updates. | framework | Approved API and upstream integration | Implemented; generation tests and native start/restart/cancel passed |
| `target/headstart/milestone72-build.log` | M72 Owner Tasks Review build. | Exit 0. |
| `target/headstart/milestone72-owner-tasks-gui.json` | M72 native owner-task restart/cancel/error/release/restore observations. | All recorded checks passed; pending child worker finished with no completion delivered and one unmount; restored child was Idle with two mounts. Blocking-worker interruption and private registry allocation cleanup were not established. |
| `crates/rustui/src/runtime/window.rs` | Internal root window creation and GPUI Components root mounting. | runtime | ComponentHost, GPUI | Implemented baseline |
| `crates/rustui/src/theme/loader.rs` | Reads, deserializes, validates, and serializes RustUI theme documents. | themes | ThemeDocument, ThemeSerializeError | Implemented; M26 workspace run passed 82 tests across 12 binaries |
| `crates/rustui/src/theme/mod.rs` | Public theme loader, validated JSON serializer, schema, and adapter facade. | themes | schema, loader, adapter | Partial facade; `serialize_theme` and `ThemeSerializeError` exported |
| `crates/rustui/src/theme/registry.rs` | Activates complete documents, updates GPUI Components, and stores resolved theme including effective `ActiveTheme.mode` in GPUI globals. | themes | resolver and adapter | Implemented; M65 bounded appearance-mode polling behavior recorded |
| `crates/rustui/src/theme/resolver.rs` | Resolves typed Button appearance rules and metric aliases with explicit interaction fallback. | themes | ThemeDocument | Implemented Button baseline |
| `crates/rustui/src/theme/schema.rs` | Versioned platform profile and typed, validated Button component record. | themes | serde | Implemented Button baseline |
| `crates/rustui/src/theme/tokens.rs` | Typed semantic theme tokens. | framework | Approved API and upstream integration | Documentation only |
| `crates/rustui/src/theme/watcher.rs` | Transactional explicit theme-file reload with content/mode comparison and last-valid-state preservation. | framework | `ThemeFile::reload_if_changed`; `docs/tasks/AUTO_THEME_RELOAD.md` | Implemented; M65 Playground polling accepted for bounded valid/edit/error/recovery/stop cases; watcher/performance and isolation limits remain |
| `crates/rustui/src/view.rs` | View lowering context with actions, bindings, retained entities and observations, environment, theme, locale, direction, messages, and AnyView erasure. | runtime | ComponentHost, GPUI | Implemented baseline |
| `docs/adr/README.md` | Preserve the explicit ADR approval requirement. | design | — | Configuration/documentation |
| `docs/ARCHITECTURE.md` | Layer ownership, dependency graph, and proposed decisions. | documentation | Approved architecture and upstream audit | Configuration/documentation |
| `docs/BUILDING.md` | Headstart, persistent artifact caching, and verification commands. | documentation | Approved architecture and upstream audit | Configuration/documentation |
| `docs/COMPONENT_CATALOG.md` | Planned component inventory and upstream ownership choices. | documentation | Approved architecture and upstream audit | Configuration/documentation |
| `docs/FILE_REGISTRY.md` | Per-file purpose, phase, prerequisites, and implementation status. | documentation | Approved scaffold | Configuration/documentation |
| `docs/FILE_TREE.txt` | Complete scaffold path inventory. | documentation | File registry | Configuration/documentation |
| `docs/IMPLEMENTATION_ROADMAP.md` | Implementation order and next file proposal. | documentation | Approved architecture and upstream audit | Configuration/documentation |
| `docs/PUBLIC_API.md` | Public API contracts for Offset, typed virtualized Collection, Sidebar keyboard/width behavior, and existing public boundaries. | documentation | Current framework source | Source contract updated; native acceptance for the new Offset/Collection and Sidebar keyboard/width behavior remains pending |
| `docs/RUSTUI_LAB.md` | Standalone Lab responsibilities and delivery order. | documentation | Approved architecture and upstream audit | Configuration/documentation |
| `docs/UPSTREAM_AUDIT.md` | Verified source identities, compatibility findings, and approved snapshot selection. | documentation | Approved architecture and upstream audit | Configuration/documentation |
| `examples/counter/README.md` | State updates and bindings; example reserved until APIs exist. | examples | — | Configuration/documentation |
| `examples/data_table/README.md` | Virtualized table behavior; example reserved until APIs exist. | examples | — | Configuration/documentation |
| `examples/forms/README.md` | Input ownership, validation, focus, and localization; example reserved until APIs exist. | examples | — | Configuration/documentation |
| `examples/hello_world/README.md` | First native window and text; example reserved until APIs exist. | examples | — | Configuration/documentation |
| `examples/navigation/README.md` | Stable navigation state and destinations; example reserved until APIs exist. | examples | — | Configuration/documentation |
| `LICENSE` | MIT license selected by the user. | bootstrap | — | Configuration/documentation |
| `locales/ar/locale.json` | Arabic runtime locale record with RTL direction and catalog format. | localization | Catalog subset v1 | Runtime configuration |
| `locales/ar/README.md` | ar translation status and scope. | localization | — | Configuration/documentation |
| `locales/en/locale.json` | English fallback runtime locale record and catalog format. | localization | Catalog subset v1 | Runtime configuration |
| `locales/en/README.md` | en translation status and scope. | localization | — | Configuration/documentation |
| `locales/fr/locale.json` | French runtime locale record with LTR direction and catalog format. | localization | Catalog subset v1 | Runtime configuration |
| `locales/fr/README.md` | fr translation status and scope. | localization | — | Configuration/documentation |
| `locales/README.md` | Localization target boundaries. | localization | — | Configuration/documentation |
| `patches/README.md` | Controlled upstream patch policy. | performance | — | Configuration/documentation |
| `README.md` | Project entry point and honest implementation status. | documentation | Approved architecture and upstream audit | Configuration/documentation |
| `rust-toolchain.toml` | Pin the already-installed compiler used for this scaffold. | bootstrap | — | Configuration/documentation |
| `scripts/cargo-headstart` | Project-local Headstart launcher with Cargo-visible metadata workaround and persistent cache. | bootstrap | Installed Headstart Cargo/rustc; host rustc; POSIX shell | Build/planning utility |
| `scripts/validate_locales.py` | Validate runtime locale records, catalog syntax, duplicates, empties, and key parity. | localization | Python 3 standard library | Validation utility |
| `scripts/validate_themes.py` | Validate theme planning metadata without claiming runtime support. | bootstrap | Python 3 standard library | Build/planning utility |
| `scripts/verify_theme_adapter_coverage.py` | Fail if pinned GPUI Components `ThemeColor` gains a role without a RustUI mapping. | bootstrap | Python 3 standard library | Adapter coverage guard |
| `scripts/verify_dependency_reuse.py` | Require successful Cargo JSON build evidence and fresh artifacts for named dependencies. | bootstrap | Python 3 standard library | Build/planning utility |
| `tests/accessibility/README.md` | Keyboard, focus, semantics, and native assistive-technology behavior; no acceptance is claimed. | validation | — | Configuration/documentation |
| `tests/integration/README.md` | Public API, identity, lifecycle, bindings, and dependency boundaries; no acceptance is claimed. | validation | — | Configuration/documentation |
| `tests/localization/README.md` | Messages, pluralization, formatting, RTL, bidi, fallback, and runtime switching; no acceptance is claimed. | validation | — | Configuration/documentation |
| `tests/visual/README.md` | Deterministic reference captures and reviewed image comparisons; no acceptance is claimed. | validation | — | Configuration/documentation |
| `themes/android.json` | Planning record for android; not a runtime theme. | themes | Approved runtime theme schema | Planning metadata |
| `themes/base.json` | Planning record for base; not a runtime theme. | themes | Approved runtime theme schema | Planning metadata |
| `themes/ios26.json` | Planning record for ios26; not a runtime theme. | themes | Approved runtime theme schema | Planning metadata |
| `themes/ios27.json` | Planning record for ios27; not a runtime theme. | themes | Approved runtime theme schema | Planning metadata |
| `themes/ipados26.json` | Planning record for ipados26; not a runtime theme. | themes | Approved runtime theme schema | Planning metadata |
| `themes/ipados27.json` | Planning record for ipados27; not a runtime theme. | themes | Approved runtime theme schema | Planning metadata |
| `themes/linux.json` | Planning record for linux; not a runtime theme. | themes | Approved runtime theme schema | Planning metadata |
| `themes/macos26.json` | Planning record for macos26; not a runtime theme. | themes | Approved runtime theme schema | Planning metadata |
| `themes/macos27.json` | Unified macOS 27 runtime theme with platform tokens and RustUI component styles. | themes | `theme.schema.json` | Runtime configuration |
| `themes/README.md` | Explain theme target metadata and the runtime implementation gate. | themes | — | Configuration/documentation |
| `themes/web.json` | Planning record for web; not a runtime theme. | themes | Approved runtime theme schema | Planning metadata |
| `themes/windows11.json` | Planning record for windows11; not a runtime theme. | themes | Approved runtime theme schema | Planning metadata |
| `themes/windows12.json` | Planning record for windows12; not a runtime theme. | themes | Approved runtime theme schema | Planning metadata |

## Generated dependency sources and artifacts

`vendor/` contains Cargo-generated versioned third-party package directories, identified
by `Cargo.lock` and their `.cargo-checksum.json` files. They retain upstream licenses
and are outside the first-party scaffold table above. `target/headstart/` contains
ignored build artifacts and evidence; neither directory is a new RustUI API.
## Existing localization and text paths


| File or path | Current status |
| --- | --- |
| `crates/rustui/src/localization/` | Active locale/catalog global, GPUI Components synchronization, locale actions, and host invalidation are implemented. |
| `crates/rustui/src/text/font_policy.rs` | Implemented ordered caller-supplied `FontFallbackPolicy` and validation; `Text`/`LocalizedText` apply it without changing inherited font properties. M42 sample rendered with policy set; OS font selection provenance and other-locale samples pending. |
| `crates/rustui/src/text/backend.rs` | Planned backend scaffold; no independent shaping or rendering runtime. |
| `crates/rustui-i18n/src/message.rs` | Validated message IDs are implemented; message arguments and formatting remain pending. |
| `crates/rustui-i18n/tests/` | Test structure exists; tests were not run for Milestone 6. |
| `crates/rustui/tests/` | Locale switching, bidi, multilingual input, and visual acceptance remain unfinished. |
| `themes/theme.schema.json` | Versioned JSON schema with a required typed Button component definition. |
| `locales/{en,fr,ar}/rustui.ftl` | Runtime framework catalogs for the initial supported subset. |
| `apps/rustui-playground/locales/{en,fr,ar}/lab.ftl` | Runtime application catalogs used by the Playground locale selector. |
| `assets/` | Asset ownership and licensing directory exists. |
| `.cargo/config.toml` aliases | `playground`, `lab`, `warm`, `dev`, and `audit` commands share `target/headstart`; `lab` delegates through the compatibility package. |
| `Cargo.toml` `[profile.headstart]` | Shared development profile for warmup, Playground runs, and workspace checks. |
| `crates/rustui-i18n/tests/catalog_contract.rs` | Initial catalog contract check exists but was not run in this milestone. |


## R2 retention milestone

| Path | Responsibility / change | Evidence and remaining validation |
|---|---|---|
| `crates/rustui/src/runtime/component_host.rs` | Scoped entity/subscription registry, frame visitation and pruning | Workspace check and formatting passed; interaction/lifetime acceptance pending |
| `crates/rustui/src/view.rs` | Private retention scope and preservation APIs | Workspace check and formatting passed |
| `crates/rustui/src/components/navigation/tab_view.rs` | Scope selected tab and preserve declared inactive tabs | Workspace check and formatting passed; GUI acceptance pending |
| `crates/rustui/src/components/navigation/navigation_stack.rs` | Full-prefix page scopes; preserve covered declared pages and release popped pages | Workspace check and formatting passed; GUI acceptance pending |
| `docs/tasks/R2_RUNTIME_RETENTION.md` | Registry behavior, ownership boundaries and limits | Implementation note |
| `docs/tasks/R2_COMPOSITION_RETENTION.md` | Tabs/navigation lifetime policy and explicit container IDs | Implementation note |

No `rustui-core/src/identity.rs` change was needed: scope composition is private to
the host and uses the existing public `Identity` type. No dependency revisions changed.

## Milestone 5 additions

| Path | Change | Status |
|---|---|---|
| `apps/rustui-lab/src/main.rs` | Independent retention-demo state and contextual controls; tab removal selection fallback | Compiled; GUI acceptance pending |
| `apps/rustui-lab/src/canvas.rs` | Compact bound editors in tab/page scopes and conditional content | Compiled; GUI acceptance pending |
| `apps/rustui-lab/src/component_catalog.rs` | Updated story summaries | Compiled |
| `apps/rustui-lab/src/code_preview.rs` | Concrete retention-example APIs | Compiled; displayed snippets not separately compiled |
| `scripts/verify_dependency_reuse.py` | Optional hashed structured report; validated artifacts and source-log overwrite protection | Executed on current successful Cargo check log |
| `docs/tasks/L3B_RETENTION_CONTROLS.md` | Lab behavior boundaries and manual acceptance scope | Implementation note |
| `docs/tasks/P2B_DEPENDENCY_REUSE_REPORT.md` | Reporting schema, command and evidence boundaries | Implementation note |

`properties_panel.rs` needed no code change. No dependencies or vendored sources changed.

## Milestone 6 additions

| Path | Change | Status |
|---|---|---|
| `crates/rustui/src/layout/scroll_view.rs` | M62 native viewport with vertical/horizontal/both axes, retained handle and optional overlay bars | Implemented; 191 tests passed; bounded GUI observations recorded; horizontal gestures and full accessibility remain unverified |
| `crates/rustui/src/layout/mod.rs` | Layout exports, including M62 ScrollAxis/ScrollView, GeometryReader, eager Grid, and virtualized LazyGrid. | framework | VStack, HStack, ScrollView, GeometryReader, Grid, LazyGrid, Spacer, SplitView, ZStack | Partial facade; ScrollView has bounded M62 native evidence; GeometryReader, Grid and LazyGrid retain their own bounded evidence |
| `crates/rustui/src/prelude.rs` | Canvas drawing types and ScrollAxis/ScrollView exports | Implemented; 191 tests passed; bounded Canvas and ScrollView acceptance recorded |
| `apps/rustui-lab/src/main.rs` | Stable-ID scrollable outer host | Compiled; nested-scroll acceptance pending |
| `docs/tasks/C1A_SCROLL_VIEW.md` | Primitive API and upstream evidence | Implementation note |
| `docs/tasks/L3C_SCROLL_HOST.md` | Lab integration and acceptance boundaries | Implementation note |
| `docs/tasks/F1_NESTED_OWNERSHIP_DESIGN.md` | Conditional/keyed composition and later child ownership | First slice implemented; child-host design pending |


## Milestone 7 additions and updates

| Path | Responsibility | Status |
| --- | --- | --- |
| `crates/rustui/src/view.rs` | `Keyed`, `KeyedView`, optional views, paired identity scope helper | Compiled |
| `crates/rustui/src/runtime/component_host.rs` | Per-render duplicate keyed-path detection | Compiled |
| `crates/rustui/src/runtime/scoped_element.rs` | Layout-transparent native GPUI identity boundary | Compiled; GUI acceptance pending |
| `crates/rustui/src/runtime/mod.rs`, `prelude.rs` | Module registration and public exports | Compiled |
| `crates/rustui/src/layout/{vstack,hstack}.rs` | Eager iterable children | Compiled |
| `crates/rustui/src/components/navigation/{tab_view,navigation_stack}.rs` | Paired retention/native identity boundaries | Compiled; GUI acceptance pending |
| `apps/rustui-lab/src/{main,canvas,component_catalog,code_preview}.rs` | Keyed editor reorder and removal/restoration example | Compiled; GUI acceptance pending |
| `docs/tasks/F1A_KEYED_RUNTIME.md` | Runtime implementation record | Implemented slice documented |
| `docs/tasks/F1A_KEYED_COMPOSITION.md` | Stack/navigation implementation record | Implemented slice documented |
| `docs/tasks/F1A_KEYED_LAB.md` | Lab implementation record | Implemented slice documented |


## Milestone 8 additions and updates

| Path | Responsibility | Status |
| --- | --- | --- |
| `crates/rustui/src/view.rs` | `Component`, `ComponentView`, child entity lookup | Compiled |
| `crates/rustui/src/runtime/component_host.rs` | Child entity retention, type replacement, duplicate keys and pruning | Compiled; GUI lifecycle acceptance pending |
| `crates/rustui/src/prelude.rs` | Public child-host exports | Compiled |
| `crates/rustui-core/src/actions.rs` | Wrong-owner action diagnostics | Compiled |
| `crates/rustui-core/src/binding.rs` | Wrong-owner write/update diagnostics, optional read probe retained | Compiled |
| `apps/rustui-lab/src/child_component.rs` | Independent counter/editor component | Compiled; GUI acceptance pending |
| `apps/rustui-lab/src/{main,canvas,component_catalog,code_preview}.rs` | Child demo integration and API example | Compiled; GUI acceptance pending |
| `docs/tasks/F1B_CHILD_RUNTIME.md` | Runtime implementation contract and limits | Implemented baseline documented |
| `docs/tasks/F1B_CHILD_LAB.md` | Child demo contract and limits | Implemented baseline documented |
| `docs/tasks/F1B_OWNER_DIAGNOSTICS.md` | Owner mismatch contract change | Implemented baseline documented |

F1 now includes keyed/optional descriptions and explicit initialized child hosts.
Live props, environment observation and acceptance of lifecycle/focus behavior
remain open; the entire F1 phase is not declared complete.


## Milestone 9 additions and updates

| Path | Responsibility | Status |
| --- | --- | --- |
| `crates/rustui/src/view.rs` | `.props(...)`, `ComponentPropsView`, child projection/notification | Compiled |
| `crates/rustui/src/runtime/component_host.rs` | Previous-props comparison metadata and lifecycle with retained slots | Compiled; runtime acceptance pending |
| `crates/rustui/src/prelude.rs` | Public props-view export | Compiled |
| `apps/rustui-lab/src/{main,canvas,child_component,component_catalog,code_preview}.rs` | Parent-driven title demo preserving child-local state | Compiled; GUI acceptance pending |
| `docs/tasks/F1C_CHILD_PROPS.md` | Live-props contract, equality and callback rules | Implemented baseline documented |
| `docs/tasks/F1C_CHILD_PROPS_LAB.md` | Demo behavior and acceptance limits | Implemented baseline documented |
| `docs/tasks/P3_PATCH_VALIDATION_DESIGN.md` | Isolated candidate workflow, source/checksum boundaries and approval gates | Design only; pipeline not implemented |

F1 now includes explicit live props. Environment observation/reconciliation,
parent-target operations and runtime acceptance remain open.


## Milestone 10 additions and updates

| Path | Responsibility | Status |
| --- | --- | --- |
| `crates/rustui/src/view.rs` | Scoped environment and typed reader; inherited snapshot refresh before child lowering | Compiled |
| `crates/rustui/src/runtime/component_host.rs` | Internal snapshot replacement preserving component state | Compiled; runtime acceptance pending |
| `crates/rustui/src/prelude.rs` | Environment constructors and view exports | Compiled |
| `apps/rustui-lab/src/{main,canvas,child_component,component_catalog,code_preview}.rs` | Shared environment demonstration | Compiled; GUI acceptance pending |
| `docs/tasks/F1D_ENVIRONMENT.md` | Scope, propagation, costs and observation limits | Implemented baseline documented |
| `docs/tasks/F1D_ENVIRONMENT_LAB.md` | Lab demo and pending checks | Implemented baseline documented |
| `docs/tasks/P2_WORKLOAD_PLAN.md` | Proposed native workload, metrics, measurement protocol and thresholds | Design only; no instrumentation or measurements |

Environment propagation is now render-driven. Fine-grained observation,
parent-target operations, cached-child invalidation and runtime acceptance remain
open. The complete foundation/optimization roadmap is not declared finished.
## Milestone 12 runtime measurement files

| Path | Responsibility | Status |
| --- | --- | --- |
| `scripts/runtime_metrics.py` | Shared metric validation and exclusive evidence writer | Implemented |
| `scripts/collect_runtime_metrics.py` | External PID CPU/RSS sampling and partial evidence | Implemented; live smoke recorded |
| `scripts/summarize_runtime_metrics.py` | Validated per-run descriptive summaries | Implemented |
| `scripts/tests/test_collect_runtime_metrics.py` | Collector unit and failure coverage | 8 tests passed |
| `scripts/tests/test_summarize_runtime_metrics.py` | Summary validation and calculation coverage | 9 tests passed |
| `docs/tasks/P2C_PROCESS_COLLECTOR.md` | Collector contract and limitations | Implementation record |
| `docs/tasks/P2C_METRICS_SUMMARY.md` | Summary contract and limitations | Implementation record |
| `docs/tasks/P2C_GPUI_TIMING_HOOKS.md` | Pinned upstream timing-hook review | Read-only audit; no feature enabled |

## Milestone 13 W1 story files

| Path | Responsibility | Status |
| --- | --- | --- |
| `apps/rustui-playground/src/stories/mod.rs` | Manual W1 stepper/reset, expected parent state, dispatch and operator records | Compiles; GUI acceptance pending |
| `apps/rustui-playground/src/stories/canvas.rs` | Child-key reset epoch and workload command props | Compiles; GUI acceptance pending |
| `apps/rustui-playground/src/stories/child_component.rs` | Idempotent props-driven child increments | Compiles; GUI acceptance pending |
| `docs/tasks/P2D_W1_WORKLOAD.md` | W1 use, record semantics, and remaining measurement gaps | Implementation record |

## Milestone 14 acceptance and calibration files

| Path | Responsibility | Status |
| --- | --- | --- |
| `scripts/w1_native_benchmark.js` | Native CUA action sequence and per-step readback assertions | Syntax checked; equivalent runner executed for 90 measured cycles |
| `scripts/collect_w1_baseline.py` | Reuse process collection until a caller stop marker | Three live captures completed |
| `scripts/collect_runtime_metrics.py` | Optional stop-request callback with final sample | Existing behavior retained; new regression coverage |
| `scripts/tests/test_collect_runtime_metrics.py` | Stop-boundary regression test | Metric suite: 18 passed |
| `scripts/summarize_w1_baseline.py` | Validate paired process/native evidence and compute calibration | Live report and three rejection probes passed |
| `docs/tasks/P2E_W1_ACCEPTANCE_BASELINE.md` | GUI acceptance, baseline results, protocol and limits | Completed for bounded W1 CPU/RSS scope |

Story canvas now offers a compact W1 layout, and child action accessibility names
include mounted title/count readback. Existing framework and upstream ownership
boundaries are preserved.

## Owner callbacks, owner tasks, and layout/localization additions

| File | Responsibility | Current evidence |
|---|---|---|
| `crates/rustui/src/runtime/tasks.rs` | Exact-owner GPUI task lifetime, replacement and cancellation | Implemented; generation tests and native start/restart/cancel passed |
| `crates/rustui-core/src/async_state.rs` | Typed idle/loading/success/error value | Compiles without new dependencies |
| `apps/rustui-playground/src/stories/owner_callbacks.rs` | Two same-type parents receiving child callbacks | Four real native clicks verified; JSON observations under target/headstart |
| `apps/rustui-playground/src/stories/owner_tasks.rs` | Start/restart/error/cancel examples | Integrated, native acceptance pending |
| `apps/rustui-playground/src/stories/foundation_additions.rs` | Explicit theme reload plus opt-in M65 350 ms theme-data polling with effective appearance tracking, stop/unmount cancellation, environment-selectable source path, and last-valid-state behavior; Spacer/Divider and logical inset examples | M65 bounded native valid-edit/error-recovery/mode/disable/owner-release observations recorded; exact counters, performance and multi-root limits remain |
| `crates/rustui/src/theme/watcher.rs` | Transactional explicit file reload used by M65 Playground polling | Focused implementation present; bounded M65 valid/edit/error/recovery/stop cases accepted; watcher/performance and isolation limits remain |
| `crates/rustui/src/modifiers/padding.rs` | Validated logical start/end insets | Unit tests executed |
| `crates/rustui-i18n/src/direction.rs` | Script-aware direction and opt-in bidi isolation | Focused tests executed; no shaping engine added |

## Milestone 22 feedback and loading acceptance

| File | Responsibility | Current evidence |
|---|---|---|
| `docs/tasks/C8_EMPTY_ERROR.md` | Empty and error feedback API and acceptance limits | M22 native visual/AX review; retry count 0→1 |
| `docs/tasks/C9_SKELETON.md` | Skeleton adapter API and acceptance limits | M22 dark-card visual review; two named ProgressIndicator nodes |
| `target/headstart/milestone21-22-tests-final.log` | Final unit/integration run | 77 tests passed across 12 binaries |

Native observations do not establish VoiceOver announcements or cadence.

## Milestones 25 and 26 public API, stories, and cache workload

| Path | Responsibility | Current evidence |
|---|---|---|
| `crates/rustui/src/lib.rs` | Re-exports GPUI `App` for `RustUI::on_launch` and `View` lifecycle callback signatures. | M25 Playground release accepted; `target/headstart/milestone25-owner-release-gui.json` |
| `crates/rustui/src/theme/mod.rs` | Exports `serialize_theme` and `ThemeSerializeError` with the typed theme facade. | M26 serialization-focused tests included in 82 passed across 12 binaries; `target/headstart/milestone25-26-tests.log` |
| `crates/rustui/examples/cache_benchmark.rs` | Opt-in frame-stepped fixed-bounds child-cache workload; records host lowering counts and props freshness, not speedup. | M113 small/medium/large props runs passed unchanged-props cache reuse and all changed values 1–100; `target/headstart/evidence/props-cache-acceptance-20261019.json` |
| `apps/rustui-playground/src/stories/task_release.rs` | Conditional child-owner task release and restoration story, exposed by the Release tab. | M25 native acceptance recorded; `target/headstart/milestone25-owner-release-gui.json` |
| `apps/rustui-playground/src/stories/theme_inspector.rs` | Read-only normalized theme JSON preview and Light/Dark Button resolution, embedded in Foundation Additions. | Source test and native preview/reload accepted in M27; see target/headstart/milestone27-theme-inspector-gui.json |
| `apps/rustui-playground/src/stories/mod.rs` | Routes Link, Label, Icon, and Image catalog entries to existing demos; other keys return no catalog demo. | Routing source and catalog coverage present; broader visual acceptance pending |
| `docs/tasks/C10_COMBO_BOX_SELECT.md` | ComboBox searchable Select semantics, M77 regression reproduction, M78 bounded native closeout, and remaining acceptance limits. | M78 popup option-replacement, disabled, external-owner and missing-option cases observed; Picker not separately checked; full keyboard/focus/accessibility remains pending |

## Milestone 30 menu acceptance

| Path | Responsibility | Current evidence |
|---|---|---|
| `apps/rustui-playground/src/stories/menu_gallery.rs` | Menu action counters and disabled-item callback probe | Keyboard and pointer Increment, pointer Reset, disabled callback suppression, and ordinary Button smoke accepted |
| `apps/rustui-playground/src/catalog.rs` | Menu catalog status and description | Updated to bounded M30 interaction acceptance and positional native row identity |
| `docs/tasks/O1_MENU_DROPDOWN.md` | Menu API and acceptance boundary | Native interaction accepted; Enter dismisses a disabled row while callback remains suppressed |
| `target/headstart/milestone30-menu-tests-accepted.log` | Workspace test run | 86 tests across 12 binaries passed; 0 failed |
| `target/headstart/milestone30-menu-gui.json` | Native handoff observations and binary identity | Native outcomes recorded; row identity remains positional |

## Milestone 31 tooltip adapter

| Path | Responsibility | Current evidence |
|---|---|---|
| `apps/rustui-playground/src/stories/tooltip_gallery.rs` | Text and Button tooltip examples; confirms child button remains actionable | Native layout and Button action count 0 to 1 observed; tooltip popup and dismissal not verified |
| `apps/rustui-playground/src/catalog.rs` | Menu route description for combined Menu and Tooltip gallery | Records M30 menu acceptance and bounded M31 observations without claiming tooltip failure |
| `docs/tasks/O2_TOOLTIP.md` | Tooltip API and evidence boundary | Records required ID, localization, visual-only description, and pending hover verification |
| `target/headstart/milestone31-32-tests-fixed.log` | Workspace build/test run | 90 tests passed across workspace; 0 failed |

## Milestones 33–35 inventory and acceptance

| Path | Responsibility | Current evidence |
|---|---|---|
| `scripts/calibrate_cache_workload.py` | Fresh-process whole-workload CPU/RSS calibration runner | M33 protocol and bounded baseline are documented in `docs/tasks/P3C_CACHE_CALIBRATION.md`; results are whole-process metrics, not a cache speedup |
| `docs/tasks/T1C_THEME_REFERENCE_VALIDATION.md` | Format-v1 Button theme token-reference validation | 90 tests across 12 binaries passed in `target/headstart/milestone31-32-tests-fixed.log` |
| `docs/tasks/P3C_CACHE_CALIBRATION.md` | Cache workload calibration protocol and M33 results | Six accepted process records; five measured runs; see `target/headstart/milestone33-cache-calibration.json` and paired logs |
| `docs/tasks/O2_TOOLTIP.md` | Tooltip public contract and bounded native evidence | M31 child layout/action observed; hover display and dismissal remain unverified |
| `docs/tasks/R3B_KEYED_LIST.md` | M34 stable-ID list contract and acceptance scope | 97 tests across 12 binaries passed; native stable selection and row names verified; limits recorded in `target/headstart/milestone34-keyed-list-gui.json` |
| `docs/tasks/A1_OPACITY_ANIMATION.md` | M35 one-shot opacity contract and acceptance scope | Normal and reduced-motion endpoint captures verified; intermediate frames remain pending in `target/headstart/milestone35-opacity-gui.json` |
| `crates/rustui/src/components/data/keyed_list.rs` | Stable-ID string rows over retained GPUI Components `ListState` | M34 insertion, reorder, filtering, removal, keyboard, scroll, and row-name checks recorded; no arbitrary RustUI cells or measured render-call bounds |
| `crates/rustui/src/modifiers/opacity.rs` | Validated one-shot opacity animation using GPUI scheduling | M35 unit validation and endpoint evidence; no intermediate-frame, GPU, or OS-preference claim |
| `apps/rustui-playground/src/stories/keyed_list_gallery.rs` | 1,000-row stable-ID list gallery | `KeyedList` catalog route; native behavior recorded in `target/headstart/milestone34-keyed-list-gui.json` |
| `apps/rustui-playground/src/stories/animation_gallery.rs` | One-shot opacity animation gallery | Endpoint captures only; M35 trajectory acceptance remains pending |
| `apps/rustui-playground/src/stories/mod.rs` | Routes List to `KeyedList` and composes Menu, Tooltip, and animation galleries | Catalog route tests are included in `target/headstart/milestone34-35-tests-accepted.log` |
| `apps/rustui-playground/src/catalog.rs` | M34 keyed-list catalog description | Records stable IDs and bounded string-list scope |
| `target/headstart/milestone34-keyed-list-gui.json` | M34 native observations and binary identity | Stable selection and native row naming observed; outer upstream paths stay positional |
| `target/headstart/milestone35-opacity-gui.json` | M35 endpoint observations and binary identity | Normal/reduced-motion endpoint captures; intermediate frames pending |
| `target/headstart/milestone34-35-tests-accepted.log` | Combined workspace test run for M34/M35 | 97 tests passed across 12 binaries; 0 failed |

## Milestones 36–37 Sidebar and Popover

| Path | Responsibility | Current evidence |
|---|---|---|
| `crates/rustui/src/prelude.rs` | Public `Sidebar`, `SidebarItem`, `SidebarView`, `Popover`, and `PopoverView` exports | Source exports present; bounded M36/M37 native evidence below; complete keyboard/focus/screenreader acceptance pending |
| `crates/rustui/src/components/navigation/sidebar.rs` | Controlled flat Sidebar adapter with semantic IDs, bound selection, localization, and disabled guard | M37 native AX selection, reorder/relabel, disabled pointer, remove/restore, and Inbox checks observed; outer row paths positional; keyboard/focus pending |
| `apps/rustui-playground/src/stories/sidebar_gallery.rs` | Sidebar selection, reorder/relabel, disabled Settings, and remove/restore demonstration | Initial Review.app native route exercised; evidence `target/headstart/milestone37-sidebar-gui.json` |
| `docs/tasks/N2_SIDEBAR.md` | Sidebar source contract, identity boundary, and acceptance checklist | M37 native selection observations remain bounded; new keyboard and parent-width behavior plus full focus/screenreader acceptance remain pending |
| `crates/rustui/src/components/overlay/popover.rs` | Controlled Popover open binding and same-owner content retention | M36 native editing, owner/child lifecycle, environment/locale, Escape/outside click, and intrinsic width observed; trigger keyboard/full focus/screenreader pending |
| `apps/rustui-playground/src/stories/popover_gallery.rs` | Popover owner, child lifetime, inherited environment, localization, and edit controls | Final Review.app native route exercised; evidence `target/headstart/milestone36-popover-gui.json` |
| `docs/tasks/O3_POPOVER.md` | Popover open/close, owner-context, lifecycle, and native acceptance contract | Bounded M36 native observations recorded; trigger Space/Enter/full focus/screenreader acceptance pending |
| `apps/rustui-playground/src/stories/mod.rs` | Routes Sidebar gallery and combines Popover with existing Menu/Tooltip/animation demos | Both M36/M37 native routes exercised; keyboard/focus acceptance remains bounded by task notes |
| `apps/rustui-playground/src/catalog.rs` | Sidebar catalog API label and bounded behavior note | M37 native selection cases observed; keyboard/focus acceptance pending |
| `target/headstart/milestone36-37-tests.log` | Workspace test run for M36/M37 | 102 tests passed across 12 binaries; 0 failed |
| `target/headstart/milestone36-37-build.log`, `target/headstart/milestone36-37-final-build.log` | M36/M37 dev-profile compilation records | Both builds completed; compilation does not establish native GUI acceptance |

## Milestones 43 and 45 ContextMenu and static Container opacity

| Path | Responsibility | Current evidence |
|---|---|---|
| `crates/rustui/src/components/overlay/context_menu.rs` | Public child wrapper using pinned native context menu behavior and validated RustUI actions | M43 bounded right-click, action, disabled guard, ordering, and in-menu owner-removal observations; stale detached action and complete accessibility pending |
| `apps/rustui-playground/src/stories/context_menu_gallery.rs` | Context target with local counters, ordering, and an in-menu gallery-owner removal action | Two reviewed binary hashes and native observations in `target/headstart/milestone43-context-menu-gui.json` |
| `docs/tasks/O6_CONTEXT_MENU.md` | ContextMenu contract and bounded acceptance record | M43 native observations recorded with explicit remaining checks |
| `target/headstart/milestone43-context-menu-gui.json` | M43 native observations and two reviewed binary hashes | Right-click, Increment, disabled suppression, reverse order, in-menu removal, and fresh restore observed |
| `crates/rustui/src/layout/container.rs` | Validated static opacity applied to Container's GPUI box and child | M45 visual comparison and unit validation; pixel alpha and hit testing unverified |
| `apps/rustui-playground/src/stories/animation_gallery.rs` | Adjacent static 1.0 and 0.5 opacity samples beside one-shot animation | Bounded native screenshot observation in `target/headstart/milestone45-static-opacity-gui.json` |
| `docs/tasks/STATIC_CONTAINER_OPACITY.md` | Static Container opacity contract and bounded acceptance | M45 visual observation recorded; pixel alpha and child events unverified |
| `target/headstart/milestone45-static-opacity-gui.json` | M45 native visual observation and reviewed binary hash | Full-opacity sample brighter than half-opacity sample; no pixel measurement |
| `target/headstart/milestone43-45-tests.log` | Combined workspace test run | 120 tests across 12 binaries passed; 0 failed |

## Milestone 48 Text accessibility

| Path | Responsibility | Current evidence |
|---|---|---|
| `crates/rustui/src/components/basic/text.rs` | Name literal and localized Text labels from the resolved visible value | M48 native accessibility tree exposed both labels and locale updates; announcements unverified |
| `crates/rustui/src/view.rs` | Supply owner-scoped positional element IDs for unkeyed Text, including Environment scopes | M48 duplicate equal-text nodes remained distinct; arbitrary conditional reordering unverified |
| `docs/tasks/TEXT_ACCESSIBILITY.md` | Text accessibility contract, positional identity boundary, and bounded acceptance | M48 native observations and remaining checks recorded |
| `target/headstart/milestone48-text-accessibility-gui.json` | M48 native accessibility-tree observations and reviewed executable hash | Literal and localized label names, duplicate nodes, and locale changes observed |
| `target/headstart/milestone46-48-tests-label-fixed.log` | Combined workspace test run after label fix | 132 passed across 12 result groups; 0 failed |


## Milestones 46–50 integration

| Path | Responsibility | Current evidence |
|---|---|---|
| `crates/rustui/src/layout/split_view.rs` | Native resizing and range/topology reconciliation with keyed child owners | M46 native lifecycle/axis/vertical resize and range update checks; post-range drag continuity pending |
| `apps/rustui-playground/src/stories/split_view_gallery.rs` | Public SplitView counters, topology and range controls | Native narrowing/restoring preserved A1 and mount counts |
| `crates/rustui/src/layout/container.rs` | Validated native shadow configuration | M47 bounded Light appearance comparison |
| `docs/tasks/CONTAINER_SHADOW.md` | Shadow API and acceptance limits | Native evidence recorded |
| `crates/rustui/src/runtime/events.rs` | Public motion, refresh and diagnostic action factories | M49 native diagnostic ordering and owner refresh checks |
| `docs/tasks/M49_RUNTIME_ACTIONS.md` | Runtime action contract and observed outcomes | Source, build and native evidence |
| `crates/rustui/src/modifiers/opacity.rs` | Optional animation scheduling limit through native GPUI | Two configuration tests plus four runtime probe modes |
| `crates/rustui/examples/opacity_benchmark.rs` | Isolated normal/reduced/limited scheduling probe | M50 normal40 versus limited10 lowerings; quiescent afterward |
| `scripts/verify_opacity_probe.py` | Validate probe modes and provenance | Four-process runtime run completed |
| `docs/tasks/ANIMATION_FRAME_LIMIT.md` | Scheduling API and bounded measurement interpretation | Executed M50 evidence and hashes |
| `target/headstart/milestone46-50-tests.log` | Workspace integration evidence | 135 passed, zero failed across 12 binaries |


## Milestone 51 accessibility grouping

| Path | Responsibility | Evidence |
|---|---|---|
| `crates/rustui/src/accessibility/semantics.rs` | Stable structural group API and literal/localized metadata | Role/builder/identity checks and native named group |
| `crates/rustui/src/runtime/scoped_element.rs` | Delegate layout/paint and optional native AX node metadata | Actual AccessKit Node property tests; native descendants preserved |
| `crates/rustui/src/view.rs` | Semantics ownership namespace | Duplicate and namespace checks |
| `crates/rustui/src/runtime/component_host.rs` | Reject duplicate semantic sibling identities | Executed duplicate test |
| `apps/rustui-playground/src/stories/semantics_gallery.rs` | Public group, localized region and passthrough examples | Child button0→1 and names Close→Fermer→إغلاق |
| `docs/tasks/SEMANTICS_GROUPS.md` | Contract and bounded acceptance | Native evidence and remaining keyboard/screenreader limits |
| `target/headstart/milestone51-semantics-gui.json` | Reviewed binary and executed observations | 143-test/build/reuse references |

## Milestones 52–55 integration

| Path | Responsibility | Evidence / current boundary |
|---|---|---|
| `apps/rustui-playground/src/stories/focus_gallery.rs` | Public FocusScope request/report, descendant focus, pointer and removal/restoration examples. | M52 bounded native interactions; transient both-true report, removal binding policy, optional tab-stop, and screen-reader limits are recorded in `target/headstart/milestone52-focus-scope-gui.json` and `docs/tasks/FOCUS_SCOPE.md`. |
| `apps/rustui-playground/src/stories/code_editor_gallery.rs` | Controlled source editor language/gutter, readonly/disabled, external replacement and remount examples. | M53 bounded native editing; parser-feature approval, no syntax/LSP, and editor accessibility limits are recorded in `target/headstart/milestone53-code-editor-gui.json` and `docs/tasks/CODE_EDITOR.md`. |
| `apps/rustui-playground/src/stories/lazy_grid_gallery.rs` | 2,400-item three-column virtualized row probe with child/root actions, scrolling, reorder, and lifecycle counters. | Corrected M54 build visibly painted; bounded state/RTL/locale evidence and remaining limits are in `target/headstart/milestone54-lazy-grid-gui.json` and `docs/tasks/LAZY_GRID_IMPLEMENTATION.md`. |
| `apps/rustui-playground/src/stories/frame_gallery.rs` | Public Frame fixed/centered/logical alignment/clamp/zero-size, retained binding, mode cycle, and stateful child-button samples. | M74 bounded native review: retained text through Center/Stretch/Baseline and Undo, Stretch background expansion, Center cycle retention, and child action; inner width 160 plus horizontal Stretch fixed Input holder collapse. Exact-pixel, caret/selection, full AX/keyboard, RTL/platform and peer-baseline claims remain out of scope. See `target/headstart/milestone74-frame-gui.json` and `docs/tasks/FRAME_MODIFIER.md`. |
| `apps/rustui-playground/src/stories/keyboard_gallery.rs` | Public `KeyboardShortcuts` focus-scope demo with inside/outside controls, enabled binding, and typing probes. | M56 bounded native observations: inside/outside focus, activation, disable, typing, and native Undo; limits recorded in `target/headstart/milestone56-keyboard-gui.json` and `docs/tasks/KEYBOARD_SHORTCUTS.md`. |
| `apps/rustui-playground/src/stories/spring_gallery.rs` | Public `SpringOpacity` retargeting, reduced-motion actions, and child activity control. | M57 bounded native qualitative opacity observations remain valid for their reviewed bundle; identity-isolation follow-up accepted in `target/headstart/milestone57-animation-identity-gui.json`, SHA-256 `bc184cdda3d2e0f0e3d134f288a3e808153ee4febc74ac2defbe6410826f8dbd`. Motion/alpha/velocity were not measured by that gallery. See `docs/tasks/SPRING_OPACITY.md`. |
| `apps/rustui-playground/src/stories/geometry_gallery.rs` | Public `GeometryReader` width-responsive composition with persistent and conditional child counters, resize, localization, removal, and external bound-text controls. | M58 bounded native observations and limits recorded in `target/headstart/milestone58-geometry-gui.json` and `docs/tasks/GEOMETRY_READER.md`. |
| `apps/rustui-playground/src/stories/canvas_gallery.rs`, `docs/tasks/CANVAS.md` | M61 public Canvas scene and semantic-summary sample; task contract and acceptance checklist. | Bounded basic drawing, clip-toggle and descriptive accessibility group-label observations recorded in `target/headstart/milestone61-canvas-gui.json`; broader visual/accessibility checks remain unverified. |
| `apps/rustui-playground/src/stories/scroll_gallery.rs`, `docs/tasks/SCROLL_AXES.md` | M62 bounded vertical/horizontal/both-axis, nested, overlay-bar, retained-position, localization, and lifecycle acceptance gallery. | Bounded native observations recorded; horizontal gestures, shell wheel regressions, diagonal input and full accessibility remain unverified. |
| `apps/rustui-playground/src/stories/group_gallery.rs`, `docs/tasks/GROUP.md` | M63 public Group flattening examples across HStack, VStack, Grid and ZStack, with keyed child state/reorder/removal controls. | Bounded direct/group placement and keyed reorder/removal/restoration accepted; exact pixels, RTL matrix, macro grammar, native undo and full accessibility remain unverified. |
| `apps/rustui-playground/src/stories/decoration_gallery.rs`, `docs/tasks/VIEW_DECORATION.md` | M64 Background/Border public composition, stable child IDs, independent retained fields, parent updates, style changes, theme, locale/RTL, and removal/restoration. | Bounded identity, theme, RTL, action and removal observations accepted; exact pixels, layout impact, native undo, full keyboard and screen-reader speech remain unverified. |
| `docs/tasks/AUTO_THEME_RELOAD.md` | M65 opt-in Playground theme-data polling behavior and acceptance procedure. | Bounded valid edit, invalid-file preservation, repair, mode control, disabled polling and owner-release outcomes recorded; listed polling/performance/isolation limits remain. |
| `target/headstart/milestone63-group-gui.json`, `target/headstart/milestone64-decoration-gui.json`, `target/headstart/milestone65-theme-reload-gui.json` | M63–M65 bounded native evidence bundles. | Shared bundle SHA-256 `e917a357fbe391eb428d0cb029d87fecb41cf7aba6e9dc128804618a4dd5d678`; limits are recorded in the matching task contracts. |
| `target/headstart/milestone63-65-final-tests.log` | M63–M65 workspace test suite. | 200 passed, 0 failed across 12 binaries. |
| `target/headstart/milestone63-65-reuse-report.json` | M63–M65 successful build and dependency reuse report. | Build succeeded; all three pinned GPUI dependencies fresh. |
| `target/headstart/milestone63-65-all-targets-check.log` | M63–M65 locked/offline all-targets validation. | Exit 0. |
| `docs/tasks/SERIALIZATION_BRIDGE_PROPOSAL.md` | Proposed framework-owned descriptor bridge boundary for future UI Studio integration. | Proposed only; awaiting user decision; not an accepted ADR and authorizes no implementation. |
| `target/headstart/milestone62-scroll-gui.json` | M62 native CUA screenshots and accessibility-tree observations. | SHA-256 `8ead905f2c20ddc003372a7a509627f54df761fd38f622765bcde80412d6f8ba`; bounded observations and limits are documented in `docs/tasks/SCROLL_AXES.md`. |
| `apps/rustui-playground/src/stories/animation_identity_gallery.rs` | Four shared-ID text fields under sibling SpringOpacity and AnimatedOpacity wrappers, with independent bindings and lifecycle controls. | Bounded M57 native identity isolation accepted; bundle SHA-256 `bc184cdda3d2e0f0e3d134f288a3e808153ee4febc74ac2defbe6410826f8dbd`. Undo grouping and per-field entity lifecycle counts were not measured. |
| `crates/rustui/examples/support/spring_probe.rs`, `scripts/verify_spring_probe.py`, `scripts/test_verify_spring_probe.py`, `docs/tasks/SPRING_OPACITY_PROBE.md` | M60 opt-in normal/reduced-motion SpringOpacity host-lowering probe and verifier. | Final aggregate passed; all four Python verifier tests passed. Measures RustUI root host lowerings only, not GPUI frames, presented alpha, or velocity continuity. |
| `target/headstart/milestone57-animation-identity-gui.json` | Bounded M57 shared-child-ID native isolation observations. | Bundle SHA-256 `bc184cdda3d2e0f0e3d134f288a3e808153ee4febc74ac2defbe6410826f8dbd`; motion/alpha/velocity not measured. |
| `target/headstart/spring-opacity-verification-63179-1792336863069682000.json` | M60 aggregate verifier report for normal and reduced-motion scheduling modes. | Both modes passed; root host lowering counts only. |
| `target/headstart/milestone57-60-final-build.jsonl`, `target/headstart/milestone57-60-final-build.log`, `target/headstart/milestone57-60-final-reuse-report.json` | M57–M60 final Headstart build and dependency reuse evidence. | Build passed; all three pinned dependencies were all-fresh. |
| `target/headstart/milestone57-60-final-all-targets-check.log` | Final M57–M60 locked/offline workspace all-targets check. | Exit 0; unused variable/assignment and future-incompatibility warnings are recorded. |
| `target/headstart/milestone57-59-repairs-tests.log` | Latest repaired Rust workspace test suite. | 180 passed, 0 failed across 12 binaries. |
| `target/headstart/milestone60-probe-verification-final.log`, `target/headstart/milestone60-probe-verification.log` | Final M60 verifier pass and preserved initial exact-float rejection. | Final normal/reduced-motion aggregate passed; initial log preserves its emitted f32 versus decimal comparison. |
| `target/headstart/milestone61-canvas-gui.json` | M61 native CUA screenshots and accessibility-tree observations. | Bundle SHA-256 `ba2979d96f69ed94c6dc1edd1fba285c65e529d452f6154c5016acaf79f5ebce`; basic drawing, canvas-bound clipping, clip toggle, descriptive group labels and outer preview scrolling observed; listed limits remain. |
| `target/headstart/milestone66-68-tests.log` | M66–M68 workspace test run. | 221 passed, 0 failed across 12 binaries. |
| `target/headstart/milestone66-68-build.log`, `target/headstart/milestone66-68-reuse-report.json` | M66–M68 Headstart build and pinned dependency reuse report. | Build succeeded; `gpui-component`, `gpui-pre`, and `gpui-pre-platform` all fresh. |
| `target/headstart/milestone66-68-all-targets-check.log` | Initial M66–M68 locked/offline workspace all-targets check. | Failed on M69 macro-in-progress errors in `task_release.rs:87` and `stories/mod.rs:572`; a later M69–M71 check passed. |
| `target/headstart/milestone66-drag-drop-gui.json` | M66 native CUA pointer drag/drop observations and limits. | Typed copy/drop, transfer, reorder, target removal/restoration, disabled source/target and incompatible numeric payload cases recorded; keyboard-only alternative and actual in-flight owner destruction not tested; no broad accessibility claim. |
| `target/headstart/milestone67-shadow-gui.json` | M67 native CUA Shadow inspection bundle. | Dark appearance, theme elevations, explicit style/offset changes, retained A/B values, omission/restoration, owner action and Arabic RTL observed. Light appearance, native shadow-disable toggle, full keyboard/AX speech and performance remain pending; no saved screenshot image. |
| `target/headstart/milestone69-71-tests.log` | Earlier combined M69–M71 workspace lib/tests run. | 236 passed, 0 failed across 12 binaries; bounded native acceptance for M69–M71 is now complete with feature-specific limits. |
| `target/headstart/milestone69-71-all-targets-check.log` | M69–M71 locked/offline workspace all-targets check. | Exit 0; warnings recorded. M69–M71 bounded native acceptance completed; broader accessibility/keyboard/performance limits remain. |
| `target/headstart/milestone62-sizing-tests.log` | M62 workspace test suite. | 191 passed, 0 failed across 12 binaries. |
| `target/headstart/milestone62-final-reuse-report.json` | M62 build and dependency reuse report. | Build passed; all three pinned dependencies fresh. |
| `target/headstart/milestone61-62-final-all-targets-check.log` | M61/M62 locked/offline all-targets validation. | Passed. |
| `apps/rustui-playground/src/stories/hover_gallery.rs` | Public `HoverRegion` enter/exit counters, text input, child button, removal/restoration, and explicit owner-state clearing. | M59 bounded native observations and limits recorded in `target/headstart/milestone59-hover-gui.json` and `docs/tasks/HOVER_REGION.md`. |
| `target/headstart/milestone52-focus-scope-gui.json` | Reviewed M52 binary observations and limits. | Binary SHA-256 `05fabb23fc5780056a01adeca40eab671a2704a740ba428638aab531d027ab9a`. |
| `target/headstart/milestone53-code-editor-gui.json` | Reviewed M53 native editing observations and limits. | Same M52/M53 binary SHA-256; syntax highlighting is unavailable pending approval. |
| `target/headstart/milestone54-lazy-grid-gui.json` | Initial failed and corrected M54 native painting/state/localization observations. | Corrected review bundle SHA-256 `316b268732146400e3f6efecb788317a509f3f11c8b48bff6dd764afd30bea7f`; first binary painted blank viewport and is not accepted. |
| `target/headstart/milestone55-frame-gui.json` | M55 native Frame screenshots and accessibility-tree observations. | Same corrected M54–M55 bundle; qualitative geometry only. |
| `target/headstart/milestone74-frame-gui.json` | M74 native Frame Center/Stretch/Baseline, retained text, Undo, cycle, and framed-button observations. | Bounded single-child acceptance; inner Frame width 160 with horizontal Stretch resolved Input holder collapse. No exact pixels, caret/selection, full AX/keyboard, RTL/platform, or peer-baseline claim. |
| `target/headstart/milestone74-inner-stretch-build.log` | M74 final Frame Review build. | Exit 0; final bundle SHA-256 `a993fc7eeb1f1749d190c92635bbb59a28fa6501ab1351153053ecdcd266eb58`. |
| `target/headstart/milestone74-workspace-tests.log` | M74 final workspace lib/tests run. | 238 passed, 0 failed across 12 binaries; Cargo exit 0. |
| `target/headstart/milestone74-all-targets-check.log` | M74 locked/offline all-targets validation. | Exit 0. |
| `target/headstart/milestone54-55-paint-tests.log` | Shared unit/integration test run. | 161 passed, 0 failed across 12 binaries. |
| `target/headstart/milestone54-55-paint-reuse-report.json` | Successful-build artifact reuse verification. | All three pinned GPUI packages confirmed all-fresh. |
| `target/headstart/milestone55-all-targets-check.log` | Final locked/offline workspace all-targets check. | Exit 0; warnings are recorded in the log; root confirmed completion. |
| `docs/tasks/FOCUS_SCOPE.md`, `docs/tasks/CODE_EDITOR.md`, `docs/tasks/LAZY_GRID_IMPLEMENTATION.md`, `docs/tasks/FRAME_MODIFIER.md` | Focus, editor, virtualized grid, and Frame contracts/evidence boundaries. | Milestone-specific native observations and unresolved acceptance limits. |

## Milestones 66–68 integration

| Path | Responsibility | Evidence / current boundary |
|---|---|---|
| `crates/rustui/src/interaction/drag_drop.rs`, `docs/tasks/DRAG_AND_DROP.md` | M66 typed same-process drag/drop wrappers, payload event, Copy/Transfer intent, ownership and keyboard-path contract. | Build and 221-test M66–M68 suite passed; bounded native pointer flow recorded in `target/headstart/milestone66-drag-drop-gui.json`; keyboard-only alternative and actual in-flight owner destruction remain pending. |
| `apps/rustui-playground/src/stories/drag_drop_gallery.rs` | M66 typed source/target, mismatch, transfer, removal, and keyboard-equivalent examples. | Story source is registered and built; bounded native pointer observations recorded; keyboard-only acceptance pending. |
| `crates/rustui/src/modifiers/shadow.rs`, `docs/tasks/SHADOW_MODIFIER.md` | M67 theme elevation and validated explicit native outer-shadow API. | Build and six focused Shadow tests included in the 221-test suite; bounded Dark-mode inspection recorded in `target/headstart/milestone67-shadow-gui.json`; Light appearance and remaining acceptance pending. |
| `apps/rustui-playground/src/stories/shadow_gallery.rs` | M67 theme/custom style, scope isolation, theme, locale, action, and omit/restore examples. | Story source is registered and built; Dark-mode native observations recorded; inspection remains open. |
| `crates/rustui/src/layout/card.rs`, `docs/tasks/CARD.md` | M68 Panel Card API and contract, composed from existing Container and Shadow. | Build and seven focused Card tests included in the 221-test suite; native visual/input acceptance pending. |
| `apps/rustui-playground/src/stories/card_gallery.rs` | M68 default/custom Card, theme/locale, same local child IDs in separate scopes, and omit/restore example. | Story source is registered and built; native observations pending. |
| `docs/FILE_TREE.txt` | Workspace source/document path inventory. | Includes the M66–M68 source, story, and task-contract paths. |

## Milestones 75–76 Tree and Table

| Path | Responsibility | Current evidence |
|---|---|---|
| `crates/rustui/src/components/data/tree.rs`, `apps/rustui-playground/src/stories/tree_reconciliation_gallery.rs` | Controlled stable-ID Tree and its reconciliation gallery. | M75 bounded native checks: unchanged expansion retained across replacement, explicit changes applied, removal/restoration reset nested expansion, stable selection followed reorder and cleared when removed, and a selected descendant caused GPUI ancestor expansion. Limits are in `docs/tasks/R3A_COLLECTION_SYNC.md`. |
| `crates/rustui/src/components/data/table.rs`, `apps/rustui-playground/src/stories/table_reconciliation_gallery.rs` | Controlled source-index Table and its reconciliation gallery. | M76 bounded native checks: invalid indices normalized to `None`, reorder/removal remained index-based, row click/repeat-click and clear observed. Notification counts and ABA provenance were not established. |
| `docs/tasks/R3A_COLLECTION_SYNC.md`, `docs/PUBLIC_API.md`, `docs/IMPLEMENTATION_ROADMAP.md` | M75/M76 task closeout, public behavior, and roadmap status. | Records bounded native observations, 248 passing workspace tests across 12 binaries, successful final build, and remaining acceptance limits. |
| `target/headstart/milestone75-tree-gui.json`, `target/headstart/milestone76-table-gui.json` | Native M75/M76 evidence and binary identity. | Shared binary SHA-256 `f7d78d68b7c688f4d5ca7ab24148df751b913cada32038df15d547dc05853b53`; detailed observations and limits are in the JSON records and task doc. |
| `target/headstart/milestone75-76-workspace-tests.log` | Combined M75/M76 workspace test run. | 248 passed, 0 failed across 12 binaries. |
| `target/headstart/milestone75-76-final-build.jsonl`, `target/headstart/milestone75-76-final-build.log` | Final Headstart build evidence. | Exit 0; GPUI and GPUI Components were fresh. |
| `target/headstart/milestone75-tree-selection-before.json`, `target/headstart/milestone76-table-invalid-before.json` | Preserved pre-fix M75/M76 native evidence. | Initial selection/reconciliation behavior retained for comparison; not acceptance evidence for the final build. |


## Milestones 67–71 and P3D–P3E integration

| Path | Responsibility | Evidence / current boundary |
|---|---|---|
| `target/headstart/milestone67-shadow-gui.json` | M67 bounded native Shadow acceptance. | Completed: Dark/Light; retained A/B independence; style/offset/parent updates; owner action; omission/restoration; Arabic RTL; English LTR. Limits: exact pixels; native shadow-disable toggle; full keyboard/AX speech; performance. Bundle SHA-256 `a00fcb2482c3ac63122228d7b704e53a264763df078bd7aca1386f5f9848f4f4`. |
| `target/headstart/milestone68-card-gui.json` | M68 bounded native Card acceptance. | Completed: independently editable same-ID children; custom style/parent updates; omission/restoration; Light; Arabic RTL. Equal 260-point width is gallery layout, not intrinsic Card sizing. Limits: exact pixels/padding; undo; full keyboard/AX speech; performance. Corrected bundle SHA-256 `c19a89cd2fe3597cf1cd487bcf352486afb6debc3c8c036c53a28a0c1661fe25`. |
| `crates/rustui-macros/src/view_builder.rs`, `docs/tasks/CONDITIONAL_STACKS.md` | M69 conditional stack children. | 236 workspace tests passed; bounded native acceptance completed: branch selection, keyed editor reorder, actions, parent updates, optional-child removal/restoration. RTL, full keyboard/AX speech, cursor/undo, release callbacks, and performance remain unverified. Evidence: `target/headstart/milestone69-conditional-stacks-gui.json`. |
| `crates/rustui/src/interaction/gestures.rs`, `apps/rustui-playground/src/stories/tap_gallery.rs`, `docs/tasks/TAP_REGION.md` | M70 TapRegion pointer wrapper. | Workspace checks passed; bounded native pointer acceptance completed: bubbling, independent child inputs, disabled wrappers, alternative button actions, removal/restoration. Keyboard activation, full AX speech, exact pixels, RTL, and performance remain unverified; no button role. Evidence: `target/headstart/milestone70-tap-region-gui.json`. |
| `crates/rustui/src/components/advanced/property_inspector.rs`, `apps/rustui-playground/src/stories/property_inspector_gallery.rs`, `docs/tasks/PROPERTY_INSPECTOR.md` | M71 typed PropertyInspector and Playground story. | Workspace tests/build/all-targets passed; bounded native acceptance complete: accessible Name input, sibling row-ID isolation, typed updates, reorder retention, disabled guards, and resumed editing. Full keyboard/AX speech, cursor/undo, removal, locale/theme, and performance remain unverified. |
| `target/headstart/milestone69-71-final-tests.log` | Earlier combined M69–M71 workspace test run. | 236 passed, 0 failed across 12 binaries; latest expanded rerun is registered separately. |
| `target/headstart/milestone69-71-final-build.log`, `target/headstart/milestone69-71-final-reuse-report.json` | M69–M71 build/dependency reuse report. | Build succeeded; all three pinned GPUI dependencies fresh. Reuse only; no performance claim. |
| `target/headstart/milestone69-71-all-targets-check.log` | M69–M71 locked/offline workspace all-targets check. | Exit 0; warnings recorded. M69–M71 bounded native acceptance completed; broader accessibility/keyboard/performance limits remain. |
| `crates/rustui/examples/cache_comparison.rs`, `scripts/compare_cache_workload.py`, `scripts/tests/test_compare_cache_workload.py`, `docs/tasks/P3D_CACHE_COMPARISON.md` | P3D paired retained-subtree example, collector/planner, tests, protocol. | 24 focused Python tests and Headstart example build passed after adding macOS 27 theme activation to the native probe. Finder/CUA sessions cover 8-leaf and 32-leaf workloads; each completed 12 processes with correctness true and AX/drag false. 8-leaf CPU ratio 0.2895; 32-leaf median paired CPU ratio 0.0729 (range 0.0718–0.0759), sampled max RSS ratio 0.9873. Process-only; no frame/GPU or visual acceptance. |
| `target/headstart/p3d-build.jsonl`, `target/headstart/p3d-build.log` | P3D example build evidence. | Build succeeded. |
| `target/headstart/p3d-reuse-report.json` | P3D pinned dependency reuse report. | `gpui-component`, `gpui-pre`, `gpui-pre-platform` all fresh. |
| `target/headstart/p3d-all-targets-check.log` | Corrected P3D locked/offline workspace all-targets check. | Exit 0 after `font_size` correction; warnings recorded. |
| `target/headstart/p3d-comparison-tests.log` | Earlier P3D comparison tool test run. | Superseded by the final 21-test run in `target/headstart/p3d-final-python-tests.log`. |
| `scripts/validate_patch_plan.py`, `scripts/tests/test_validate_patch_plan.py`, `docs/tasks/P3E_PATCH_PLAN_VALIDATOR.md` | P3E read-only patch-plan manifest validator. | 8 Python tests passed. Proposed schema confers no patch authorization. |
| `docs/tasks/GPUI_FAST_FEATURE_DELTA.md` | Source comparison of GPUI Fast and pinned RustUI stack; isolated quad-sort validation status. | GPUI Fast remains disabled. Exact quad-only patch approved for isolated validation; harness passed 9 tests and baseline/candidate builds succeeded. Paired probes are building; native visual/performance validation pending. Promotion not approved. |
| `docs/tasks/QUAD_SORT_PATCH_PROPOSAL.md`, `patches/proposed/gpui-pre-quad-stable-sort.patch`, `scripts/prepare_quad_sort_candidate.py` | Approved isolated quad-only stable-sort candidate definition and preparation tool. | Validation only; patch is not promoted or active in workspace dependency resolution. |
| `target/headstart/milestone71-property-inspector-gui.json` | M71 bounded native PropertyInspector observations. | Accessible Name input, independent sibling inspector values, typed updates, reorder, disabled guards, and resumed editing observed. Cursor/undo, removal, full keyboard/AX speech, locale/theme, and performance remain unverified. |
| `target/headstart/candidates/quad-stable-sort-20261018-a/evidence/harness-tests-rerun.log` | Corrected isolated quad-sort algorithm and Scene integration harness rerun. | 9 passed, 0 failed: 7 algorithm tests and 2 integration tests. |
| `target/headstart/candidates/quad-stable-sort-20261018-a/baseline/production-build.log`, `target/headstart/candidates/quad-stable-sort-20261018-a/candidate/production-build.log` | Isolated stock baseline and patched candidate production build logs. | Both builds succeeded. This is build evidence only; paired probes are building and native/performance review remains pending. |
| `target/headstart/candidates/quad-stable-sort-20261018-a/evidence/0001-preparation.json`, `0002-harness-preparation.json`, `0003-run-command-plan.json`, `0004-harness-import-repair.json`, `0005-upstream-unit-test-build-failure.json` | Immutable quad-sort candidate preparation and validation evidence. | Corrected harness rerun passed 9 tests (7 algorithm, 2 integration); upstream full-library test build exited 101 with 23 compile errors and zero quad tests executed. |
| `target/headstart/candidates/quad-stable-sort-20261018-a/candidate/quad-library-tests.log`, `target/headstart/candidates/quad-stable-sort-20261018-a/evidence/harness-tests.log`, `target/headstart/candidates/quad-stable-sort-20261018-a/evidence/harness-tests-rerun.log`, `target/headstart/candidates/quad-stable-sort-20261018-a/evidence/harness-lock.log` | Raw candidate upstream and standalone harness logs. | Upstream test build failed before quad tests; corrected harness rerun passed 9 tests (7 algorithm, 2 integration). |
| `target/headstart/milestone71-expanded-tests.log`, `target/headstart/milestone71-expanded-build.log`, `target/headstart/milestone71-expanded-build.jsonl`, `target/headstart/milestone71-expanded-reuse-report.json`, `target/headstart/milestone71-property-inspector-gui.json` | Latest M69–M71 expanded test/build/reuse evidence and M71 native record. | 236 tests passed across 12 binaries; build succeeded with all three pinned GPUI dependencies fresh; M71 bounded native acceptance completed with documented limits. |
| `target/headstart/p3d-final-python-tests.log` | Final P3D comparison-tool test suite. | 21 passed; final accessibility/native-drag guards verified. Completed Finder/CUA session: 12 processes, including 1 correctness warmup pair and 5 measured pairs; all correctness checks passed and AX/drag remained false. Median cached/uncached attached CPU ratio 0.2895; single opt-level-0 workload only. Summary and plan are recorded below; this is not frame/GPU/generalized speed evidence. |
| `target/headstart/performance/cache-comparison-p3d-review-01-summary.json`, `cache-comparison-p3d-32-20261019-c-summary.json` | Immutable 8-leaf and 32-leaf P3D session summaries. | Five measured pairs per workload. 8-leaf median attached CPU ratio 0.2895; 32-leaf 0.0729 (range 0.0718–0.0759), max-sampled RSS ratio 0.9873. No frame/GPU/generalized speedup claim. |
| `docs/UPSTREAM_AUDIT.md` | Approved source provenance; GPUI Fast base-revision correction. | Fast and approved snapshot declare same Zed base; package/source trees differ. Fast remains disabled. |

## M77a release counters and M78 ComboBox / NavigationSplit review

| Path | Responsibility | Current evidence |
|---|---|---|
| `target/headstart/candidates/dialog-focus-rebase-20261018-a/evidence/native-final-release-counters.png` | M77a post-run weak input-state release-counter capture. | 15/15 probe `InputState` entities retired/released; not a count of all allocations. |
| `target/headstart/milestone77-combobox-before-fix.json`, `target/headstart/milestone77-combobox-popup-mismatch.png` | Preserved ComboBox native regression reproduction. | After replacing options, bound Skia remained but popup selected old-index CoreGraphics; defect reproduced on the pre-fix binary. |
| `target/headstart/milestone78-combobox-gui.json`, `target/headstart/milestone78-combobox-popup-fixed.png` | M78 bounded native ComboBox regression verification and fixed popup capture. | Replacement popup selected Skia; Metal binding, disabled popup guard, external disabled-owner update, and re-enabled selection observed. Missing-option blank popup leaves owner CoreGraphics; no normalization contract added. |
| `target/headstart/verification/M78 Navigation ComboBox Review.app`, `target/headstart/verification/milestone78-navigation-combo-box-review.json` | M78 packaged Playground review binary and hash/test/build record. | Source/package executable SHA-256 `df0d1e92f50d116aef04478fe1363c77d0444d14f1e5a0aa25596a1629d71025`; GUI package is for review. |
| `target/headstart/milestone78-final-workspace-tests.log`, `target/headstart/milestone78-viewport-all-targets.log`, `target/headstart/milestone78-final-playground-build.log` | Final M78 workspace tests, viewport all-targets check, and Playground build. | Workspace: 252 passed, 0 failed across 12 binaries; all-targets and Playground build exited 0. |
| `target/headstart/milestone78-navigation-before-fix.json`, `target/headstart/milestone78-navigation-empty-collapsed.png` | Preserved initial M78 NavigationSplit native result with an unbounded gallery parent. | None/unknown collapsed the whole split; the bounded 560×300 gallery parent resolved this routing failure in the final review. |
| `target/headstart/milestone78-navigation-gui.json`, `target/headstart/milestone78-navigation-final.png` | Final M78 bounded native NavigationSplit routing checks and screenshot. | None/unknown kept the sidebar and hid detail; native selection recovered; Files binding updates, reorder, selected-item removal, and restoration observed. Resize drag, keyboard, locale, full accessibility, and direct allocation-release instrumentation remain pending. |
| `target/headstart/verification/M78 Final Navigation Review.app`, `target/headstart/verification/milestone78-final-navigation-review.json` | Final M78 Navigation review app and package/build/test hash record. | Source/package executable SHA-256 `732bc0da634acc6fbf49c1a050ec8ba96fe5af6e1b611d2a2a75b6e5d35c0123`; final bounded routing evidence is in `target/headstart/milestone78-navigation-gui.json`. |

## Milestone 79 source checks and isolated O7 scripted assertions

| Path | Responsibility | Current evidence |
|---|---|---|
| `target/headstart/milestone79-workspace-tests.log` | M79 workspace tests for the updated RustUI source. | 263 passed, 0 failed across 12 binaries. |
| `target/headstart/milestone79-final-all-targets.log` | M79 workspace all-targets check. | Exit 0; build diagnostics and warnings remain in the raw log. |
| `target/headstart/milestone79-playground-build.log`, `target/headstart/milestone79-playground-build.jsonl` | M79 canonical Playground build and Cargo compiler-artifact provenance. | Build exited 0; GPUI and GPUI Components artifacts were fresh/reused. This is compile evidence only, not GUI acceptance. |
| `docs/tasks/O7_DIALOG_KEYED_FOCUS_REBASE_REVIEW.md` | Isolated O7 revised tracked-Dialog patch findings, M77a native observations, M79 scripted assertions, and limits. | M79 records 19 isolated assertions and result/provenance hashes with normal dependency promotion false; both per-scenario runtime binary hashes are null. M77a counters show 15/15 weak probe `InputState` entities retired/released. Cross-window, timing/race, broader entity-release/callback counts, and full acceptance remain open. |
| `target/headstart/candidates/dialog-focus-rebase-20261018-a/evidence/milestone79-scripted-dialog.json` | M79 isolated scripted Dialog assertion and provenance summary. | 19 assertions across stale-ID and double-confirm results; summary `binary_sha256` is `689e4e08481ae9bc9260106050024b76f625ea6d7e54bfab95ab25a276e3a2ce`; result files are hash-identified, runtime binary hashes are null, normal dependency promotion is false. |

## Milestone 80 Collection native closeout

| Path | Responsibility | Current evidence |
|---|---|---|
| `target/headstart/milestone80-collection-final-tests.log` | Focused Collection regression suite after the unknown-height scroll fix. | 8 passed, 0 failed. |
| `target/headstart/milestone80-collection-final-check.log` | RustUI all-targets check after the Collection fix. | Exit 0. |
| `target/headstart/milestone80-collection-final-playground-build.log` | Canonical Playground build for native acceptance. | Exit 0; the disposable bundle and executable were removed after testing; Cargo cache retained. |
| `target/headstart/milestone80-collection-native-final.json` | Bounded native Collection scroll and state-retention observations. | Selected `record-0999` scrolled to the final six rows; edited text and row 0000/0001 counters survived scrolling out and back. Broader keyboard, accessibility, reorder, removal, and lifecycle acceptance remain pending. |
| `docs/tasks/R3A_COLLECTION_SYNC.md` | Collection selection synchronization contract and M80 native acceptance record. | M80 checks and explicit limits documented. |

## Milestone 81 — dependency-reuse verifier

| Path | Responsibility | Current evidence |
|---|---|---|
| `scripts/verify_dependency_reuse.py` | Validate Cargo JSON event ordering before confirming GPUI artifact freshness. | M81 rejects any nonblank event after `build-finished`; accepted historical log confirms freshness only. |
| `scripts/tests/test_verify_dependency_reuse.py` | Exercise valid and invalid Cargo event ordering. | M81: 2 tests passed. |
| `target/headstart/milestone81-dependency-reuse-tests.log`, `target/headstart/milestone81-dependency-reuse-verifier.log`, `target/headstart/milestone81-dependency-reuse-report.json` | Durable M81 verification evidence. | Unit tests pass; three pinned GPUI packages have fresh artifacts; no speedup inferred. |

| apps/rustui-playground/src/lib.rs | Reset the filtered catalog List viewport when the search query changes. | M82: native search from a scrolled catalog showed the matching Collection row. |
| apps/rustui-playground/src/stories/collection_gallery.rs | Preserve Collection restore position with stable neighboring IDs across reorder. | M82: 3 focused tests and native remove/reverse/restore acceptance passed; row state stayed attached to record-0000. |
| target/headstart/milestone82-collection-tests.log, target/headstart/milestone82-collection-check.log, target/headstart/milestone82-collection-playground-build.log, target/headstart/milestone82-collection-mutation-gui.json | Durable M82 test, build, and native interaction evidence. | Three focused tests passed; check/build succeeded; GUI record uses binary SHA-256 f44b6120476b8d0ed6df360dd6aa46a9bb42ce62690cb38255b3e8aa3eb6dbb7. |

## Milestone 83 — Collection keyboard decision tests

| Path | Responsibility | Current evidence |
|---|---|---|
| `crates/rustui/src/components/data/collection.rs` | Collection keyboard decision helpers and regression test. | M83: focused helper test passed; `rustui --all-targets` check passed. Native key-event, focus/IME, and screen-reader acceptance remain open. |
| `docs/tasks/M83_COLLECTION_KEYBOARD.md` | Scope, validation evidence, and remaining limits for M83. | Recorded. |
| `target/headstart/milestone83-collection-keyboard-tests.log`, `target/headstart/milestone83-collection-keyboard-check.log` | Focused test and all-targets check logs. | Test: 1 passed; check: passed with existing warnings. |

## Milestone 84 — Collection keyboard acceptance

| Path | Responsibility | Current evidence |
|---|---|---|
| `apps/rustui-playground/src/stories/collection_gallery.rs` | Public RustUI Collection interaction demo used for native keyboard verification. | M84: bounded navigation, Escape, Enter gating, and named accessibility-tree rows observed. Screen-reader and cross-platform acceptance remain open. |
| `docs/tasks/M83_COLLECTION_KEYBOARD.md` | Collection keyboard helper and native acceptance limits. | M83 and M84 recorded. |
| `target/headstart/milestone84-collection-keyboard-gui.json` | Native keyboard and accessibility-tree observations with test binary provenance and cleanup record. | Bounded M84 observations recorded. |

| `scripts/verify_dependency_boundary.py`, `scripts/tests/test_verify_dependency_boundary.py` | M85 Cargo-metadata guard for direct, transitive, optional Allui dependencies and local paths outside the RustUI workspace; includes fixtures for allowed vendored paths. | Four focused tests passed; current workspace check passed. |
| `docs/tasks/M85_DEPENDENCY_BOUNDARY.md` | M85 scope, invocation behavior, validation evidence, and limits. | Recorded. |
