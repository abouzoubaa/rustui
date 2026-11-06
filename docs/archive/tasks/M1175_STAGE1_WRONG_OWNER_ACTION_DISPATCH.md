# M1175 — Stage 1 / contain wrong-owner component action dispatch

`Action` stores an erased owner callback. A component action attached to a
different component's control could reach the wrong `dyn Any` owner during
normal event dispatch and panic on a failed downcast.

The current shared source returns `Invalidation::None` when `Action::new` or
`Action::new_if_changed` receives the wrong owner type. The regression verifies
both action kinds leave the wrong owner unchanged and request no invalidation.

Focused check passed 1/1:

```sh
./scripts/cargo-headstart test -p rustui-core --lib wrong_owner_dispatch_is_a_noop_for_both_action_kinds --locked --offline
```

This is dispatch-unit evidence; no end-to-end GUI action or broader Stage 1
acceptance was run.
