# R1: Owner-current binding updates for navigation

Follow-up: the Stepper issue below was implemented in R1a; see
`R1A_STEPPER_UPDATES.md`. Navigation presentation and unknown-route handling were
updated in N1; see `N1_NAVIGATION_POLICY.md`. Retention lifetime is now implemented under the approved mixed R2 policy; see
`R2_RUNTIME_RETENTION.md` and `R2_COMPOSITION_RETENTION.md`.

## Scope and edits

- `crates/rustui-core/src/binding.rs`: `Binding::update_on` reads a field from the current component owner, applies a mutation, and writes it back only if the mutation reports a change. A failed owner downcast or a no-op returns `false`.
- `crates/rustui/src/view.rs`: `RenderContext::binding_updater` schedules that mutation through the existing `ComponentUpdater`. The host's updater calls `cx.notify()` only when the mutation returns `true`.
- `crates/rustui/src/components/navigation/navigation_stack.rs`: NavigationLink pushes onto the path read at click time; Back pops that path at click time. Public constructors remain unchanged.

## Semantics and limits

`Binding::value()` remains a render snapshot. Navigation callbacks no longer use that snapshot as the base for a path write, so multiple callbacks before a rerender apply in order to the current owner field. Back on an already empty path does not write or notify. The update closure's boolean is its change contract; the helper does not compare arbitrary `T` values. The callback still depends on the weak GPUI owner retained by the host: after release, the host updater silently drops the mutation.

## Dependency edges

`NavigationLinkView` and `NavigationStackView` → `RenderContext::binding_updater` → `ComponentUpdater` → GPUI `ComponentHost` owner update → `Binding::update_on` → owner field read/write. This adds no reactive runtime, dependency, upstream revision, or vendored edit.

## Related work outside R1

- `crates/rustui/src/runtime/component_host.rs` now reconciles retained controls after each root render: entries seen in the frame and declared inactive scopes remain; other entries and their subscriptions are pruned. The scoped-retention implementation and its limits are documented in `R2_RUNTIME_RETENTION.md` and `R2_COMPOSITION_RETENTION.md`; it does not add child-view lifecycle callbacks.

## Acceptance status and future checks

Implementation is complete for the bounded R1 path. No Cargo build, tests, or GUI run were performed in this task; shared compilation is assigned to the parent task. Future acceptance should exercise two push callbacks before rerender, push then Back before rerender, Back after the owner path has become empty, owner type mismatch, and owner release. A native GPUI window interaction check is still needed to establish end-to-end behavior.


## M414 — Wrong-owner update rejection

The R1 contract requires `Binding::update_on` to reject a mismatched component owner by returning `false`, without invoking the mutation or panicking. Added regressions for wrong-owner rejection/no mutation, no-op behavior, and consecutive owner-current updates. Focused tests passed 3/3. Evidence: `target/headstart/evidence/r1-binding-owner-current-tests-20261021.log`. The `rustui-core` suite passed 4/4, RustUI suite passed 522/522, and locked/offline workspace all-targets check exited 0; see the latest Tests row in `docs/IMPLEMENTATION_ROADMAP.md`.

## M445 — Reject wrong-owner binding writes

`Binding::new` and `Binding::new_with` writers now return `false` when the supplied component owner has the wrong concrete type, matching the owner-rejection contract already recorded for `Binding::update_on`. Two focused regression tests passed. Native R1 owner/update checks remain open. Evidence: `target/headstart/evidence/binding-wrong-owner-writes.log`.

## M530 wrong-owner Action dispatch

`Action::new` and `Action::new_if_changed` now return `Invalidation::None`
when dispatch receives a different owner type. The shared regression checks
both constructors, verifies no panic, and confirms the rejected owner was not
mutated. The focused wrong-owner tests passed 4/4 in
`target/headstart/evidence/m530-action-wrong-owner-dispatch-tests-20261021.log`.

## M553 generated component state setter contract

Counter value starts at 0. `set_value(0)` returns false and preserves 0. `set_value(1)` returns true and updates to 1. The public macro integration test passed 1/1 in `target/headstart/evidence/m553-component-state-setter-contract-20261021.log`.

## M615 — Reject stale update writer contexts

Added a focused regression for `Binding::update_on` when its `new_with` writer rejects a captured stale context after the mutation closure reports a change. The call returns `false`; the owner's revision and value remain unchanged. Focused binding tests passed 4/4; scoped rustfmt and diff checks passed. Evidence: `target/headstart/evidence/m615-stage2-update-on-stale-writer-20261021.log`.

## M620 — Generated updater return and mutation contract

The `#[component]` integration test verifies that a generated `update_value`
returns the closure's result and writes the changed field. Focused integration
test passed (2/2); scoped rustfmt passed. It does not assign change-detection
meaning to arbitrary closure outputs.
