# F1E — Explicit owner callback target

`OwnerActionReader` runs during the current component owner's lowering. It hands
the view builder an `OwnerAction<C>` bound to that exact GPUI entity. A parent can
pass the token to a child through `.props(...)`; the child can turn it into an
`AppAction` with `.action` or `.action_if_changed` and attach it to RustUI Button.

```rust
OwnerActionReader(|owner: OwnerAction<Parent>| {
    Component("child", Child::default)
        .props(owner, Child::set_parent_action)
})
```

`OwnerActionReader` requires `C` to be the current host's concrete component type.
It checks the type before building its child view. `OwnerAction` compares by GPUI
entity ID, so unchanged parent identity does not retrigger props reconciliation,
while two same-type parent hosts remain distinct. The callback carries the host's
existing weak updater, so it does not retain the parent after release. Dispatch
uses the GPUI `App` received by `AppAction`; a dead owner is ignored. A successful
`.action` always notifies; `.action_if_changed` notifies only on `true`.

This is separate from `self.action`, whose target remains the rendering child's
own host. The reader constructs a callback token during lowering; the mutation
only executes later when the `AppAction` fires. It must be placed in the parent
body before entering an explicit `Component` child boundary.

Source check: `view.rs` has a focused equality test for stable/sibling entity IDs
without `C: Clone + PartialEq`. Root owns the shared Cargo check and native
same-type/stale-owner interaction demo. This task adds no new state scheduler,
reactive runtime, upstream patch, or dependency.

## Executed integration verification — 2026-10-18

- Workspace all-targets check passed with the explicit owner callback API.
- Native Playground clicked Left parent once, Right parent twice, and Left local once.
- Observed parent counts were Left=1 and Right=2; local counts Left=1 and Right=0. Each click left the other owner unchanged.
- Binary hash and per-click observations: `target/headstart/milestone15-owner-callback-gui.json`.
- This exercises two same-type owners. Invoking an escaped callback after owner release remains unverified.
