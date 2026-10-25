# Table header relationship API boundary

The Stage 8 header-to-cell `LabelledBy` relationship remains open. This is a
source inspection of the pinned GPUI 0.3.9 / Components 0.7.3 surface, not native
screen-reader acceptance. See `DATA_GRID_ACCESSIBILITY.md` for the table slice's
existing roles, labels, indices, and remaining acceptance work.

RustUI already renders real `ColumnHeader` and `Cell` elements through
`table_column_header_element` and `table_data_cell_element` in
`crates/rustui/src/components/data/table.rs`. Cells currently receive an accessible
name containing the column name and value. That name is not an AccessKit
relationship to the rendered header.

AccessKit itself supports `Node::set_labelled_by` (`vendor/accesskit-0.24.1/src/lib.rs:1886`,
the `LabelledBy` property). The pinned GPUI surface does not provide
`aria_labelled_by` or an equivalent real-element relationship setter:

- `StatefulInteractiveElement::accessibility_id` sets an author-visible identifier,
  not the AccessKit `NodeId` used by `LabelledBy`
  (`vendor/gpui-pre-0.3.9/src/elements/div.rs:1319`).
- GPUI creates the real node using `global_id.accesskit_node_id()` before calling
  `Element::write_a11y_info`
  (`vendor/gpui-pre-0.3.9/src/element.rs:431`). That conversion is `pub(crate)`
  (`element.rs:253`), so an owned RustUI element cannot obtain the real header's
  ID through it. `Window::with_global_id` exposes a `GlobalElementId`, but does not
  make its private AccessKit conversion public (`window.rs:3119`).
- `a11y_synthetic_children` allows an owned callback to mutate its parent node
  through `A11ySubtreeBuilder::parent_node` (`window/a11y.rs:364`). It also derives
  IDs for added synthetic children through `synthetic_node_id` (`:335`) and inserts
  them through `push_child` (`:346`). It does not expose the real parent's ID or
  resolve sibling/header element IDs. A generated synthetic child ID does not
  identify the existing `ColumnHeader`.

Consequently, calling `set_labelled_by` from an owned element or synthetic-child
callback still lacks a supported target ID for the actual rendered header.
Reproducing GPUI's private hashing or adding duplicate synthetic headers would
not establish this relationship through a supported public API. No such workaround
was added, and no vendor source or dependency revision was changed.

The next implementation requires an approved upstream API exposing a relationship
to a real element's stable accessibility identity (including virtualization and
multiple-table scoping), or equivalent public node-ID resolution. A setter-only
serialization test with invented IDs would prove AccessKit's setter, not RustUI's
rendered table relationship, so no implementation regression or GUI probe was run
for this blocked slice.
