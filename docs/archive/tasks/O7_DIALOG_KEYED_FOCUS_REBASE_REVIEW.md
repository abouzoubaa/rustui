# O7 tracked dialog patch review: focus rebase and native dismissal

**Status: revised proposal; M77a isolated manual native checks and M79 isolated scripted assertions only. Not promoted.** `patches/proposed/gpui-component-tracked-dialog-focus-rebase.patch` is a 366-line diff against the pristine pinned `gpui-component 0.7.3` source. Its SHA-256 is `07a507ac52c21bb4f3205a4358b6b27458941e5274e91be4d31bfc482bbb3462`. It was generated from source in memory and passed `patch --dry-run -p1`; it has since been exercised in an isolated candidate for M77a manual native checks and M79 scripted assertions. Full automated correctness coverage is still absent, and the patch has not been approved for intake or normal dependency promotion. The original 272-line O7 patch remains intact with SHA-256 `1da71c0eb4f232c724398dfd30b8fc119564efb3eb30e2c5fb2e0aa50b9cce72`.

## Findings in the original proposal

The original diff assigns a unique `DialogId` to each opening, exposes tracked open/presence/close operations, rewires the next opening's predecessor focus when removing a lower dialog, and increments a generation on stack mutations. These are needed for controlled RustUI ownership. Two close paths still violate that ownership:

1. After Confirm defers closing B, `pending_focus_restore` can hold focus inside A. An ordinary `close_dialog` can then remove A, but the original diff leaves the pending target in A. Its generation check suppresses the old timer but a subsequent opening C consumes the stale target; C may then fail to restore background focus. The revised diff rebases an existing pending target to the predecessor of the opening removed by the ordinary top close.
2. GPUI Base runs `on_ok` or `on_cancel` before `request_close`. If that callback opens C while dismissing B, the original request closes the new top C because it still calls unkeyed `close_dialog` or `defer_close_dialog`. The revised diff tags each rendered native `Dialog` with its opening ID and routes its native request to that ID. Deferred Confirm retains the existing delay when its opening is still top; if a callback made it a lower opening, it removes that opening immediately and leaves the new top focused.

The tracked API remains additive. Stale and cross-window IDs have no effect. Programmatic keyed close still does not invoke the native `on_close` callback. The builder is still repeatable and must capture persistent content. No RustUI adapter or public `Dialog` constructor is included.

## Isolated validation before adoption

Use an approved isolated copy of the pinned package, verify pristine source checksums and license, apply this revised diff without fuzz, and record postimage hashes. Compile the candidate and unchanged `WindowExt` users. In native tests, cover A/B/C middle removal, successor focus, stale and cross-window IDs, Confirm followed by ordinary close of its predecessor then a new opening, and callbacks that open a new dialog during Confirm or Cancel. Check that the new top remains open, that the original opening closes, and that focus returns to the correct predecessor. Include Escape, backdrop, close button, selection scope, callback counts, and repeated builder renders. Only after that should a separate RustUI adapter and GUI/AX acceptance be considered.

This review does not establish compile, runtime, or visual correctness. Keep the candidate isolated if any validation fails; do not edit `vendor/` or its registry checksum.

## M79 isolated scripted assertions

The isolated candidate recorded 19 passing assertions across two result files.
`milestone79-scripted-dialog.json` records a `binary_sha256` value of
`689e4e08481ae9bc9260106050024b76f625ea6d7e54bfab95ab25a276e3a2ce`, the
result-file hashes, and `normal_dependency_promotion: false`. The stale-ID
result has 10 checks for closing A, rejecting repeated/stale IDs, allowing a
fresh ID, and leaving no active A. The double-confirm result has 9 checks for
two Confirm dispatches, one deferred close, same-length replacement with C
active, focus remaining in C after timers, background remaining unfocused, and
two native close callbacks. Both result records identify the patch hash above
and have `runtime_binary_sha256: null`; treat this as narrow isolated scripted
state assertions, not runtime-binary or GUI acceptance.

Evidence:
`target/headstart/candidates/dialog-focus-rebase-20261018-a/evidence/milestone79-scripted-dialog.json`,
`native-scripted-stale-ids.json` (SHA-256
`05050b0f41bde941dd2d0e5e55111cc54f294e6c5512ce82969004a7b3201015`), and
`native-scripted-double-confirm.json` (SHA-256
`42561c5de7805fffd16d6e8d3d55972e6f076a3b6c50d2eeadfd6080a79a3473`).


## M77a partial native evidence

Manual CUA checks on the isolated revised patch are recorded in `target/headstart/candidates/dialog-focus-rebase-20261018-a/evidence/milestone77a-native-dialog.json`; screenshots are in the same evidence directory. With A→B→C open, keyed removal of middle B left C usable and preserved its text selection/focus. Closing C and then A restored typing to A and the background. B's native OK and Cancel callbacks opened C while B closed, leaving C usable. A deferred Confirm on B followed by ordinary close of A and opening C left only C; C accepted typing and closing it restored background typing. Further checks observed that removing bottom A preserved C and Escape on B restored background typing. Backdrop and native Close each removed A and restored background typing.
The unpatched baseline probe was launched from `target/headstart/candidates/dialog-focus-rebase-20261018-a/visual/Dialog Baseline.app` (binary SHA-256 `dca0b287186648b731d95f2ae347c1a202434aabb6c5a7a6ee6847dc8be03526`; package metadata in `target/headstart/candidates/dialog-focus-rebase-20261018-a/evidence/baseline-dialog-review-package.json`). After seeding background and A input, the operator opened B with “OK opens C” and pressed Confirm. The baseline left B visible and removed C: `native-baseline-ok-wrong-close.png` shows Dialog B and its event log, and `native-baseline-ok-wrong-close.ax.txt` contains B’s content input but no C input. In the same callback sequence, the revised isolated candidate removed B and kept C visible and usable (`native-ok-opens-c.png`). This is one manual before-and-after reproduction of the callback-close defect.

These are bounded manual observations on isolated candidate and baseline probes. Together with M79, they do not establish full acceptance, complete automated correctness, or normal dependency promotion. Automated and timing-sensitive race variants, same-length by-ID deferred sequences, cross-window stale IDs, broader entity-release and callback counts, and remaining dismissal/focus paths still need coverage. The structured evidence records these observations and gaps.

The updated post-run counter capture, `native-final-release-counters.png`, shows
15/15 weak `InputState` probe entities retired/released, with the background
focused. This count covers those probe input entities only, not every allocation;
broader entity-release and callback counts remain open.
