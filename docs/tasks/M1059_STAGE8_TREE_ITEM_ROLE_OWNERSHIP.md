# M1059 Stage 8 Tree Item Role Ownership

## Finding

`TreeView` renders through GPUI Components' `GpuiTree`. Its row callback also
applies `Role::TreeItem` in `tree_item_accessibility`. The pinned GPUI Components
implementation already wraps every callback result in a `div` with
`Role::TreeItem` (`vendor/gpui-base-0.7.3/src/tree.rs:463`); its tree root emits
`Role::Tree` (`:549`). This makes RustUI's callback row a second nested tree
item. The current helper-level test (`crates/rustui/src/components/data/tree.rs:777`)
checks only the inner element and misses that runtime composition.

## Fix boundary

The duplicate role is confirmed by the two source paths, but a safe correction
needs hierarchy and selection metadata written to GPUI's owning outer row.
RustUI's render callback receives the `TreeEntry` and returns only the child
element; GPUI constructs the accessible row wrapper internally. Removing the
inner role would leave level metadata on a content child while the actual
treeitem still lacks its hierarchy level. The pinned public GPUI API exposes no
callback for setting that metadata on the wrapper. Vendored upstream code must
not be edited directly, and changing the approved dependency/patch process is
outside this task.

No source change or regression test was made because the available public API
does not permit a semantics-preserving fix. This is an accessibility-tree
structure finding from source inspection; no native screen-reader behavior was
observed or claimed.
