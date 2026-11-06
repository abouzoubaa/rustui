# R19: proposed Components modal focus recovery

The separate [RustUI integration proposal](../../patches/proposed/rustui-modal-focus-integration-v1.patch)
has SHA-256 `5fcfed26ceef92525e6b650385c685facd1615cfd39ff1f39cb62564700f8c8c`.
It is rebased on the current `crates/rustui/src/runtime/window.rs` preimage
`9899b606b34fae666b9a016773ef4d3584ca5c2d4d4a6decfccbdacd510efd85`; the
proposed postimage is `8d1e02253aaf0cf2497a15d73a417eead5729a678163e99af94689b186ed0274`.
It calls the proposed method directly when no element is focused. The method
returns false when no modal exists; the isolated Components test covers that
case. RustUI initializes Components before creating its Components root, and
the existing fallback already reads that root's `WindowState` for every initial
Tab. The native probe used a temporary guarded variant, not this exact patch.
The exact final integration therefore still requires compilation and native
acceptance after approval.

Run `python3 patches/proposed/check_modal_focus_proposal.py` to verify the exact
current source and patch images and apply and reverse both patches on temporary
copies. This read-only checker is not an approved patch installer. Promotion
still requires the project's versioned apply/reverse process and user approval.

**Status: proposal only.** The pinned vendor source and approved patch stack
are unchanged. Promotion requires the user's approval and the project's
versioned exact-image patch process.

When a native Sheet or Dialog has lost focus, RustUI currently prevents Tab
from reaching background controls, but cannot restore the modal's focus. The
Components `WindowState` owns the focus handles privately. Recording a handle
only when RustUI opens a modal would be unsafe when other callers replace or
nest modals.

The proposed [patch](../../patches/proposed/gpui-component-modal-focus-v1.patch)
adds `WindowExt::focus_active_modal`. It focuses the last active Dialog, or
the active Sheet when no Dialog exists, and returns `false` when neither
exists. This follows Components' existing dialog-over-sheet rendering and
selection order. RustUI's initial-Tab fallback could call it before resuming
ordinary modal traversal. No separate focus registry is introduced.

The patch targets the current selected Components 0.7.3 source after the
existing approved patches. Its SHA-256 is
`353ab857b3085288530fabf397a825d5b20828cc21b64314068ac256dd37fd03`;
preimage hashes are `c438aabc60f18252420a40e773e0aa4447fe89b00e27033c0ad7e5980b9c2749`
for `src/root.rs` and `697b8a6867d1f12085ac6c976568745f6ba4ca21f5a972c7a74e30523ed77eb`
for `src/window_ext.rs`. Applying and reversing the patch with `patch --fuzz=0`
on a disposable copy reproduced both postimages and restored both preimages.

An isolated test using the [retained test source](../../patches/proposed/gpui-component-modal-focus-v1-test.rs)
passed 1/1 for no modal, two nested Dialogs over a Sheet, and restoring the
remaining Sheet. Its temporary lockfile added 18 package identities for test
support; each identity already existed in the project lockfile. The locked,
offline test log is `target/headstart/evidence/modal-focus-proposal-test.log`
(SHA-256 `496fb09fc617734b441b4d17f9a061c2bf4ab7116ff947185849e596e53bc580`).

A separate native probe used the patched copy and a temporary RustUI change
that called the proposed method; an F8-only probe hook cleared focus. On the
exact executable `ae97c2dfb6d13c466dbea432cc6f689dd9e58d1dc2d040b29743731e5c785007`
(PID 56880), Tab restored an unfocused Sheet and Dialog, a second Tab focused
a control inside each, and Space closed each. Probe source and temporary
RustUI patch are retained under `target/headstart/evidence/modal-focus-proposal-`.
The probe process exited, its bundle was removed, and temporary RustUI edits
were reversed. This does not establish full native modal acceptance.

Before promotion, prepare the project-standard exact-image application and
reversal helper, verify composition with the approved patch stack, add the
RustUI integration and regressions, and rerun native and release checks.
Do not apply this proposal to the vendored source without approval.
