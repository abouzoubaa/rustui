# Proposed GPUI Tree disabled accessibility state

**Status: proposal only.** The proposed patch at `patches/proposed/gpui-tree-aria-disabled.patch` has not been applied, compiled, tested, or promoted. No vendor source or lockfile was changed to prepare this review artifact.

## Confirmed defect

RustUI exposes `TreeNode::disabled(bool)` and passes that value to the pinned `gpui_base::TreeItem` (`crates/rustui/src/components/data/tree.rs`). The pinned GPUI Tree renders each row with the `TreeItem` role, label, selected state, and expanded state. It suppresses pointer handlers for disabled entries, but does not set AccessKit's disabled attribute. GPUI's existing Switch accessibility test explicitly records the same API limitation and asserts that its disabled node is not exposed as disabled. Thus the disabled Tree row behaves disabled for pointer activation but screen readers do not receive its disabled state.

The `gpui-component` Tree wrapper delegates row construction to `gpui-base::Tree`; the relevant fix belongs in the GPUI element accessibility API and the base Tree renderer. RustUI cannot represent this state with the currently pinned API. The patch leaves RustUI behavior unchanged and adds the missing upstream capability.

## Pinned source and integrity context

The repository's approved upstream snapshot is:

| Package | Version and source lineage | Cargo.lock registry checksum |
| --- | --- | --- |
| `gpui-pre` | `0.3.9`, Longbridge packaging of official Zed revision `93d745668805029476e6d7fbb19652116fed51de` | `34f61c1cfd36fc9c436af09308c963822ae27a45a84feb21a77ba5c9e051f1c0` |
| `gpui-base` | `0.7.3`, Longbridge `gpui-kit` revision `67265a36e260ffe03c5aaebdcd8d654dbf892eab` | `2e9955b6f4b048fd50ca15fc8e225d80bb00ba9ebc8fecdddde5ad26dbef9270` |
| `gpui-component` | `0.7.3`, same `gpui-kit` revision; its Tree delegates to `gpui-base` | `465a363235cd4062ef5f0953fc096fc764ed6f53dfbd9f409f669f93ea3b1ff9` |
| `accesskit` | `0.24.1`, pinned dependency; `Node::set_disabled(bool)` is available | `d3b7f7f85a7e5f68090000ed7622545829afd484d210358702ae4cb97dd0c320` |

The package lineage and pin context above match `patches/proposed/README.md`, the vendored package metadata, and `Cargo.lock`. The patch targets only these two files:

| File | Pristine preimage SHA-256 | Proposed postimage SHA-256 |
| --- | --- | --- |
| `vendor/gpui-pre-0.3.9/src/elements/div.rs` | `f320948b8f90e36b7100ac3e0e061235d1e9422aeb323918e313f7d69980cfa9` | `bc8026eee0fd1da8f1e497d140ffccc0f798deb7eb1f2bb6c8ef1282535f8819` |
| `vendor/gpui-base-0.7.3/src/tree.rs` | `e74e4fcdef6ccec174e3e6ced50f108433e726f3e3d46f3539b1f41d5944e5df` | `9ef2efa0a44af810eef3088343b0838179d5e952c6ca8734e5306d5c74736924` |
| `vendor/gpui-base-0.7.3/src/switch.rs` | `d06394441975ac7eb4478d353762c527ca0e9d8034c7f5097902d9d976543567` | `aed20dabde162201be920d297dac451c31b8fc18385c60298f90123c1cae724d` |

Patch SHA-256: `7ac8cc51b65a9f96792ebfdface9a4c6dff8103ccbe9c10b998f6be66b6d1c55`.

Both affected packages are Apache-2.0. Preserve their vendored `LICENSE-APACHE` files unchanged in any isolated candidate. Current license SHA-256 values are `752daf2fb234ca4a1fa372c073fe127f44b7b90fd2529ae44273a64f9d53da7a` (`gpui-pre`) and `d1b0449e5478c574ba4f686c2656df7fe77d66821a61f8b6ed3378a58ed9a811` (`gpui-base`). Current `.cargo-checksum.json` SHA-256 values are `7b8e889ae4ac0cbe651b654f5e39d9003969d09ae4a70aff4147f521eda8faa0` (`gpui-pre`) and `9e96d55b0071b60576bc6ac18c3e9c48e6d0de5ff848821af7623bd151f7c3ed` (`gpui-base`). Preserve those registry manifests unchanged; validate patched path-source candidates by source-file hashes instead of rewriting `.cargo-checksum.json`.

## Smallest proposed change

1. Add `StatefulInteractiveElement::aria_disabled(bool)` in `gpui-pre` and store the optional value alongside the existing selected and expanded ARIA properties.
2. Forward that value to `accesskit::Node::set_disabled` when constructing the accessibility node.
3. Set `.aria_disabled(entry.is_disabled())` on the existing GPUI `TreeItem` row in `gpui-base`.
4. Update the existing Switch test's explanatory comment: the API now exists, but that separate wrapper still does not forward the state. Its current assertion remains scoped to Switch and continues to document its incomplete accessibility behavior.

The method is additive and has a default trait implementation, so existing element implementations and callers retain their behavior. `gpui-base` must be built with the paired `gpui-pre` change because its new method call does not exist in the pristine pinned GPUI API. Apply both changes together in an isolated source copy only.

## Focused validation required before any promotion decision

1. Confirm the exact pristine preimage hashes above, the locked package identities, and intact license/checksum files in a fresh isolated candidate.
2. Apply this patch to the candidate from the repository root with exact paths and no fuzz; verify both postimage hashes.
3. Run the `gpui-pre` and `gpui-base` crate checks against the locked source set. Confirm existing `StatefulInteractiveElement` implementers and call sites remain source-compatible.
4. Add or run a focused GPUI accessibility test following the existing AccessKit probe pattern in `gpui-base/src/switch.rs`: render enabled and disabled tree rows; assert both have `Role::TreeItem`, correct label/selection/expanded values, disabled row `is_disabled() == true`, and enabled row `is_disabled() == false`.
5. Verify disabled rows still suppress pointer actions and enabled rows still accept them; confirm keyboard navigation, expansion, selection, and focus behavior are unchanged.
6. Run RustUI Tree tests and a bounded native screen-reader/AX observation only after isolated source-level validation succeeds. Remove disposable GUI outputs and close the test app afterward.

No build, test, visual, or native check has been run for this proposal.

## Compatibility and scope risks

- The proposal spans two pinned crates and is atomic: applying only the `gpui-base` hunk fails to compile, while the `gpui-pre` API addition alone is behavior-neutral.
- The API is verified only against the pinned `gpui-pre 0.3.9` / `gpui-base 0.7.3` pairing and AccessKit `0.24.1`; other GPUI snapshots may have different accessibility internals.
- The `aria_disabled` field is additive, but `AccessKit` adapters may map disabled state differently across macOS, Windows, and Linux. Native acceptance is still required for each target the framework supports.
- This change exposes disabled state to accessibility clients; it does not alter current keyboard behavior or decide whether disabled tree parents should remain navigable/expandable through keyboard commands.
- Do not edit active `vendor/`, change `Cargo.lock`, change dependency revisions, or promote this patch without the separate approval required for upstream patch intake and promotion.
