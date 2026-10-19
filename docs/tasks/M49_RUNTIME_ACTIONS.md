# M49 — RustUI-owned runtime action factories

## Scope

This small API addition lets Playground request two common GPUI runtime
operations through named RustUI `AppAction` factories. It narrows ordinary
Playground code without removing the public `App` escape hatch needed by
advanced framework integrations.

`AppAction::reduced_motion(bool)` sets GPUI's reduced-motion preference when
dispatched. `AppAction::refresh_windows()` requests a refresh of all GPUI
windows. Render-diagnostics actions now have corresponding factories:
`enable_render_diagnostics()`, `clear_render_diagnostics()`, and
`disable_render_diagnostics()`. `read_render_diagnostics_snapshot(callback)`
passes a point-in-time `RenderDiagnosticsSnapshot` to
`impl Fn(RenderDiagnosticsSnapshot) + 'static`, without exposing `App` to the
caller. The callback runs synchronously during dispatch and receives an empty
snapshot if diagnostics are disabled. Enable starts a fresh session, clear
resets counters without ending the session, and disable removes the session.
These factories are implemented in `crates/rustui/src/runtime/events.rs`;
`AppAction::new` and `.then(...)` remain available for advanced uses.

## Ordering and notification behavior

`AppAction::then` runs its left action and then its right action synchronously
within the same click dispatch and GPUI event context. Playground uses that
order so the work that changes or samples state finishes before refreshing:

- Animation controls call `AppAction::reduced_motion(true/false)`. They do not
  add a separate window refresh; this preserves the prior
  `set_reduce_motion` behavior.
- Theme reload stores its result string before chaining
  `AppAction::refresh_windows()`.
- Render diagnostics enable, disable, clear, and snapshot actions finish their
  diagnostics or snapshot update before chaining the refresh.
- The cache gallery uses the diagnostics factories and then its existing
  `OwnerAction::action` to refresh only the gallery owner. Snapshot sampling
  completes before that owner refresh, preserving the sample boundary. No
  all-window refresh is introduced.

## Source audit boundary

The Playground callers migrated for this milestone no longer call
`cx.set_reduce_motion(...)` or `cx.refresh_windows()` directly. Those GPUI
method calls now live only inside the framework factories.

The cache gallery's diagnostics buttons and snapshot reader no longer create
raw `AppAction::new` callbacks. They use the named factories while retaining
their current action order and owner-only notification path.

This is not a complete removal of GPUI context from Playground. Theme file
reload and render-diagnostics APIs still accept the `App` context supplied to
`AppAction::new`; startup theme activation also uses an `AppAction` closure.
Lifecycle hooks can still name `rustui::App`, and RustUI continues to re-export
`gpui::App` for framework compatibility. These are remaining runtime-context
escape paths, not direct Playground view composition with GPUI or
`gpui_component` types. The milestone does not add a second runtime.

## Bounded native acceptance

The native run used `verification/M46 M50 Final Review.app`, binary SHA-256
`4b5629b3d2c92b42854bfd0fed515bd81b685f93a93b1e919d919cdf3446c347`. The
recorded sequence showed:

- Enabling diagnostics advanced display revision from 0 to 1; rerendering the
  parent advanced its tick from 0 to 1.
- Sampling advanced revision 1 to 2 and displayed four lowerings each for
  `CachedLeaf`, `UncachedLeaf`, and `CacheGallery`.
- Disabling then sampling advanced revision 2 to 4 and displayed zero counts.
- Enabling, clearing, then sampling advanced revision 4 to 7 and displayed two
  counts each, confirming recording resumed after clear.

Observations are in `target/headstart/milestone49-runtime-actions-gui.json`.
They verify the diagnostics factory order and owner-targeted refresh behavior;
they do not establish cache reuse because accessibility was active. The
reduced-motion and all-window refresh factories were source/build verified but
did not receive isolated native dispatch checks. The public `App` escape hatch
remains supported.

The source suite passed 135 tests across 12 binaries with zero failures
(`target/headstart/milestone46-50-tests.log`). The final build succeeded, and
the dependency reuse verifier confirmed all three pinned GPUI packages fresh
(`target/headstart/milestone46-50-final-reuse-report.json`).

## Next independent candidate

Add focused RustUI dispatch tests for the reduced-motion and all-window refresh
factories, whose individual effects were not isolated in the M49 native run.
This needs no dependency or UI Studio serialization-schema decision.
