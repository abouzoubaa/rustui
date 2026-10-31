# M1139 — Stage 4: preserve component cfg on generated impls

The `#[component]` macro applies the component's effective `cfg` presence
predicate to its generated `Default` and inherent impls. This prevents the
generated impls from referring to a component type excluded by its own
conditional compilation attributes.

Focused regression: `generated_impls_preserve_component_cfg` passed 1/1 with
`./scripts/cargo-headstart test -p rustui-macros generated_impls_preserve_component_cfg --lib`.

This is focused macro expansion evidence; no full workspace check or runtime
acceptance is claimed.
