# DataGrid accessibility slice

When sibling grids can share an accessible label, set a unique stable identity with `DataGrid(...).id(...)`; the label remains the fallback identity for a single grid.

`DataGrid(table, label)` wraps an existing `TableView` with AccessKit's native
`Table` role and the supplied accessible name. The pinned GPUI Components
`DataTable` already gives each rendered data row the native `Row` role, so the
wrapper provides a table-to-row semantic path without duplicating the table
implementation or changing its selection behavior.

This slice does not claim complete grid accessibility. GPUI Components 0.7.3
does not assign native `ColumnHeader` or `Cell` roles in `DataTable`; visible
column and cell text remains available, but explicit header-to-cell
relationships are not represented. It also does not add or change keyboard
navigation. A follow-up requires an upstream API that exposes those semantics
or a separately approved implementation that can preserve the pinned table's
virtualization and interaction behavior.

The standalone Playground story in `apps/rustui-playground/src/stories/data_grid_accessibility.rs`
demonstrates the named table and controlled selection. It is intentionally not
registered here; the shared story registry is maintained by the parallel D1b
slice.


## M126 native observation

The canonical Table story exposed a native Table named `Quarterly revenue` with native row nodes. Clicking its first rendered DataGrid row changed `Selected source row` from `None` to `Some(0)`. This confirms the wrapper's bounded table-to-row semantics and controlled-selection path on macOS; it does not establish full screen-reader, RTL, cross-platform, or performance acceptance. Explicit `ColumnHeader` and `Cell` roles remain unavailable through the pinned GPUI Components API. Evidence: `target/headstart/evidence/r3-list-datagrid-acceptance-20261019.json`.
