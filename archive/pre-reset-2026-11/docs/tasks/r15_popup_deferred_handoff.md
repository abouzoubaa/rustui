# R15 native popup work — deferred

**Status: unfinished.** This work is parked to avoid holding MVP progress on a
native window feature. It does not close an MVP evidence milestone or
acceptance check. The remaining native popup behavior must be verified before
RustUI can claim popup support is production-ready.

## Current implementation

- The approved GPUI macOS anchored-popup patch v4 is applied in the current
  consolidation worktree. Patch SHA-256:
  `d83e5fdfa57c0eeaa104b00d65ea717b3f5ca7d6e117cd2ac79f427ebd11febc`.
- V4 is still an uncommitted worktree change in
  `vendor/gpui-pre-macos-0.3.9/src/platform.rs` and `src/window.rs`. The
  patch-test fixture change is also uncommitted in
  `scripts/tests/test_gpui_macos_anchored_popup_patch.py`. Preserve these files.
- Commit `6ace4708` adds a RustUI test that opens, explicitly closes, and
  reopens a popup, checking each root's mount/unmount and parent-only window
  count.
- Commit `ef5856b1` exposes `AnchoredPopupHandle` through the RustUI runtime
  API and gives the Playground native popup separate lifecycle counters plus
  explicit close/reopen controls. This makes the manual probe attributable; it
  is not native acceptance evidence.

## Evidence recorded

- `python3 -m unittest scripts.tests.test_gpui_macos_anchored_popup_patch`:
  7/7 passed. Log:
  `target/headstart/evidence/r04-popup-patch-tests-20261105.log`.
- Focused RustUI popup lifecycle test: 1/1 passed. Current API/probe log:
  `target/headstart/evidence/r15-popup-repeat-lifecycle-handle-api-20261105.log`
  (SHA-256 `9ee623a4a317748aa6750ff239be1141bf43a020317b41226e554aea173f3301`).
- Locked offline Playground check and bin build returned exit 0. Logs:
  `target/headstart/evidence/r15-popover-native-probe-check-20261105.log` and
  `target/headstart/evidence/r15-popover-native-probe-bin-build-20261105.log`.
- `cargo fmt --all -- --check` and `git diff --check` passed.
- None of these checks exercised AppKit popup placement, outside-click
  dismissal, native parent-window teardown, or caller-owned state behavior.

## Open blockers

- Source review found that v4 passes GPUI's top-left anchor rectangle into an
  unflipped AppKit `NSView` conversion without correcting the Y coordinate.
  This may misplace native popups.
- The proposed v5 correction is
  `patches/proposed/gpui-macos-anchored-popup-v5-coordinate-fix.patch`
  (SHA-256
  `5626c7c499b464fec8dc5b88cb55fc091a00ca1084b7150354dc9ea9bb8e442e`). It
  has not been approved or applied. Do not promote it without explicit user
  approval.
- Native acceptance is still open for visible placement, outside-click
  dismissal, parent-window close cascade, and owner-state behavior. The API is
  imperative and has no controlled visibility binding or automatic caller
  state synchronization.
- On 2026-11-05 the protected Playground process in the `c611` worktree was
  explicitly quit at the user's request to free the GUI slot. Its bundle was
  preserved. Do not remove or overwrite that bundle. The slot is now available
  to the active consolidation claim for a later probe.
- Four commits are ahead of `work/m118-diagnostics-provenance` on the
  consolidation branch. That project branch is checked out in the primary
  worktree, which has unrelated untracked files. Do not integrate there until
  the primary worktree is available.

## Resume procedure

1. Renew and audit claim `c44f414bfbf4ef4c4f30` from the registered
   consolidation worktree.
2. Confirm the shared Playground GUI slot is free; reuse the canonical
   disposable bundle and bundle identifier. Preserve the bundle and caches
   owned by other worktrees.
3. Either run the approved v4 build and record any placement defect, or obtain
   explicit approval before applying v5. Do not infer approval from this
   handoff.
4. Bind the GUI observations to the exact source commit, patch hash, executable
   hash, macOS/display details, and owned process. Observe placement at center
   and screen edges, explicit close/reopen, inside/outside clicks, and closing
   the parent while the popup is open. Clean up only the test process, bundle,
   and standalone executable created by that run.
5. Keep the milestone and acceptance check open until the required native
   observations pass. Do not promote source tests or build success into native
   acceptance.
