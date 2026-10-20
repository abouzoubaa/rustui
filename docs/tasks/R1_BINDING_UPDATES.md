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
