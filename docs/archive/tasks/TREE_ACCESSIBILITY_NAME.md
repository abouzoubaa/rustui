# Tree node accessible names

`TreeNode::new(id, label)` supplies the label that RustUI forwards to the native
tree row. `TreeView` converts each node into a GPUI Components `TreeItem`, then
renders each visible row with `ListItem::accessibility_label(entry.item().label)`.
An empty or whitespace-only input therefore produced an unnamed tree item.

The constructor now rejects blank labels before the node can reach rendering.
This also applies to nested nodes because all nodes use the same constructor.
The focused unit regression is
`blank_tree_node_accessible_names_are_rejected` in
`crates/rustui/src/components/data/tree.rs`.

Validation: `rustfmt --edition 2024 --check crates/rustui/src/components/data/tree.rs`
and `git diff --check -- crates/rustui/src/components/data/tree.rs
 docs/tasks/TREE_ACCESSIBILITY_NAME.md` passed. `./scripts/cargo-headstart test
-p rustui --lib blank_tree_node_accessible_names_are_rejected --locked
--offline` passed (1 test, 418 filtered out); full output is retained at
`target/headstart/evidence/tree-accessible-name-focused-20261020.log`. No GUI
or native accessibility acceptance run.
# M407 — refresh retained Tree selection binding
Tree's retained selection observer now invokes the current owner updater after same-identity rerenders, preventing stale captured `Binding::new_with` contexts from dropping selection writes. The Tree module suite passed 16/16, including the regression. Native keyboard, accessibility, and platform acceptance remain open. Evidence: `target/headstart/evidence/tree-stage8-m407-module-tests-20261021.log`.
