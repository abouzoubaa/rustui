# M1222 — Stage 8 TreeItem row semantics boundary

## Result

RustUI no longer assigns a second `TreeItem` role and hierarchy state to the
inner `ListItem` rendered inside each tree row. The pinned `gpui-base` Tree
already creates the outer row as a `TreeItem` and supplies its accessible name,
selected state, disabled state, and expanded state for folders. The GPUI
Components wrapper places RustUI's `ListItem` beneath that row. Assigning
another `TreeItem` role to the child produced nested tree-item semantics.

The change preserves the existing tree focus handle, row selection and
expansion behavior, the RustUI row label, and leaf activation callback. It
changes no vendored source or dependency revision.

## Remaining renderer change and approval boundary

The pinned `gpui-base` Tree builds the outer row internally, so RustUI cannot
set its level through the child `ListItem`. The renderer already has the
required information and GPUI already exposes the `aria_level(usize)` setter.
`TreeEntry::depth()` is zero-based; the outer row should set
`.aria_level(entry.depth() + 1)` alongside its existing `TreeItem` role and
label. No new public GPUI API or dependency revision is required.

This one-line change belongs in the `gpui-base` renderer. First request the
change from the pinned upstream source owner. If upstream cannot take it,
prepare an exact-identity, versioned patch through the approved patch process.
Do not edit vendored source by hand or apply a new patch before that approval.

## Source evidence

- `vendor/gpui-base-0.7.3/src/tree.rs`: `TreeState::render` creates the outer
  `TreeItem`, has access to `TreeEntry::depth()`, and can set the one-based
  level on the same row.
- `vendor/gpui-pre-0.3.9/src/elements/div.rs`: the existing `.aria_level`
  setter serializes through AccessKit's level field.
- `vendor/gpui-component-0.7.3/src/tree.rs`: wraps the supplied `ListItem`
  below the base Tree row.
- `crates/rustui/src/components/data/tree.rs`: keeps the RustUI row label and
  leaf activation callback on the `ListItem`.

`./scripts/cargo-headstart check -p rustui --locked --offline` passed. No tests
or GUI acceptance were run.
