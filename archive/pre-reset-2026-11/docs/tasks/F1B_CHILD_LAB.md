# F1b child host Lab demo

The Layout story now shows two keyed child components, labeled Alpha and Beta,
alongside the existing root-bound keyed editor example. Each child owns a counter
and notes editor. Their repeated local increment button uses the same child-local
ID in each retained child host.
The existing Reverse keyed rows control reorders both examples, and Remove Beta
removes Beta from both. Reordering is intended to retain each child's state;
removing and restoring Beta is intended to create fresh child-owned state.

The preview shows the intended public usage:

```rust
let alpha_label = String::from("Alpha");
Component("child-alpha", move || ChildComponent::named(alpha_label.clone()))
```

`ChildComponent`, including its action and text binding, is defined in
`apps/rustui-lab/src/child_component.rs`. The initializer captures only the
child's initial label; it uses no parent bindings or actions. This source
integration has not been built or run in the GUI, so lifecycle behavior remains
unverified.
