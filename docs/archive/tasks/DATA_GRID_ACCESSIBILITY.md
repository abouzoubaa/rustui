# DataGrid accessibility slice

When sibling grids can share an accessible label, set a unique stable identity with `DataGrid(...).id(...)`; the label remains the fallback identity for a single grid.

`DataGrid(table, label)` wraps an existing `TableView` with AccessKit's native
`Table` role and the supplied accessible name. The pinned GPUI Components
`DataTable` already gives each rendered data row the native `Row` role, so the
wrapper provides a table-to-row semantic path without duplicating the table
implementation or changing its selection behavior.

This slice does not claim complete grid accessibility. GPUI Components 0.7.3
assigns native `Row` to data rows but does not assign `ColumnHeader` or `Cell`
roles. RustUI's `TableDelegate` hooks can provide those roles without replacing
the pinned renderer: the header container is a `Row`, each rendered header is a
named `ColumnHeader`, and each rendered data cell is a `Cell` whose accessible
name comes from its visible text. The renderer still owns layout, horizontal
column virtualization, scrolling, and selection. This does not establish native
screen-reader speech or cross-platform acceptance, and it does not change
keyboard navigation.

The standalone Playground story in `apps/rustui-playground/src/stories/data_grid_accessibility.rs`
demonstrates the named table and controlled selection. It is intentionally not
registered here; the shared story registry is maintained by the parallel D1b
slice.


## M126 native observation

The canonical Table story exposed a native Table named `Quarterly revenue` with native row nodes. Clicking its first rendered DataGrid row changed `Selected source row` from `None` to `Some(0)`. This confirms the wrapper's bounded table-to-row semantics and controlled-selection path on macOS; it does not establish full screen-reader, RTL, cross-platform, or performance acceptance. At M126, RustUI had not yet supplied header or cell roles. Evidence: `target/headstart/evidence/r3-list-datagrid-acceptance-20261019.json`.

## M225 source-level column and cell semantics

RustUI's `StringTableDelegate` now marks its header container as `Row`, each
header label as a named `ColumnHeader`, and each rendered data cell as `Cell`.
This uses supported GPUI element semantics in the existing delegate hooks; the
pinned `DataTable` continues to render and virtualize the table. Source-level
semantics are implemented, but native screen-reader speech, accessibility-tree
inspection after this change, and cross-platform behavior remain unverified.
`rustfmt --edition 2024 --check crates/rustui/src/components/data/table.rs` and
`CARGO_TARGET_DIR=target/headstart/table-accessibility-check ./scripts/cargo-headstart check -p rustui --all-targets --locked`
passed. The isolated check log is `target/headstart/table-accessibility-check.log`.

## Native AX probe (2026-10-20)

Inspected the canonical Playground's Table story on macOS 27 using its current accessibility tree. The story rendered the named `Quarterly revenue` specimen with visible `Quarter`, `Revenue`, and `Status` headings and data values. The returned tree included rows, but did not expose a table node for that specimen, named column-header nodes, cell nodes, or the visible cell text. The same window did expose row nodes for the plain Table example. This is a bounded native AX observation, not VoiceOver evidence; it does not establish whether the omission is in GPUI's AX bridge, the native host, or the CUA tree representation.

Evidence: `target/headstart/evidence/data-grid-accessibility-ax-probe-20261020.json`. The next acceptance probe should inspect the same specimen with macOS Accessibility Inspector and VoiceOver, checking table name/role, each column header, cell association, and spoken values, then compare with the plain Table example. Do not close native screen-reader acceptance from the source role builders alone.

## M260 native AX retest (2026-10-20)

Reran the Table story in the disposable Playground bundle on macOS 27.0 with source HEAD `cb165c36b6d168895fbcfbd45140889627d54b24` and binary SHA-256 `217a85d8222e0932a81f2d7b41068f14b8c9f11fbf499154ac47e91c8c9ce1a4`. The CUA native accessibility tree exposed the named `Quarterly revenue` table, its row nodes, a header row, named `Quarter`, `Revenue`, and `Status` header cells, and body cell nodes. The same-window plain Table exposed row nodes, its `Source index` and `Row` header cells, and body cell nodes. The screenshot showed all DataGrid values (`Q1`–`Q3`, `$18.2M`–`$22.1M`, and statuses); body cell text was absent from the returned accessibility tree.

This supersedes the earlier tree omission as evidence for the current binary's table/header/cell node exposure. Accessibility Inspector output and VoiceOver speech were not captured; spoken header/value acceptance remains open. Evidence: `target/headstart/evidence/r3-datagrid-native-ax-retest-20261020.json`.


## M261 explicit body-cell names (2026-10-20)

`StringTableDelegate` now sets each body cell's accessible name to `<column name>: <value>`. It uses the same accessible column-name fallback as the header (`Column N` when the supplied header is blank). The rendered child text is unchanged. This is a source-level improvement; no new native-tree or spoken-output result is claimed. Recheck the `Quarterly revenue` DataGrid and the plain Table comparison with Accessibility Inspector and VoiceOver before closing cell-value and header-association acceptance.

## DataGrid identity validation

`DataGridView::id` rejects empty or whitespace-only IDs, matching the nonempty stable-ID contract used by Tree and KeyedList. This prevents a blank value from becoming the retained table entity's stable identity. A focused unit regression covers whitespace-only IDs. Native screen-reader speech, RTL, cross-platform behavior, and performance remain open.

## Native Table/DataGrid probe — 2026-10-21

On macOS 27, the named `Quarterly revenue` Table exposed its table role/name, three headers, rows, and cells in the accessibility tree. Clicking Q2 updated the owner selection to `Some(1)` and highlighted Q2; Down moved selection to `Some(2)` and highlighted Q3. The accessibility tree exposed no selected or focused row state, and the focused element remained the window. The blue outline after Down does not establish accessibility focus. Evidence: `target/headstart/evidence/table-datagrid-native-selection-focus-20261021.json`. Disposable app and executable were removed after the probe. This records observed platform behavior; it does not establish a source defect.

Pinned Components source review found `TableState::render_table_row` already applies `.aria_selected(is_selected)` to data rows (`vendor/gpui-component-0.7.3/src/table/state.rs`, around lines 2056–2059). The CUA accessibility-tree observation alone cannot distinguish a platform bridge omission from the CUA representation. Before changing RustUI source, inspect the same fixture with macOS Accessibility Inspector and VoiceOver after click and Down-key selection.

### Source and observation limits

The pinned source confirms that `is_selected` is `TableState::selected_row() == Some(row_ix)` and that the row builder passes this value to `.aria_selected(...)`. Pinned GPUI maps that property to AccessKit's selected state (`vendor/gpui-pre-0.3.9/src/elements/div.rs`, `aria_selected` and node construction). Thus selected state is requested at the GPUI element/AccessKit layer when a row is selected; the 2026-10-21 CUA result does not show that state in its returned native tree. The saved artifact is not a raw AccessKit tree or Accessibility Inspector capture, so it cannot identify which layer omits the state.

Pinned GPUI also provides `.aria_active_descendant()` for projecting focus to a descendant while an ancestor owns keyboard focus. The pinned `DataTable` tracks a focus handle on its table container but does not apply active-descendant focus to rows. The probe reports the window as focused after click and Down, so it does not establish that the table container held accessibility focus or that a row-focus projection was expected. No RustUI source defect or public row-focus contract is established by this evidence. Keep selected-state and focused-row acceptance open until the same interactions are inspected with macOS Accessibility Inspector and VoiceOver; do not treat this CUA observation as an AccessKit or screen-reader result.

## Inspector and VoiceOver follow-up (2026-10-21)

Rebuilt the Playground with `./scripts/cargo-headstart build -p rustui-playground --locked` (log: `target/headstart/evidence/table-accessibility-inspector-voiceover-build-20261021.log`) and recreated the existing disposable bundle path `target/headstart/verification/M56 Review.app` with bundle ID `dev.rustui.playground.validation`. In the CUA run with VoiceOver off, clicking Q2 changed the owner selection to `Some(1)` and Down changed it to `Some(2)`. The app was closed and the disposable bundle and standalone executable were removed; Cargo caches were preserved.

Accessibility Inspector launched, but its Scope Reticle stayed at “Select an element to inspect” after enabling point inspection and clicking the Playground through the bound-app CUA target. The CUA tool cannot move a system-wide pointer into another app for Inspector's reticle. VoiceOver was initially off; it was enabled through System Settings for a follow-up, then returned to off. With VoiceOver enabled, the owner remained at `Some(1)` after clicking Q2 and a plain Down key did not change it. VoiceOver Utility reported its Caption Panel setting enabled, but the CUA app capture did not expose caption text, and no speech transcript was obtained. This follow-up therefore records a tool limitation, not Inspector or VoiceOver acceptance: selected/focused-row semantics and spoken output remain open pending a system-wide pointer and caption/audio capture path.


## M485 source-level column indices (2026-10-21)

`StringTableDelegate` now sets the same 1-based `aria_column_index` on each
rendered `ColumnHeader` and each body `Cell`, using the `col_ix` supplied by
the existing `TableDelegate` hooks. This adds logical column-position metadata
without changing the pinned renderer or dependency revisions. It does not add
an explicit per-cell `LabelledBy` relationship. Native accessibility-tree,
Accessibility Inspector, and VoiceOver header-to-cell association remain
unverified.

Evidence: `target/headstart/evidence/m485-datagrid-column-index-test-20261021.log`
and `target/headstart/evidence/m485-datagrid-column-index-playground-check-20261021.log`.

## M486 source-level row indices (2026-10-21)

Pinned `gpui-pre` exposes `.aria_row_index(...)` and projects it to AccessKit's
`row_index`. Pinned GPUI Components `TableState::render_table_row` assigns data
rows the `Row` role and selected state, but does not assign a row index. The
RustUI `StringTableDelegate` now assigns index 1 to its single header row and
`row_ix + 2` to data rows: indices are one-based, with the header occupying row
1. The focused existing Table suite passed 6/6, and the locked/offline
Playground all-targets check passed. No test inspected rendered AccessKit
metadata; native Accessibility Inspector and VoiceOver verification remain
pending. The source change does not establish header-to-cell association.

Evidence: `target/headstart/evidence/m486-datagrid-row-index-test-20261021.log`
and `target/headstart/evidence/m486-datagrid-row-index-playground-check-20261021.log`.

## M487 native macOS AX metadata probe (2026-10-21)

Built the canonical Playground with the locked/offline command and opened only
`dev.rustui.playground.validation` at `target/headstart/verification/M56 Review.app`.
The existing `Table` route mounted both the plain Table story and the
`Quarterly revenue` DataGrid specimen. The CUA native AX tree exposed the named
`table`, a header `row` with `Quarter`, `Revenue`, and `Status` `cell` nodes,
three body `row` nodes, and named body `cell` nodes including their values.

The returned CUA tree reported roles and accessible names, but did not report
row-index or column-index attributes, or an explicit header-to-cell relationship
such as `LabelledBy`. Accessibility Inspector remained at “Select an element to
inspect” with no selected Playground element, so its attribute details could
not be captured. These are limits of this inspection result; they do not prove
the attributes are absent from the native AX objects. Header association remains
unverified and is not inferred from roles. Evidence:
`target/headstart/evidence/m487-datagrid-native-ax-20261021.json`; build log:
`target/headstart/evidence/m487-datagrid-native-ax-build-20261021.log`.

## M489 — Stage 8 / DataGrid logical row and column counts

The existing DataGrid table-role node now carries the logical column count and
row count, including its single header row. The focused count regression passed
1/1; the Table suite passed 7/7; locked/offline RustUI and Playground all-targets
checks passed. These are source and compile results; native AX exposure and
VoiceOver announcements remain unverified. Evidence:
`target/headstart/evidence/m489-datagrid-aria-count-test-20261021.log`,
`target/headstart/evidence/m489-datagrid-aria-count-table-tests-20261021.log`,
`target/headstart/evidence/m489-datagrid-aria-count-rustui-check-20261021.log`, and
`target/headstart/evidence/m489-datagrid-aria-count-playground-check-20261021.log`.

## M506 partial native AX retest (2026-10-21)

The current locked/offline Playground build exposed the named `Quarterly revenue`
table, its three headers, three body rows, and named body cells in the CUA tree.
That tree did not represent row/column indices or header relationships; this
does not prove those attributes are absent from native AX objects. Accessibility
Inspector selected no element, and no VoiceOver transcript was captured. Native
AX metadata and VoiceOver acceptance therefore remain open. Evidence:
`target/headstart/evidence/m506-datagrid-ax-voiceover-retest-20261021.json`;
build log: `target/headstart/evidence/m506-datagrid-native-probe-build-20261021.log`.

## M549 source-level AccessKit element regression (2026-10-21)

The focused regression inspects AccessKit nodes written by the same builders used in the TableDelegate render callbacks. It verifies the header row role/index, a named ColumnHeader with its one-based column index, a data Cell with its accessible name/column index, and a data-row index using the Row role that pinned DataTable applies to that same element. The locked/offline Table suite passed 8/8. This is element-builder serialization evidence, not a mounted window-tree, native AX, VoiceOver, or header-association acceptance result. Evidence: `target/headstart/evidence/m549-datagrid-table-suite-20261021.log`.


## M562 DataGrid wrapper metadata
The DataGrid wrapper uses one shared element builder for production rendering and the serialization regression. The AccessKit node reports Table role, accessible name, row count, and column count. Focused test passed 1/1 in `target/headstart/evidence/m562-data-grid-accesskit-metadata-20261021.log`. This verifies builder serialization only; mounted-tree relationships, header-to-cell association, and native AX/VoiceOver acceptance remain open.

## M563 selected-row active descendant
`StringTableDelegate` now reports its rendered row as GPUI's active accessibility descendant only when that row is selected and the `TableState` focus handle owns focus. This reuses the pinned public `.aria_active_descendant()` API and the existing Collection pattern; it creates no synthetic IDs and changes no vendored code. The focused predicate test passed 1/1, and `rustfmt --edition 2024 --check` passed. Evidence: `target/headstart/evidence/m563-table-active-descendant-focused-20261021.log`. This establishes the source condition only; native focus ownership, Accessibility Inspector, VoiceOver, and header-to-cell association remain open.
