# Stage 8 Tree Root Accessible Name — GPUI Patch Proposal

**Status:** proposal only; not applied to `vendor/`, not promoted, and not a RustUI implementation or browser acceptance result.

## Decision

The TreeView root-name gap cannot be closed through the current RustUI or pinned GPUI public APIs without an upstream API addition. RustUI constructs the styled `gpui_component::tree::Tree` in `crates/rustui/src/components/data/tree.rs` at the existing `GpuiTree::new(...)` call. The actual `Role::Tree` node is created inside `gpui_base::Tree`; the row callback controls descendants and cannot label that root.

The narrow proposed API is an optional `.accessibility_label(...)` builder on both `gpui_base::Tree` and `gpui_component::tree::Tree`. The component forwards the value to the base tree, which applies `aria_label` to its existing `Role::Tree` root. RustUI can then resolve a localized accessible name and pass it at the existing `GpuiTree::new(...)` call. This retains one tree wrapper and does not add another role node.

RustUI should expose its localized form as `.localized_accessibility_label(...)`, following the existing literal/localized naming convention, and should supply a stable non-empty localized name for this control. It should not infer the tree name from a row label or visible placeholder.

## Pinned source identity and preimages

This proposal was prepared against committed `HEAD` `567f933c76b90fde3fcdda4ff45dc660df1eb217`, not the dirty vendor files in the shared checkout.

- `gpui-pre 0.3.9`: approved Longbridge packaging of official Zed revision `93d745668805029476e6d7fbb19652116fed51de`.
- `gpui-base 0.7.3` and `gpui-component 0.7.3`: Longbridge `gpui-kit` revision `67265a36e260ffe03c5aaebdcd8d654dbf892eab`.
- Registry package checksums: gpui-base `2e9955b6f4b048fd50ca15fc8e225d80bb00ba9ebc8fecdddde5ad26dbef9270`; gpui-component `465a363235cd4062ef5f0953fc096fc764ed6f53dfbd9f409f669f93ea3b1ff9`.
- Pristine source SHA-256: `vendor/gpui-base-0.7.3/src/tree.rs` `e74e4fcdef6ccec174e3e6ced50f108433e726f3e3d46f3539b1f41d5944e5df`; `vendor/gpui-component-0.7.3/src/tree.rs` `121c7b715b51d520e531682a14f9ba2791cd3a39554d68ab7cab289bfe7814f9`.

Both packages retain their upstream Apache-2.0 license files. This source-only patch introduces no package, feature, or lockfile dependency changes. Existing callers remain source-compatible because the new name is optional and defaults to `None`.

## Patch artifact

- Patch: [`gpui-component-tree-accessible-name-v1.patch`](../../patches/proposed/gpui-component-tree-accessible-name-v1.patch)
- Patch SHA-256: `431b914cb22038afe3f8cf4e2e8ee5818c4adc85a361cd73cfb40c4951aaff83`
- Proposed postimage SHA-256: gpui-base `src/tree.rs` `f48f0603718d5aa4718cf48079f380570602d66fd2bc3e77066560624f761603`; gpui-component `src/tree.rs` `12e2e2eca25f85878c4ab681e8fae4e58342c6ac9b3cc60a831e4c8894d53df4`.

The patch passed an isolated `git apply --check`, was applied only to temporary copies of the two committed preimages, and produced the recorded postimage hashes. It has not been compiled or applied to the shared vendor tree.

## Focused renderer check

When the pinned GPUI test harness is available, add a focused GPUI Components renderer regression that constructs a tree with a known accessible label, renders it, and inspects `Window::debug_a11y_tree_json()`. Assert that the existing root is exposed as role `tree` with exactly the supplied name and that its row descendants retain their expected `treeitem` roles and names.

This renderer regression is not run or claimed as passing. The current M1222 validation record says the corresponding vendor renderer test is unavailable in the locked offline setup because enabling GPUI test support requires the absent `futures-concurrency` dependency. A RustUI-only label-resolution test would not prove that the rendered root receives the name.

## Browser accessibility acceptance

After the patch is accepted and integrated, use a fresh current Playground Web bundle and inspect the browser accessibility tree. Acceptance requires a `tree` node with the expected computed localized name and the expected `treeitem` descendants. A visible label, successful bundle load, canvas image, or framework-level accessibility test alone is insufficient.

Current X1B evidence records that browser snapshots expose a generic page image and text-entry area without Playground controls. Therefore this Tree root name has no browser acceptance evidence yet; it remains gated on the broader GPUI Web accessibility bridge exposing the Playground controls.

## Compatibility, fallback, and removal

- Keep the API optional for existing GPUI callers; RustUI's TreeView contract should provide the required localized name.
- If the upstream API is not accepted, keep the RustUI root-name capability unavailable and report the gap. Do not synthesize a name from a row or placeholder.
- Remove this patch only when the pinned upstream GPUI Components API provides an equivalent way to label the existing `Role::Tree` root and the renderer and browser acceptance checks pass on that API.

No changes were made to `vendor/`, RustUI implementation files, `Cargo.toml`, or `Cargo.lock` for this proposal.
