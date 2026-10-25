# M992 — Stage 4 / reject conditionally nested component state attributes

The component macro recursively detects `#[state(...)]` nested in
`#[cfg_attr(...)]` and rejects the unsupported placement at the macro
boundary instead of forwarding it as a downstream unknown field attribute.

Focused check: `conditionally_nested_state_attributes_are_rejected_at_the_macro_boundary`
passed. Scoped `git diff --check` passed.
