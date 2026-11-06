# F1c: Live props for retained children

`Component(key, || Child::new()).props(props, reconcile)` returns `ComponentPropsView<Child, Props>`. `Props` must be `Clone + PartialEq + 'static`; `reconcile` is `fn(&mut Child, &Props)`, so a noncapturing closure is accepted. The descriptor and concrete result are available through `rustui::prelude`.

```rust
Component("counter", Counter::new).props(
    label.clone(),
    |counter: &mut Counter, label: &String| counter.label = label.clone(),
)
```

The reconciler projects incoming props into explicitly chosen child fields. On first creation, it runs before `on_mount`. On subsequent lowering at the same scope, key, and concrete child type, it runs only when incoming props differ from the stored previous props, then notifies the child GPUI entity. The initializer remains creation-only, and unprojected child state remains owned by the child. Equal props do not trigger reconciliation or notification.

The reconciler is a stable contract for a keyed component. Changing the callback while props remain equal does not invoke it; change a prop or use a new key when the projection semantics change. The runtime does not compare function pointers. Changing the props type at the same key counts as new incoming props and preserves the concrete child entity. Rendering a plain `Component` at that key clears the comparison baseline, so later reattachment with `.props(...)` reapplies props. Changing the concrete child type creates a new entity and applies its initial props before its mount hook.

Inactive R2-preserved children keep their last applied props. On their next lowering, the current incoming props are compared and applied. Removing a child drops its retained entity and stored props together. The creation-time `EnvironmentValues` snapshot remains unchanged; props do not refresh it.

Runtime compilation and native GUI acceptance are pending the shared milestone verification.


## Milestone 10 environment update

The creation-only environment limitation recorded for this earlier slice is
superseded by F1d: retained child snapshots now refresh before native lowering on
parent renders. Typed `Environment` and `EnvironmentReader` provide scoped values.
Inactive retained children catch up when lowered again. Fine-grained observation
and cached-child invalidation remain unsupported; see `F1D_ENVIRONMENT.md`.
