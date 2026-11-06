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

**Owner decision requested:** Can the pinned upstream source owner take this
renderer change? If not, should RustUI proceed through the approved, versioned
patch process? The proposal below remains review-only; applying it requires
separate approval.

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

## Proposed aria-level patch package (review-only)

- Patch: `patches/proposed/gpui-tree-aria-level-v1.patch`, SHA-256
  `67324d2ff67e53c6bc62b7a41609957bca559f0f70e0964fcec17de2ca48f57b`.
- Source identity: `gpui-base 0.7.3`, Longbridge `gpui-kit` revision
  `67265a36e260ffe03c5aaebdcd8d654dbf892eab`; registry archive checksum
  `2e9955b6f4b048fd50ca15fc8e225d80bb00ba9ebc8fecdddde5ad26dbef9270`.
  The pristine `src/tree.rs` checksum recorded in `.cargo-checksum.json` is
  `e74e4fcdef6ccec174e3e6ced50f108433e726f3e3d46f3539b1f41d5944e5df`;
  the unchanged manifest file SHA-256 is
  `9e96d55b0071b60576bc6ac18c3e9c48e6d0de5ff848821af7623bd151f7c3ed`.
- Exact patch transition: current aria-disabled-v1 Tree postimage
  `9ef2efa0a44af810eef3088343b0838179d5e952c6ca8734e5306d5c74736924` to
  proposed Tree postimage
  `4dae8754abef34b1173e0b69b4dfb80483ae61545e2dd3614fc259dfd38fe6fb`.
  The added line is `.aria_level(entry.depth() + 1)` on the outer TreeItem;
  `TreeEntry::depth()` is zero-based and the attribute level is one-based.
- `scripts/apply_gpui_tree_aria_level_patch.py` verifies the package identity,
  patch hash, pristine provenance metadata, and exact source image. Its
  `--check` round-trips the patch in temporary copies and leaves the checkout
  unchanged. `--apply` and `--reverse` are available only for explicit use.
- Approval boundary: this is a proposed patch, not an upstream merge or an
  applied vendor change. Do not apply it until the user separately approves
  the versioned patch application. No Cargo manifest, lockfile, or dependency
  revision change is part of the proposal.
- The proposed GPUI regression asserts `Window::debug_a11y_tree_json()` reports
  levels 1, 2, 3, and 4 on nested outer TreeItem rows. It is not compiled or
  run: the pinned offline vendor sources are missing `futures-concurrency`,
  required when enabling GPUI test support. No dependency was fetched or added.
- Focused regression command, currently unavailable in the locked offline setup:
  `./scripts/cargo-headstart test --manifest-path vendor/gpui-base-0.7.3/Cargo.toml --lib tree::tests::tree_item_accessibility_levels_follow_entry_depth --locked --offline`.
- Runtime oracle remains open: in the expanded Playground Tree reconciliation
  story, expect Workspace 1, Branch 2, Nested child folder 3, Grandchild leaf
  4, Stable sibling 2, Archive 1, and Archived item 2. Verify each outer row
  appears once with the expected name, level, and applicable state; verify
  keyboard selection and expansion; confirm the child ListItem adds no second
  TreeItem; inspect native hierarchy and VoiceOver announcement.
- Validation: `python3 scripts/apply_gpui_tree_aria_level_patch.py --check`
  verifies the temporary-copy apply/reverse hash round trip. The test is
  uncompiled and unrun; no GUI acceptance was run for this proposal.
