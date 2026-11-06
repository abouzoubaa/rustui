# M1028 — Reject duplicate Navigation retention scopes

The retained component host already rejects duplicate sibling identities for
several retained scope kinds. Navigation stack scopes were missing from that
guard, so duplicate stack identities could collide in retained state.

The host now validates `RetentionScope::Navigation` with the same sibling
identity check. The regression failed before the guard and passed afterward:

```text
./scripts/cargo-headstart test -p rustui --lib duplicate_navigation_stack_siblings_are_rejected_without_key_collisions
```

Scoped `git diff --check` passed. This closes one duplicate identity case;
broader lifecycle, native behavior, and cross-platform acceptance remain open.
