# M793 — Stage 4 component macro helper-name diagnostics

`#[component]` generates inherent `action` and `action_if_changed` helpers. A
state field with either name also generates a same-named getter, which previously
escaped the accessor-collision validator and reached Rust as a duplicate method
definition. The validator now rejects these names at the macro boundary with a
specific diagnostic; the focused regression covers both reserved names.

Validation: `./scripts/cargo-headstart test -p rustui-macros --lib component::tests::generated_state_accessors_reject_component_helper_name_collisions --locked --offline` passed (1 test). This is source-level macro diagnostic coverage; no downstream compile-fail or broader macro integration check was run.
