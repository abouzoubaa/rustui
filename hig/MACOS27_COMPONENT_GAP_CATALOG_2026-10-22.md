# RustUI macOS 27 component and platform gap catalogue — 2026-10-22

**Design-only artifact.** Read alongside `macos27.json` and `macos27.design-v2-candidate.json`. A source module or HIG concept is not proof of complete runtime behavior. Status and design completeness are separate dimensions. “Partial” means source module is present, but this study has not verified complete states/accessibility/native behavior and the HIG JSON lacks a dedicated style record. “Absent” means no matching reusable component source or matching record was found during this audit. Native-only means the OS owns the canonical surface; do not redraw it as a pretend native control.

## Count reconciliation and classification

- Preserved baseline: **41 existing HIG IDs, 419 styles**. No existing ID is replaced or renamed.
- Added design-only gap inventory: **31 component contracts whose RustUI source module exists but whose theme record is absent**, **5 reusable patterns absent from both code and the HIG theme**, and **11 native platform integrations/panels absent as runtime bridges**. These are three separate sets; do not add them to the v1 theme’s supported-component count.
- The inventory also retains **9 composition/reference examples** as examples rather than component APIs.
- Existing HIG references such as `segmented_control`, `menu_bar`, `save_dialog`, `window_chrome`, `window_controls`, `text_style_guide`, and `alert_dialog` are not counted again as absent records. Their runtime status remains reference-only/partial as identified in the earlier 41-record audit.

## Existing RustUI source, not yet represented by a dedicated theme record

| Candidate contract | Source evidence | Runtime status | Design completeness | Visual reference / principal variants still needed |
|---|---|---|---|---|
| `avatar` | `crates/rustui/src/components/basic/avatar.rs` | partial (module exists; behavior not fully verified) | inventory-only | P29 image/icon concepts; #10 asset browser; identity/profile artwork, fallback initials, loading/error |
| `badge` | `crates/rustui/src/components/basic/badge.rs` | partial (module exists; behavior not fully verified) | inventory-only | P28 status/feedback; #05 analytics; status/count/notification/semantic severity |
| `canvas` | `crates/rustui/src/components/advanced/canvas.rs` | partial (module exists; behavior not fully verified) | inventory-only | P16 wrapping; P18 split; P34 drag/drop; zoom/pan/selection/overlay/empty |
| `code_editor` | `crates/rustui/src/components/advanced/code_editor.rs` | partial (module exists; behavior not fully verified) | inventory-only | paired #13 split/code workspace; P27 rich text; syntax theme, selection, caret, diagnostics, large text |
| `collection` | `crates/rustui/src/components/data/collection.rs` | partial (module exists; behavior not fully verified) | inventory-only | P14 asset grid; #10 asset browser; grid/list density, selection, drag/drop, empty/loading |
| `data_grid` | `crates/rustui/src/components/data/data_grid.rs` | partial (module exists; behavior not fully verified) | inventory-only | P13 tables/tree; #06 data explorer; sort/filter/resize/selection/focus/virtualized rows |
| `date_picker` | `crates/rustui/src/components/input/date_picker.rs` | partial (module exists; behavior not fully verified) | inventory-only | P07 date/time; #09 calendar; locale calendar, range, disabled date, keyboard |
| `empty_state` | `crates/rustui/src/components/feedback/empty_state.rs` | partial (module exists; behavior not fully verified) | inventory-only | P14 assets; P12 lists; empty/search-empty/permission/action |
| `error_state` | `crates/rustui/src/components/feedback/error_state.rs` | partial (module exists; behavior not fully verified) | inventory-only | P28 feedback; P33 document viewer; recoverable/fatal/retry/inline |
| `graph_view` | `crates/rustui/src/components/advanced/graph_view.rs` | partial (module exists; behavior not fully verified) | inventory-only | paired #12 workflow; P31 charts; node/edge selection, zoom, keyboard alternative |
| `keyed_list` | `crates/rustui/src/components/data/keyed_list.rs` | partial (module exists; behavior not fully verified) | inventory-only | P12 list/outline; P34 drag/drop; stable identity, reorder, insertion/removal, focus retention |
| `node_editor` | `crates/rustui/src/components/advanced/node_editor.rs` | partial (module exists; behavior not fully verified) | inventory-only | paired #12 workflow; P19 rule editor; connection/selection/drag/validation |
| `progress_indicator` | `crates/rustui/src/components/feedback/progress.rs` | partial (module exists; behavior not fully verified) | inventory-only | P28 corrected progress concept; determinate/indeterminate/paused/error/read-only |
| `property_inspector` | `crates/rustui/src/components/advanced/property_inspector.rs` | partial (module exists; behavior not fully verified) | inventory-only | P10 sidebar; P18 inspector; sections, editable/read-only, mixed value, validation |
| `resizable_panel` | `crates/rustui/src/components/advanced/resizable_panel.rs` | partial (module exists; behavior not fully verified) | inventory-only | P16 adaptive layout; P18 split views; min/max/collapse/keyboard resize/focus retention |
| `secure_field` | `crates/rustui/src/components/input/secure_field.rs` | partial (module exists; behavior not fully verified) | inventory-only | P04 text validation; password visibility, IME, paste policy, accessible reveal |
| `skeleton` | `crates/rustui/src/components/feedback/skeleton.rs` | partial (module exists; behavior not fully verified) | inventory-only | P28 loading; loading shape, reduce motion, avoid false semantic values |
| `spinner` | `crates/rustui/src/components/feedback/spinner.rs` | partial (module exists; behavior not fully verified) | inventory-only | P28 feedback; busy status, announcement throttle, reduced motion |
| `text_editor` | `crates/rustui/src/components/input/text_editor.rs` | partial (module exists; behavior not fully verified) | inventory-only | paired #11 Notes; P27 rich text; selection/IME/bidi/formatting/scale |
| `tree` | `crates/rustui/src/components/data/tree.rs` | partial (module exists; behavior not fully verified) | inventory-only | P12 outline; P13 tree tables; hierarchy/expand/selection/typeahead/virtualization |
| `breadcrumb` | `crates/rustui/src/components/navigation/breadcrumb.rs` | partial (module exists; behavior not fully verified) | inventory-only | P11 path controls; overflow, current item, RTL, keyboard |
| `navigation_split` | `crates/rustui/src/components/navigation/navigation_split.rs` | partial (module exists; behavior not fully verified) | inventory-only | paired #13 split workspace; P10 sidebar; sidebar/detail/inspector modes and resize |
| `navigation_stack` | `crates/rustui/src/components/navigation/navigation_stack.rs` | partial (module exists; behavior not fully verified) | inventory-only | P11 tabs/path controls; push/pop/history/keyboard/focus |
| `context_menu` | `crates/rustui/src/components/overlay/context_menu.rs` | partial (module exists; behavior not fully verified) | inventory-only | P08 Menus, Pop-ups and Pull-downs; P34 drag/drop; context action semantics, shortcuts, dismissal |
| `dialog` | `crates/rustui/src/components/overlay/dialog.rs` | partial (module exists; behavior not fully verified) | inventory-only | P21 alerts/dialogs/sheets; modal/nonmodal, initial/return focus, cancel/destructive |
| `drawer` | `crates/rustui/src/components/overlay/drawer.rs` | partial (module exists; behavior not fully verified) | inventory-only | P18 split/panel concepts; edge/placement, modality, focus, reduce motion |
| `popover` | `crates/rustui/src/components/overlay/popover.rs` | partial (module exists; behavior not fully verified) | inventory-only | P20 popovers/tooltips; anchor/placement/dismissal/focus behavior |
| `sheet` | `crates/rustui/src/components/overlay/sheet.rs` | partial (module exists; behavior not fully verified) | inventory-only | P21 sheets; P23 save/open; task-scoped vs OS-owned sheet distinction |
| `toast` | `crates/rustui/src/components/overlay/toast.rs` | partial (module exists; behavior not fully verified) | inventory-only | P28 feedback; P35 notification; severity, timeout, action, announcement |
| `tooltip` | `crates/rustui/src/components/overlay/tooltip.rs` | partial (module exists; behavior not fully verified) | inventory-only | P20 help; hover/focus timing, keyboard, accessible description |
| `alert_view` | `crates/rustui/src/components/overlay/alert.rs` | partial (module exists; behavior not fully verified) | inventory-only | P21 alerts; inline RustUI alert distinct from native alert/panel |

## Reusable patterns absent from both the HIG JSON and RustUI component source

| Candidate | Type | Runtime status | Design completeness | Evidence and first contract scope |
|---|---|---|---|---|
| `command_palette` | reusable component | absent | candidate | #14 paired command/search center, P36 keyboard/focus; query/scopes/recent commands/shortcuts/empty state |
| `media_player` | reusable domain component | absent | candidate | P30 media; play/pause/scrub/volume/captions/keyboard |
| `chart_view` | reusable domain component | absent | candidate | P31 charts and gauges; #05 dashboard; series/legend/scale/selection/non-color encoding |
| `map_view` | reusable domain component | absent | candidate | P32 maps and locations; markers/selection/zoom/keyboard/list alternative |
| `document_preview` | reusable document component / OS bridge | absent | candidate | P33 Quick Look; preview/loading/unavailable/protected/OS handoff |

## Native platform surfaces and integrations

These are design/bridge tasks, not ordinary color-token components. The visual artwork can guide information hierarchy only; actual operation, permission, keyboard behavior, focus and system appearance belong to the native API/host layer. Existing JSON compositions (e.g. `save_dialog`, `window_chrome`) do not confer native behavior.

| Candidate bridge | Runtime status | Design completeness | Visual evidence | Review boundary |
|---|---|---|---|---|
| `font_panel_bridge` | native-only | integration candidate | P26 typography | native panel lifecycle, selection, cancellation; require macOS/AppKit ownership and cancellation/error behavior. |
| `print_panel_bridge` | native-only | integration candidate | P24 print/export | native print workflow, preview, progress/error; require macOS/AppKit ownership and cancellation/error behavior. |
| `open_panel_bridge` | native-only | integration candidate | P23 open/save | sandboxed file selection, filter, cancellation; require macOS/AppKit ownership and cancellation/error behavior. |
| `save_panel_bridge` | native-only | integration candidate | P23 open/save | filename validation, overwrite, extension, cancellation; require macOS/AppKit ownership and cancellation/error behavior. |
| `share_panel_bridge` | native-only | integration candidate | P35 sharing | system services handoff, permissions, failure; require macOS/AppKit ownership and cancellation/error behavior. |
| `menu_bar_command_bridge` | native-only | host integration candidate | P35 menu bar; Apple menu-bar guidance | command hierarchy, standard shortcuts, validation; require macOS/AppKit ownership and cancellation/error behavior. |
| `window_chrome_bridge` | native-only | host integration candidate | P22 window anatomy | active/inactive, full-screen, titlebar/toolbar integration; require macOS/AppKit ownership and cancellation/error behavior. |
| `window_controls_bridge` | native-only | host integration candidate | P22 window anatomy | OS-owned close/minimize/zoom; no imitation without full ownership; require macOS/AppKit ownership and cancellation/error behavior. |
| `notification_center_bridge` | native-only | integration candidate | P35 system previews/sharing; P40 Desktop Widgets and System Controls; P28 app feedback comparison | delivery, user permission, actions, grouping; require macOS/AppKit ownership and cancellation/error behavior. |
| `menu_extra` | native-only | integration candidate | P35 system previews/sharing | status item, menu lifecycle, overflow, accessibility; require macOS/AppKit ownership and cancellation/error behavior. |
| `system_widget_extension` | native-only | extension candidate | P40 Desktop Widgets and System Controls | timeline refresh, privacy, extension lifecycle; require macOS/AppKit ownership and cancellation/error behavior. |

## Compositions and reference examples (not component counts)

1. Studio and analytics dashboards — paired #01/#05.
2. Settings/forms and guided setup — paired #07/#15; components compose rows, fields, help and validation.
3. Kanban and workflow graph — paired #08/#12; separate board composition from GraphView/Canvas and drag/drop contracts.
4. Calendar planning — paired #09; date picker is a control, week calendar is a composition.
5. Asset browser/grid — paired #10 and P14; collection/data grid, image, selection, inspector and drag/drop.
6. Notes/code/split workspace — paired #11/#13 and P18/P27; text editor, code editor, panel and navigation compositions.
7. Command/search center — paired #14 and P35/P36; SearchField/Menu are existing families; CommandPalette remains an absent reusable pattern.
8. Multilingual settings — paired #16 and P38; localization and RTL are cross-cutting, not a single widget.
9. Component and flex/layout galleries — #02–04 and P15–19; these are test scenarios for variants, not new components.

## Visual reference coverage: all 86 unique stored originals

`MACOS27_VISUAL_REFERENCE_MAP_2026-10-22.csv` contains one row per unique stored path with checksum, source ID, source message, category and proposed family/status mapping. It reconciles the 77 Library records and 19 extracted archive members after nine byte-identical archive duplicates are mapped to their v1.0 canonical file. Summary groups:

- 19 paired gallery candidates including retained #02/#16 alternatives; screen families #01–16.
- 41 dark Playground concepts P01–P39 with corrected/final alternatives including P28.
- 16 v1.0 and 10 distinct v1.1 specification artworks; nine unchanged v1.1 members link to v1.0 originals. These are architecture/workflow proposal diagrams, not runtime evidence or Apple HIG requirements.
- 86 unique originals inspected in manageable contact-sheet batches. Broad classification is complete; detailed pixel-level annotations for every image are not claimed. The source inventory’s final/alternate statuses and SHA-256s remain authoritative.

## Owner-supplied component reference reconciliation — 2026-10-22

`OWNER_COMPONENT_REFERENCE_2026-10-22.md` is now reconciled row-by-row in [`MACOS27_OWNER_COMPONENT_COVERAGE_2026-10-22.csv`](MACOS27_OWNER_COMPONENT_COVERAGE_2026-10-22.csv): **148 exact named-item bullets across all 16 numbered sections**. The count is one row per bullet under “Named-item inventory”; repeated menu bar extra, share, Quick Look, settings, file-panel, and scene ideas remain separate source rows and use canonical IDs/aliases for cross-reference. Its eight appearance requirements, 11 version-claim families, and nine supplied source links have a separate **28-row** ledger in [`MACOS27_OWNER_CLAIM_COVERAGE_2026-10-22.csv`](MACOS27_OWNER_CLAIM_COVERAGE_2026-10-22.csv).

The row ownership classification is 17 system-owned surfaces, 5 native-system integrations, 51 native-host/composition items, 64 RustUI/style/composition concepts, 10 native material/API items, and 1 retired/unsupported item. Those are source-row classifications, not additional component counts. The previous candidate inventory remains 31 source-present contracts, 5 absent reusable patterns, 11 native integration candidates, and 9 composition examples. `macos27.design-v2-candidate.json` now links the matrix and records eight cross-cutting design-contract groups separately from those inventories. It remains design-only and runtime-incompatible.

The companion [`MACOS27_OWNER_RECONCILIATION_2026-10-22.md`](MACOS27_OWNER_RECONCILIATION_2026-10-22.md) records the per-family design contracts and confirmed/corrected/unverified Apple claims. The current production-equivalent HIG JSON remains unchanged; a related JSON ID or source module is not proof of complete runtime behavior. Run `python3 hig/validate_owner_component_coverage.py` for HIG-local structural and baseline checks.

### Additional behavior gaps retained under existing contracts

The design candidate now also lists eight owner-derived behavior-gap groupings. They do not add component primitives or change the 31/5/11/9 inventory totals.

Subsequent visual work must meet [`MACOS27_VISUAL_FIDELITY_ACCEPTANCE_2026-10-22.md`](MACOS27_VISUAL_FIDELITY_ACCEPTANCE_2026-10-22.md). Image concepts are supplementary references. Native macOS 27 observations and official Apple APIs/guidance are authoritative; unsupported measurements remain unknown, and a static catalogue does not demonstrate fidelity.

| Gap grouping | Existing owner | Disposition |
|---|---|---|
| Validation messages | `text_field`, `form`, `error_state` | Specify help/error association and announcements; no new validator widget is promised. |
| Empty/loading/error states | Existing feedback, collection, and media candidates | Shared lifecycle states and recovery actions, not one primitive per state. |
| Virtualized collections | `collection`, `data_grid`, `tree`, `keyed_list` | Specify identity, focus, selection, and accessibility at scale; performance remains unverified. |
| Rich text and rulers | `text_editor`, `code_editor` | Specify editing/selection/IME/bidi/diagnostics and ruler contracts; candidate modules are not capability proof. |
| Date/time and range variants | `date_picker`, `slider`, `stepper` | Keep variants with their owning controls; exact runtime support remains unverified. |
| Tree tables | `tree`, `data_grid`, v1 `outline_view`/`table` | Composition contract; no dedicated tree-table primitive asserted. |
| Pagination pattern | Existing collection families | Defer to a collection behavior if needed; no source module or owner component is claimed. |
| Breadcrumbs and splitter resizing | `breadcrumb`, `navigation_split`, `resizable_panel` | Specify overflow, min/max, collapse, keyboard resize, and focus behavior; no standalone splitter primitive. |

## State and variant coverage to design before claiming completeness

| Family | Variants/states to specify | Cross-cutting checks |
|---|---|---|
| Buttons/actions | primary/secondary/quiet/destructive; rest/hover/pressed/focused/disabled/busy; menu-button/split action | keyboard activation, cancellation, accessible name, focus ring, contrast |
| Text/fields/search/editors | placeholder/filled/selected/composing/invalid/warning/disabled/secure/read-only; clear/suggestion/loading | IME, caret, bidi, truncation vs full value, validation announcement, large text |
| Choice/range/date/color | checked/unchecked/mixed; selected/unselected; min/max/step; date range/disabled; color alpha/gamut | arrows/typeahead, locale calendars/numbers, non-color state cues, value semantics |
| Navigation/data | selected/focused/expanded/collapsed/hovered/drag target/empty/loading/error; dense/regular | arrow navigation, stable identity, row/column focus, sorting labels, RTL direction |
| Overlay/system | modal/nonmodal; initial focus/return; default/cancel/destructive; active/inactive | Escape policy, keyboard traversal, screen reader announcement, focus containment |
| Feedback/media/visualization | determinate/indeterminate/paused/failed; play/pause/scrub; legend/selection | reduced motion, announcement frequency, captions, chart non-color encoding and data table alternative |
| Layout/window | compact/regular/expanded; constrained/resizable; sidebar collapsed; split sizes | min/max, wrap, overflow/scroll, large text, long strings, multi-window and display scale |

## Official Apple pattern index checked 2026-10-22

Apple HIG pages are living guidance. Search-index snippets/titles were verified, but full page Markdown is JavaScript-rendered; validate implementation-specific details against current Apple docs during each design gate. None of these pages establishes RustUI runtime support or literal token values.

- [Designing for macOS](https://developer.apple.com/design/human-interface-guidelines/designing-for-macos): menu-bar commands and high-precision input are macOS patterns.
- [Menus](https://developer.apple.com/design/human-interface-guidelines/menus), [The menu bar](https://developer.apple.com/design/human-interface-guidelines/the-menu-bar), [Keyboards](https://developer.apple.com/design/human-interface-guidelines/keyboards): command access/keyboard interaction; key mirroring guidance explicitly connects RTL layout and shortcuts.
- [Search fields](https://developer.apple.com/design/human-interface-guidelines/search-fields): editable input with search affordance, clear button and placeholder.
- [Segmented controls](https://developer.apple.com/design/human-interface-guidelines/segmented-controls): macOS can support single or multiple choice; do not conflate with tabs. RustUI v1 has a reference record, not a renderer.
- [Tab views](https://developer.apple.com/design/human-interface-guidelines/tab-views): mutually exclusive panes; distinct from a tab bar pattern in other platforms.
- [Sheets](https://developer.apple.com/design/human-interface-guidelines/sheets), [Panels](https://developer.apple.com/design/human-interface-guidelines/panels), [Printing](https://developer.apple.com/design/human-interface-guidelines/printing): task sheets and native supplementary panels/print workflow.
- [Progress indicators](https://developer.apple.com/design/human-interface-guidelines/progress-indicators): indicate ongoing work and avoid an apparent stall; they do not specify RustUI timing or animation.
- [Accessibility](https://developer.apple.com/design/human-interface-guidelines/accessibility), [VoiceOver](https://developer.apple.com/design/human-interface-guidelines/voiceover): keyboard-only navigation and screen-reader use are required review dimensions, not palette-only settings.
- [Layout](https://developer.apple.com/design/human-interface-guidelines/layout), [Sidebars](https://developer.apple.com/design/human-interface-guidelines/sidebars), [Toolbars](https://developer.apple.com/design/human-interface-guidelines/toolbars), [Windows](https://developer.apple.com/design/human-interface-guidelines/windows): adapt layout and host ownership; sidebar is leading-side navigation, windows can contain toolbars/tab bars.
- [Context menus](https://developer.apple.com/design/human-interface-guidelines/context-menus), [Drag and drop](https://developer.apple.com/design/human-interface-guidelines/drag-and-drop), [Combo boxes](https://developer.apple.com/design/human-interface-guidelines/combo-boxes), [Toggles](https://developer.apple.com/design/human-interface-guidelines/toggles): distinct interaction semantics inform contracts.
- [Color](https://developer.apple.com/design/human-interface-guidelines/color), [Materials](https://developer.apple.com/design/human-interface-guidelines/materials), [Typography](https://developer.apple.com/design/human-interface-guidelines/typography), [SF Symbols](https://developer.apple.com/design/human-interface-guidelines/sf-symbols): adaptive semantic values, effect layers, typography roles and symbol modes.

## Review decisions before any production application

1. Agree which 31 source-present but unthemed components deserve dedicated semantic theme contracts versus composed style inheritance.
2. Choose the first absent component proposal (suggest CommandPalette only if a distinct API and behavior contract is approved).
3. Decide native bridge scope/ownership for panels, menu bar, window chrome and notifications.
4. Approve typed v2 compatibility/unknown-key/migration policy separately; the v2 candidate is not loadable by the current v1 loader.
5. For each family approve state semantics, keyboard behavior, VoiceOver roles, RTL behavior and reduced motion/transparency before styling.
6. When the user declares the sandbox design complete, create a production application patch separately and review it; never copy these candidate records into production automatically.

> Corrected visual indices: P05 Search, Tokens and Combo Boxes; P08 Menus, Pop-ups and Pull-downs; P40 Desktop Widgets and System Controls. The CSV maps each inspected source image by path and checksum. Treat these as design concepts, not platform requirements or runtime evidence.
