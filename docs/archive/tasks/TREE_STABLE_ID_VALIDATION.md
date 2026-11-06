# Tree stable-ID validation

`TreeNode::new` rejects IDs that are empty or contain only whitespace. Tree
IDs are the stable identity used by native TreeState, controlled selection,
and keyed reconciliation, so accepting a blank ID would allow an invalid
selection identity into those paths. Nonblank IDs are preserved exactly; the
validation trims only to determine whether an ID is blank.

`TreeView::id` also rejects an empty or whitespace-only component identity.
That identity selects the retained TreeState entity, so a blank explicit ID
could alias unrelated retained state. Nonblank identities remain unchanged.

The focused regression is `blank_tree_node_ids_are_rejected` in
`crates/rustui/src/components/data/tree.rs`. This source check does not close
native keyboard, focus, accessibility, localization, RTL, or cross-platform
acceptance for Tree.

## M430 — reject blank explicit Tree component identities

The focused Tree module suite passed 17/17 with locked/offline Headstart.
Focused and full-module evidence:
`target/headstart/evidence/tree-nonblank-component-id-m430-20261021.log` and
`target/headstart/evidence/tree-component-identity-tests-m430-20261021.log`.
