# M857 — Preserve conditional stack children

`VStack!` and `HStack!` parse child constructors as `syn::Expr`. A child's
outer `#[cfg]` or `#[cfg_attr]` therefore survives parsing, but the previous
expansion put it on the `.child(...)` argument. When the condition is false,
that removes the argument while leaving the builder call, producing invalid
code. The expansion now moves those conditional attributes to the generated
builder-extension `let` statement, so the entire child addition is omitted.

The focused regression covers both `#[cfg]` and `#[cfg_attr]` attributes and
checks that the generated Rust gates each builder call. It also covers the
existing shorthand `Button` action rewrite on the conditionally included child.

Evidence: `./scripts/cargo-headstart test -p rustui-macros --lib conditional_child_attributes_gate_the_child_builder_call`
passed (1/1). Scoped rustfmt and `git diff --check` passed.
