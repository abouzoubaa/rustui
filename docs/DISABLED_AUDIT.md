# Disabled-control audit

R15 covers 40 inventory cases in one GPUI test-window matrix: Tab/Shift-Tab, pointer input, Enter/Space, disabled accessibility state, and focus migration plus re-enabled operation. It also tests replacing a bound value while disabled without writing the old binding.

Before is NOT VERIFIED: the initial harness required focus-sentinel and display-scale corrections, so its totals are not a reliable baseline. The disposable-worktree matrix passed with 194 tests passed and exactly seven R15b tests ignored; the final workspace run also passed all non-ignored matrix tests. FAIL→R15b cells remain as seven explicitly ignored tests; they are unresolved, not passes. N/A cells run the shared factory and return with the reason below.

Inventory: `rg -n "pub.*disabled\(" crates/rustui/src/components` plus the SecureField/SearchField facades and the enabled-state drag/drop wrappers in `crates/rustui/src/interaction/drag_drop.rs`. PropertyInspector tests include its global disabled state and each of its five editor variants. List rows and editable-grid cells are listed explicitly because they have no disabled API.

| Control | Before | Tab / Shift-Tab | Mouse | Enter / Space | Accessibility | Disable focused → next; re-enable |
|---|---|---|---|---|---|---|
| Button | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| Link | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| Checkbox | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| Radio | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| Toggle | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| Slider | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| Stepper | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| Picker | NOT VERIFIED | PASS | PASS | PASS | PASS | FAIL→R15b — Picker re-enabled control did not work |
| ComboBox | NOT VERIFIED | PASS | PASS | PASS | PASS | FAIL→R15b — ComboBox re-enabled control did not work |
| TextField | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| SecureField | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| SearchField | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| TextEditor | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| DatePicker | NOT VERIFIED | PASS | PASS | PASS | PASS | FAIL→R15b — DatePicker re-enabled control did not work |
| ColorPicker | NOT VERIFIED | PASS | PASS | PASS | PASS | FAIL→R15b — ColorPicker re-enabled control did not work |
| TokenField | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| CodeEditor | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| GraphView | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| NodeEditor | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| PropertyText | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| PropertyChoice | NOT VERIFIED | PASS | PASS | PASS | PASS | FAIL→R15b — PropertyChoice re-enabled control did not work |
| PropertyToggle | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| PropertyInteger | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| PropertyIntegerValue | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| PropertyInspector | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| ToolbarItem | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| Toolbar | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| BreadcrumbItem | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| Disclosure | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| NavigationLink | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| SidebarItem | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| NavigationSplitItem | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| TabItem | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| TreeItem | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| MenuItem | NOT VERIFIED | N/A — Popup items use menu arrow navigation rather than independent Tab stops. | PASS | PASS | FAIL→R15b — MenuItem accessibility node has no disabled flag | FAIL→R15b — popup item live-disable recovery not yet exercised |
| ListRow | NOT VERIFIED | N/A — List has no per-row disabled API; selectable(false) disables selection only. | N/A — List has no per-row disabled API; selectable(false) disables selection only. | N/A — List has no per-row disabled API; selectable(false) disables selection only. | N/A — List has no per-row disabled API; selectable(false) disables selection only. | N/A — List has no per-row disabled API; selectable(false) disables selection only. |
| GridCell | NOT VERIFIED | N/A — read_only cells deliberately remain navigable; no disabled-cell API. | PASS | PASS | N/A — read_only cells deliberately remain navigable; no disabled-cell API. | N/A — read_only cells deliberately remain navigable; no disabled-cell API. |
| DragSource | NOT VERIFIED | N/A — Wrapper has no keyboard activation or independent focus; child is noninteractive. | PASS | N/A — Wrapper has no keyboard activation or independent focus; child is noninteractive. | N/A — Typed wrapper exposes no independent accessibility node. | N/A — Wrapper has no keyboard activation or independent focus; child is noninteractive. |
| DropTarget | NOT VERIFIED | N/A — Wrapper has no keyboard activation or independent focus; child is noninteractive. | PASS | N/A — Wrapper has no keyboard activation or independent focus; child is noninteractive. | N/A — Typed wrapper exposes no independent accessibility node. | N/A — Wrapper has no keyboard activation or independent focus; child is noninteractive. |
| FileDropTarget | NOT VERIFIED | N/A — Wrapper has no keyboard activation or independent focus; child is noninteractive. | PASS | N/A — Wrapper has no keyboard activation or independent focus; child is noninteractive. | PASS | N/A — Wrapper has no keyboard activation or independent focus; child is noninteractive. |

Final matrix totals: **173 PASS, 7 FAIL→R15b, 20 N/A** across 200 cells. Binding replacement is a separate test.

The NavigationSplit divider remains an enabled independent control: traversal skips the disabled sidebar item and may stop on the divider. Drag/drop pointer cases assert ordinary clicks do not write owner state; native drag delivery is covered by the existing drag/drop tests. Native/manual acceptance is NOT VERIFIED.

All five R15 GPUI patches are promoted after disposable-worktree proof and exact replay verification; none remains proposed.

## Deferred tests

- `components::disabled_tests::picker::recovery`: R15b: Picker re-enabled control did not work.
- `components::disabled_tests::combo_box::recovery`: R15b: ComboBox re-enabled control did not work.
- `components::disabled_tests::date_picker::recovery`: R15b: DatePicker re-enabled control did not work.
- `components::disabled_tests::color_picker::recovery`: R15b: ColorPicker re-enabled control did not work.
- `components::disabled_tests::property_choice::recovery`: R15b: PropertyChoice re-enabled control did not work.
- `components::disabled_tests::menu_item::accessibility`: R15b: MenuItem accessibility node has no disabled flag.
- `components::disabled_tests::menu_item::recovery`: R15b: popup item live-disable recovery not yet exercised.
