# R3b: Stable-ID virtualized List

`KeyedList(id, items, selection)` is an additive string-list API. `items` are
`KeyedListItem::new(id, label)` values and `selection` is a
`Binding<Option<String>>`. The list identity is required and stable across owner
renders. Row IDs must be nonempty after trimming and unique within the list;
invalid input panics at construction. The existing `List` source-index API is
unchanged.

The adapter retains the pinned GPUI Components `ListState`. Its delegate keeps
source items and a filtered visible-to-source index map. Native rows use their
stable item IDs, while native selection uses the current visible index for the
bound ID. Each row's label is an explicit GPUI `Label` child with an
accessible label; the pinned list still owns the outer `ListItem` role and
selected state. Filtering a selected row out of view retains the owner ID and restores
native selection when the row becomes visible again. Inserting or reordering
items retains the selected ID. Removing the selected item clears native
selection and defers an owner binding update to `None`; the deferred update
checks that the owner still holds the removed ID. Native selection and cancel
events write the bound ID only when it changes, avoiding notification feedback.
The upstream search path temporarily selects its first row; the adapter defers
reconciliation back to the bound ID after that update.

Builder options mirror the existing `ListView`: `height`, `row_height`,
`searchable`, `selectable`, `scrollbar_visible`, and `search_placeholder`.
All options are applied on subsequent owner renders. The retained list's
`searchable` and `selectable` flags are changed only when their requested values
change, because the upstream setters notify unconditionally.
The list renders text labels only. It does not add arbitrary RustUI row builders,
a new virtualization runtime, or a scroll-position persistence contract across
data replacement.

## Acceptance

- Focused pure tests cover duplicate/empty IDs, selected-ID mapping through
  insertion, reorder, filtering, and missing-ID normalization.
- M34 validation: `target/headstart/milestone34-35-tests-accepted.log` records 97
  passed tests across 12 binaries, with 0 failures. The accepted M34 scope and
  native observations are recorded in
  `target/headstart/milestone34-keyed-list-gui.json`.
- Native M34 checks used 1,000 rows and verified selection through reverse,
  prepend, filtering, removal, keyboard selection, scrolling away/back, and row
  labels in the native accessibility tree. The selected stable ID survived
  insertion/reorder/filter; removing it normalized selection to `None`.
- The native row range showed 14 AX row containers for a 1,000-row source, but
  render-call counts were not measured. Outer upstream row paths remain
  position-based. Arbitrary stateful RustUI rows/cells, full screenreader
  acceptance, and exact render-call bounds remain outside the accepted scope.

The explicit text child supplies native row names; full screenreader behavior
remains unverified.
