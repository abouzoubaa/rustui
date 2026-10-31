# M856 — Preserve conditional attributes on generated component items

The `#[component]` macro now copies field-level `#[cfg]` and `#[cfg_attr]` attributes onto generated `Default` initializers and state accessors. Without those attributes, conditional fields can be absent from the struct while still referenced by generated code.

The focused regression checks that expansion contains the expected conditional attributes on the source fields, generated state initializer, generated accessor, and generated default initializer.

Evidence: `./scripts/cargo-headstart test -p rustui-macros --lib generated_component_items_preserve_conditional_field_attributes` passed (1/1); scoped `git diff --check` passed. A separate broader crate/doctest run reported a Rust compiler mismatch, so that broader result is not accepted as passing.
