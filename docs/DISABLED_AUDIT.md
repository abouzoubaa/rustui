# Disabled-control audit

R15/R15b covers 40 inventory cases in one GPUI test-window matrix: Tab/Shift-Tab, pointer input, Enter/Space, disabled accessibility state, and focus migration plus re-enabled operation. It also tests replacing a bound value while disabled without writing the old binding.

The original R15 baseline remains NOT VERIFIED: its initial harness required focus-sentinel and display-scale corrections, so its totals are not a reliable baseline. The final matrix passes all 201 tests with no ignores, including a separate binding-replacement test. N/A cells run the shared factory and return with the reason below.

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
| Picker | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| ComboBox | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| TextField | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| SecureField | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| SearchField | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| TextEditor | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| DatePicker | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| ColorPicker | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| TokenField | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| CodeEditor | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| GraphView | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| NodeEditor | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| PropertyText | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
| PropertyChoice | NOT VERIFIED | PASS | PASS | PASS | PASS | PASS |
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
| MenuItem | NOT VERIFIED | N/A — Popup items use menu arrow navigation rather than independent Tab stops. | PASS | PASS | PASS | PASS |
| ListRow | NOT VERIFIED | N/A — List has no per-row disabled API; selectable(false) disables selection only. | N/A — List has no per-row disabled API; selectable(false) disables selection only. | N/A — List has no per-row disabled API; selectable(false) disables selection only. | N/A — List has no per-row disabled API; selectable(false) disables selection only. | N/A — List has no per-row disabled API; selectable(false) disables selection only. |
| GridCell | NOT VERIFIED | N/A — read_only cells deliberately remain navigable; no disabled-cell API. | PASS | PASS | N/A — read_only cells deliberately remain navigable; no disabled-cell API. | N/A — read_only cells deliberately remain navigable; no disabled-cell API. |
| DragSource | NOT VERIFIED | N/A — Wrapper has no keyboard activation or independent focus; child is noninteractive. | PASS | N/A — Wrapper has no keyboard activation or independent focus; child is noninteractive. | N/A — Typed wrapper exposes no independent accessibility node. | N/A — Wrapper has no keyboard activation or independent focus; child is noninteractive. |
| DropTarget | NOT VERIFIED | N/A — Wrapper has no keyboard activation or independent focus; child is noninteractive. | PASS | N/A — Wrapper has no keyboard activation or independent focus; child is noninteractive. | N/A — Typed wrapper exposes no independent accessibility node. | N/A — Wrapper has no keyboard activation or independent focus; child is noninteractive. |
| FileDropTarget | NOT VERIFIED | N/A — Wrapper has no keyboard activation or independent focus; child is noninteractive. | PASS | N/A — Wrapper has no keyboard activation or independent focus; child is noninteractive. | PASS | N/A — Wrapper has no keyboard activation or independent focus; child is noninteractive. |

Final matrix totals: **180 PASS, 0 FAIL, 20 N/A** across 200 cells. Binding replacement is a separate test.

The NavigationSplit divider remains an enabled independent control: traversal skips the disabled sidebar item and may stop on the divider. Drag/drop pointer cases assert ordinary clicks do not write owner state; native drag delivery is covered by the existing drag/drop tests. Native/manual acceptance is NOT VERIFIED.

The R15 patch set remains approved. R15b adds popup-menu disabled semantics, live refresh and selection-preserving rebuilds, with exact replay, owned mirrors and disposable-worktree proof.

The recovery cases open each popup with a single activation, commit a choice/date/color through window input, and assert the original focus handle survives disable/enable. The menu recovery case asserts next-enabled selection, empty selection and non-activating Enter with all items disabled, stable menu focus, and activation after re-enable. No R15b ignores remain.

## R15b diagnosis

The five original recovery assertions failed with these exact messages:

- `Picker: re-enabled control did not work`
- `ComboBox: re-enabled control did not work`
- `DatePicker: re-enabled control did not work`
- `ColorPicker: re-enabled control did not work`
- `PropertyChoice: re-enabled control did not work`

The common failure was the test's final-state comparison after repeated activation: keyboard input opened the popup, then the trigger click closed it again without choosing a value. ColorPicker also had overlapping root/trigger nodes that the pointer helper clicked twice. The original and restored focus handles matched for every case. Select already copies its disabled option into its state on every render; DatePickerState and ColorPickerState have no disabled flag to clear. Disable/enable does not recreate their retained entities. The recovery test must assert opening and a committed popup choice separately.
