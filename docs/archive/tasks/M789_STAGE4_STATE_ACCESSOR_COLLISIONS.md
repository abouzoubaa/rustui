# M789 — Reject colliding generated state accessors

## Scope

The component macro generates accessors for each `#[state]` field. A field such as `count` generates `set_count`; a second state field named `set_count` could then emit a duplicate method and leave the user with a downstream generated-code error.

## Change

Validate generated state accessor names before code generation and report the collision at the source field span.

## Evidence

`./scripts/cargo-headstart test --locked --offline -p rustui-macros generated_state_accessors_reject_cross_field_name_collisions` passed (1 test). Scoped rustfmt and `git diff --check` passed.
