# Focus trap accessibility forwarding proposal

## Status

Proposed review artifact only. This patch is not applied to `vendor/`, and it does not change the dependency lock or active build.

## Finding

The pinned `gpui-base 0.7.3` `Dialog` assigns its accessibility role and then wraps the element with `focus_trap`. `FocusTrapContainer<E>` delegates layout, prepaint, and paint to its base, but omits the three `Element` accessibility hooks. The pinned GPUI `Element` defaults return no role and no node properties, which suppresses the outer dialog accessibility node while its children still appear.

## Proposed change

Patch only `vendor/gpui-base-0.7.3/src/focus_trap.rs` to forward:

- `a11y_role`
- `write_a11y_info`
- `a11y_synthetic_children`

Forwarding all three hooks keeps the generic wrapper transparent to accessibility semantics, including synthetic children. No new dependency or public API is introduced.

## Pinned source and integrity

- Package: `gpui-base 0.7.3` from Longbridge `gpui-kit` revision `67265a36e260ffe03c5aaebdcd8d654dbf892eab`
- Cargo registry checksum: `2e9955b6f4b048fd50ca15fc8e225d80bb00ba9ebc8fecdddde5ad26dbef9270`
- Source file preimage SHA-256: `5a110237efd10a3d7b2478b8d0d3588e82ab1950984bdc71a47333b1d59e8be6`
- Candidate patch SHA-256: `cf5def8ebfb6284f2e63938dda4bd358b344327e3a6951b55eebc0ceffc9040c`.
- Apply requirement: exact preimage and zero-fuzz patch application.

## Compatibility, risks, and fallback

The patch targets only the pinned `gpui-base 0.7.3` source. It affects all `FocusTrapContainer` users, not only Dialog, by restoring semantics supplied by their wrapped base elements. Verify role, label, other node properties, and synthetic-child forwarding for both annotated and unannotated bases. It does not add missing dialog names, modal/background semantics, announcements, or accessibility actions that the wrapped element does not provide.

The fallback is to leave the pinned dependency unchanged and retain the proposal for review. Remove the patch when an upstream revision includes equivalent forwarding and passes the same acceptance checks. Do not update the dependency or promote this patch without explicit approval.

## Evidence

- Source trace: `FocusTrapContainer<E>` omits the accessibility hooks; GPUI `Element` defaults are no role and no-op node properties; `Dialog` sets its role before applying the focus trap.
- Existing native tree evidence before this candidate: `target/headstart/evidence/dialog-accessibility-tree-20261019.json` contains no Dialog role/title, while its action buttons remain.
- Zero-fuzz application to an isolated copy: **passed**. The copy changes only the three forwarding hooks.
- Isolated `gpui-base` check and canonical Playground build: **passed** with `./scripts/cargo-headstart`; commands and output status are recorded in `target/headstart/evidence/dialog-focus-trap-a11y-candidate-20261019.md`.
- Native Playground check: **passed for dialog-role exposure, visible title text, and action children**. Escape removed the Dialog node and produced the Cancel result. The explicit accessible-name property, focus containment/restoration, nested dialogs, and synthetic-child behavior were not verified.

## Validation required before promotion

1. Test forwarded node properties and synthetic children through the wrapper; verify an unannotated base remains absent from the tree.
2. Verify the dialog's explicit accessible-name property, focus containment/restoration, and nested-dialog behavior in the native accessibility tree.
3. Preserve the normal dependency source and lock; do not promote without explicit approval.
