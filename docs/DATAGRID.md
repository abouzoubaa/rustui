## Today

- `DataGrid(TableView, label)` wraps the existing table with a named table role and row/column counts.
- `TableView` accepts string headers and string rows, with `Binding<Option<usize>>` for source-row selection.
- Rows are virtualized; columns reverse visually in RTL while source-row selection is retained.
- Columns are not selectable or sortable; cells are not editable. Header-to-cell relations are not established.
- Tests cover grid identity, selection retention and table metadata, plus table scrolling and RTL presentation.

## API sketch

Keep the existing `DataGrid(TableView, label)` API. Add a separate editable constructor:

```rust
pub fn EditableDataGrid<R: Clone + 'static>(
    label: impl Into<SharedString>,
    rows: Binding<Vec<R>>,
    row_id: impl Fn(&R) -> SharedString + 'static,
    columns: Vec<DataGridColumn<R>>,
) -> EditableDataGridView<R>;

impl<R: 'static> DataGridColumn<R> {
    pub fn text<V: Clone + std::fmt::Display + 'static>(
        id: impl Into<SharedString>, title: impl Into<SharedString>,
        read: impl Fn(&R) -> V + 'static,
        parse: impl Fn(&str) -> Result<V, String> + 'static,
        write: impl Fn(&mut R, V) + 'static,
    ) -> Self;
    pub fn validate(self, f: impl Fn(&str) -> Result<(), String> + 'static) -> Self;
    pub fn sortable_by(self, compare: impl Fn(&R, &R) -> std::cmp::Ordering + 'static) -> Self;
}
```

The sketch type-erases each column's `V`, allowing differently typed columns in one vector.
The owner retains the `Vec<R>`; the grid uses its binding to write the latest owner value.
Row IDs and column IDs must be unique and stable; editing must not change a row ID.
A duplicate ID is rejected without modifying bound data. Selection and editing track IDs,
not visual positions, across sorting and owner updates.

## Editing

- Navigation mode: Enter, F2 or double-click starts editing the active editable cell with its current value;
  typing a printable character starts editing with that character as the draft.
- The editor uses `TextField::validate` from T14 for synchronous validation. Parsing errors use the same
  error display; an invalid draft remains visible and focused, and is not written to the row.
- Enter commits; Tab or Shift-Tab commits and moves to the adjacent cell; moving focus out commits.
  If validation fails, keep edit mode, show the error and prevent an in-app focus move. If the window
  deactivates, preserve the draft and restore the editor when it returns.
- Escape discards the draft and returns focus to the cell. On a valid commit, find the row by ID in the
  latest bound vector, apply its typed column writer, and write the updated vector once.
- If the row disappears, cancel the edit without writing. If the same cell changed externally, retain
  the draft and report a conflict instead of overwriting it. Preserve the draft across virtual scrolling.

## Sorting

A sortable header click cycles ascending → descending → none; Enter or Space on the header does the same.
Sort a view of row IDs with a stable comparator, preserving source order for equal values. Never reorder
or rewrite the bound vector. None restores source order. Keep the active cell by row/column ID, and
hold the current order while an edit is open; recompute after a successful commit.

## Keyboard

- Arrow keys move one cell in navigation mode; clamp at the grid edge. Home/End move to the first/last
  cell of the current row; Command-Home/End move to the first/last grid cell. PageUp/PageDown move
  approximately one visible page in the current column. In edit mode these keys belong to the editor.
- Tab/Shift-Tab move forward/backward between cells in visual reading order, exiting the grid at the
  ends. Enter starts editing in navigation mode and commits in edit mode. F2 starts editing; Escape cancels.
- Navigation scrolls the active cell into view without losing its row/column identity. Read-only cells
  can receive navigation focus but cannot start an edit.

## RTL

Keep left/right arrows physical: in RTL, Left advances to the next logical column and Right to the
previous one. Home/End and Tab order follow logical reading order, so their visual edges reverse in RTL.
Up/Down and PageUp/PageDown do not flip. Sorting preserves the same logical column and row IDs.

## Accessibility

Expose an editable grid role, named column headers, rows and cells, with row/column counts and indices;
associate each cell with its header where the GPUI platform API supports it. Expose the active cell,
selection, read-only state and sort direction. A screen reader should announce the grid name, cell value,
row/column position, header and relevant sort/selection state. While editing it should identify the text
editor and announce validation errors without exposing a stale value. Verify native announcements with
the owner; role metadata and test-window checks alone do not establish screen-reader behavior.

## Out of scope for the first version

Multicell selection, grid-level clipboard and bulk edits, grid undo/redo, async validation, multi-column sorting,
column resizing/reordering, custom editor widgets, tree grids and remote/paged data loading.

## Test plan

- GPUI test windows: each edit trigger; Enter/Tab/focus-leave commit; Escape cancel; read-only cell.
- Invalid draft and parse errors keep editing, show the message, block navigation and leave rows unchanged;
  correcting the value commits once. Exercise the T14 validator and `is_valid` behavior.
- Owner update, row removal, conflicting edit and virtual scrolling preserve or cancel the correct draft;
  duplicate row/column IDs are rejected and retained identity survives reorder.
- Header asc/desc/none, stable ties and restoration of source order; assert bound rows never reorder.
- Arrows, Home/End, PageUp/PageDown, Tab/Shift-Tab, Enter and Escape in navigation and editing modes;
  verify boundaries, scrolling, focus loss/reactivation and mirrored RTL behavior.
- Check grid/header/row/cell metadata, positions, names, selection, sort and error exposure; owner checks
  actual native screen-reader announcements.

## Open questions

1. What if the owner changes the edited cell? **Recommend:** detect the conflict and keep the draft; never silently overwrite the newer value.
2. Should sorting move the row while typing? **Recommend:** freeze view order during editing and re-sort after commit.
3. Should the first version support asynchronous or custom editors? **Recommend:** no; ship synchronous T14 validation and typed text editors first.
