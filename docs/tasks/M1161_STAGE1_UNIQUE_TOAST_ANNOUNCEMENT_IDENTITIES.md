# M1161 — Stage 1 / allocate toast announcement identity per dispatch

`ToastBuilder::show()` returned a reusable `AppAction`, but allocated its
semantic scope identity while constructing the action. Dispatching that action
more than once could therefore enqueue multiple toast notifications with the
same accessibility identity. Allocate the identity inside the window action so
each dispatch receives a fresh scope.

Regression: `components::overlay::toast::tests::each_toast_dispatch_gets_a_distinct_announcement_identity`.

Focused evidence:

```text
./scripts/cargo-headstart test -p rustui --lib each_toast_dispatch_gets_a_distinct_announcement_identity
1 passed; 0 failed
```

This focused unit check covers identity uniqueness only. It does not establish
native screen-reader announcement behavior or complete Stage 1 acceptance.
