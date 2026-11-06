# M1281 — Stage 8 SplitView divider event-path acceptance run sheet

Current status (2026-10-25): the sole-session production objective supersedes
the agent-authored test-support investigation gate below. The exact pinned
GPUI feature graph is now available, with 18 additive, checksum-verified registry
packages and no default application dependency-tree change. Ownership tests ran;
the SplitView event-path regression passed all LTR/RTL horizontal and vertical
cases, clamping and ignored-key propagation. The historical
approval wording is retained as provenance, not a new human restriction. Native
AX/VoiceOver and the independent proposed Tree patch remain separate gates.

**Status: preparation only.** This run sheet does not enable GPUI test support,
change Cargo inputs, implement a test, or claim runtime/native acceptance.
M1281 remains gated on approval and validation of the test-support dependency
closure.

## Existing record and source-state boundary

The primary checkout's `docs/tasks/M1281_STAGE8_SPLITVIEW_EVENT_PATH_TEST_READINESS.md`
records that the current pure helper test does not dispatch a GPUI key event or
verify the production `ResizableState::resize_panel` call. It also records an
offline attempt that stopped before tests ran because of missing test-support
dependencies / build artifacts. Treat that as a blocked attempt, not a failed
behavior assertion and not a pass.

The current production implementation described by the primary checkout is in
`crates/rustui/src/layout/split_view.rs`: the focusable `Role::Splitter` divider
installs `on_key_down`; the handler ignores character-input, IME, and modified
events, reads the current native panel size, maps the physical arrow key with
`divider_keyboard_size`, clamps to the effective range, then calls the pinned
`ResizableState::resize_panel` API and stops propagation. M1280 additionally
keeps Left/Right physical in RTL while panel order mirrors. The existing helper
test checks that mapping and clamp only.

**Important source identity check:** the assigned `1b20` worktree is currently
at `567f933`, whose `split_view.rs` does not contain the divider keyboard event
handler or M1280's `panel_order` logic. Any future event-path test must first
use and record a source revision that contains the production path under test;
do not silently copy the primary checkout's dirty implementation into this
worktree as part of this documentation task.

Existing acceptance context is separate: `docs/tasks/SPLIT_VIEW_IMPLEMENTATION.md`
records M586 bounded native pointer/keyboard resizing and the remaining limits;
`target/headstart/evidence/m586-splitter-native-interaction-20261021.json` is
the referenced native observation. That evidence does not satisfy the RustUI
automated event-path regression requested by M1281. The primary checkout's
`target/headstart/evidence/split-view-keyboard-focused-test.log` covers the
helper test only.

## Test host and approval boundary

Pinned GPUI exports `TestAppContext` and test event helpers behind its
`test-support` feature (`vendor/gpui-pre-0.3.9/src/app.rs`). Its
`simulate_keystrokes` / `dispatch_keystroke` helpers dispatch through a window;
the test must focus the rendered divider before sending keys. RustUI currently
has no `TestAppContext` / `VisualTestContext` event harness in its test setup,
and the locked offline feature graph has no GPUI `test-support` edge.

The readiness record identifies `futures-concurrency` as missing from the
vendored source. The pinned `gpui-pre` `test-support` feature also enables
`collections/test-support`, `http_client/test-support`, `proptest`, optional
`futures-concurrency`, `postage`, and `web-time`; platform, GPUI Base, and GPUI
Components expose additional forwarding features. The complete resolved
closure, lockfile delta, and vendor additions have not been established.

**Required owner decision:** authorize an isolated, test-only resolution spike
for the exact pinned GPUI snapshot, including any required Cargo.lock and vendor
additions, to determine whether the RustUI test host builds. Do not make those
changes in the normal workspace, change dependency revisions, or promote a
feature change without the separate approval. M1222's Tree `aria-level` patch
decision is independent and remains open.

## Focused regression after the test host is approved

The test should mount the real RustUI `SplitView` through its production owner
and render path, inside a bounded window with two panels and a measured divider.
It must focus the divider by its rendered focus/element identity, dispatch a
physical arrow through `TestAppContext`, and observe the resulting native
`ResizableState` size or the rendered splitter value. Do not call
`divider_keyboard_size` directly as the acceptance assertion.

Minimum case: for a horizontal split with a known initial size and configured
range, focus the divider and dispatch Left; verify the native size decreases by
the configured 10-point step. Dispatch at the minimum and verify it remains
clamped. Add a vertical Up/Down case if the same test harness can do so without
duplicating setup. For M1280 interaction coverage, repeat horizontal Left/Right
after setting RTL and assert physical direction remains unchanged while panel
order is mirrored.

Add negative event cases only if the host exposes the actual `KeyDownEvent`
properties without synthetic shortcuts: character-input, IME-in-progress, and
modified arrows must not resize the divider; a non-axis arrow must not resize;
an unhandled key must not be swallowed. Keep the source test focused on
production event dispatch, not broad focus, accessibility, or rendering claims.

## Separate runtime acceptance

After the automated event-path check passes on the exact source revision,
perform a bounded canonical Playground run on macOS:

1. Open the SplitView story, focus a divider using keyboard traversal, and
   record starting pane values.
2. Exercise Left/Right on a horizontal divider and Up/Down on a vertical
   divider; confirm native pane sizes change and the splitter value follows.
3. Exercise RTL horizontal layout and confirm the panel order mirrors while
   physical Left/Right meaning remains unchanged.
4. Exercise a configured range boundary; verify the value stays within its
   effective minimum and maximum. Confirm parent rerender preserves the size.
5. Inspect the native accessibility tree for the focusable Splitter role,
   localized name, orientation, current/minimum/maximum values, and updated
   value after key dispatch. Record screen-reader speech separately if tested.

Native interaction, accessibility-tree, and VoiceOver outcomes are distinct
evidence. M586's prior bounded native evidence is context, not a result for a
new binary or current source state.

## Evidence to record

- Claim/task ID, worktree, branch, exact HEAD and source-file hash for
  `split_view.rs`.
- Cargo feature-tree command and exact resolved `test-support` closure; every
  missing crate, lock entry, vendor source, checksum/license, and whether the
  isolated offline build succeeded.
- Focused test name, exact command, exit status, and full uncompressed output;
  state before/after each dispatched key and the observed clamp boundary.
- How the test focused the rendered divider and observed native `ResizableState`
  or splitter accessibility value; confirm it did not call only the pure helper.
- Runtime platform/OS, Playground build identity or executable hash, story,
  locale/direction, actions, native AX snapshot, and evidence file paths.
- Whether the app and disposable test outputs were closed/removed after native
  acceptance; preserve shared Cargo caches.
- Explicit unresolved items: TestAppContext feature approval, exact dependency
  closure, current-source alignment, native AX/VoiceOver/platform coverage.

## What remains gated or unverified

No TestAppContext build or production event-path regression has passed for
M1281. The current readiness note records a test attempt that stopped before
execution; it is not evidence of a code failure. The exact test-support
dependency closure and any persistent lock/vendor changes await owner approval.
Native M586 observations do not substitute for that automated test, and no new
native, browser, accessibility-tree, or VoiceOver acceptance is claimed here.
M1222 remains a separate upstream Tree `aria-level` decision.
