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
reduced-motion and all-window refresh factories have now received isolated
native dispatch checks. Public `App` remains supported as an escape hatch.

### Native dispatch follow-up (2026-10-21)

The isolated two-window probe dispatched both public factories through native
RustUI Buttons in the disposable `dev.rustui.playground.validation` bundle.
After `AppAction::reduced_motion(true)`, its chained GPUI readback returned
`true`. After `AppAction::refresh_windows()`, lowering counts advanced from 3 to 4 in the first window and from 4 to 5 in the second after 350 ms.
The control-free first window supports an all-window refresh interpretation. The matched no-refresh control is implemented and compiled but has not run natively; the causal gate remains open. Evidence:

The probe accepts `--idle-control`. Run the treatment and control serially in the same disposable validation app:

1. Treatment: `./scripts/cargo-headstart run -p rustui --example m49_runtime_action_dispatch_probe --locked --offline`
2. Control: `./scripts/cargo-headstart run -p rustui --example m49_runtime_action_dispatch_probe --locked --offline -- --idle-control`

In both arms, click **M49 enable reduced motion**, wait for the stable baseline, then click the measurement button. The control records the trigger without calling `refresh_windows()`; both arms sample after 350 ms. Compare first-window lowering counts: treatment should advance, while control should remain unchanged. The control reports `idle_control_complete` only when no first-window advance occurred and refresh was not dispatched. The second-window count is recorded too, though its own button may lower that root.

Build check: `target/headstart/evidence/m49-runtime-action-idle-control-build-20261021.log`.
`target/headstart/evidence/m49-runtime-action-dispatch-1792590109.json`; build:
`target/headstart/m49-runtime-action-dispatch-build-20261021.log`.

The probe compiled without GPUI `test-support` and did not alter dependency
revisions. Its disposable bundle and standalone executable were removed after
the run. Deterministic automated dispatch tests remain a separate follow-up if
GPUI test-support becomes available without changing the pinned dependency set.

The source suite passed 135 tests across 12 binaries with zero failures
(`target/headstart/milestone46-50-tests.log`). The final build succeeded, and
the dependency reuse verifier confirmed all three pinned GPUI packages fresh
(`target/headstart/milestone46-50-final-reuse-report.json`).

## Next independent candidate

Add deterministic RustUI dispatch tests if the required GPUI test-support
dependencies become available in the pinned offline source set. Native dispatch
effects are now independently verified; this follow-up needs no UI Studio
serialization-schema decision.
