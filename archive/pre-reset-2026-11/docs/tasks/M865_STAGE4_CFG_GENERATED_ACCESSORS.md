# M865 — Stage 4 / preserve conditional attributes on generated accessors

## Contract

When a component field is conditionally compiled, every generated accessor
that references it receives the same field-level `cfg` and `cfg_attr`
conditions.

## Implementation

The component macro now emits those attributes for the getter, setter,
updater, binding accessor, and state accessor. The expansion regression checks
both conditional attributes on all five generated methods.

## Evidence

- `./scripts/cargo-headstart test -p rustui-macros --lib generated_component_items_preserve_conditional_field_attributes --locked --offline` passed 1/1.
- `git diff --check -- crates/rustui-macros/src/component.rs` passed.

This verifies macro expansion structure; cross-crate feature combinations
remain broader integration coverage.
