# F1c child props Lab demo

The Layout story demonstrates parent-driven child props. Use **Prefix child
titles** to change the Alpha and Beta titles while retaining each child counter
and notes editor contents. Use the existing reorder and remove/restore controls
to compare keyed retention with removal.

The preview points to `apps/rustui-lab/src/canvas.rs` and illustrates
`Component(key, initializer).props(title, ChildComponent::apply_title)`. The
reconciler updates only the child's label. The child owns its counter and notes;
the parent changes only the title prop.

This source integration has not been built or run in the GUI. It describes the
intended interaction and does not establish visual or runtime acceptance.
