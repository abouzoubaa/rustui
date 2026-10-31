# M1167 — Stage 4 / preserve calls on parenthesized macro expressions

The stack child parser treated a parenthesized macro expression as an already
constructed view and left a following argument group behind. This rejected
ordinary constructor syntax such as `(make_view![])("first")`, even though
macro expressions are supported stack children and calls on parenthesized
callables are supported.

The parser now consumes the call arguments after a parenthesized macro
expression, while preserving the DSL boundary for an adjacent
`(condition).then(...)` optional child.

Focused regressions failed before the parser fix and pass afterward:
`./scripts/cargo-headstart test -p rustui-macros --lib parenthesized_macro
--locked --offline` (2 passed). Evidence:
`target/headstart/evidence/m1167-parenthesized-macro-callable-tests.log`.
