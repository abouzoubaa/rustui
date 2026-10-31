# M1174 — Stage 4 / preserve chained constructor call arguments

The stack-child parser stopped a constructor at its first call argument group.
That split valid Rust call chains such as `make_view()("label")` before the
second argument group, even though the expression is accepted as a constructor.

The parser now consumes call-argument groups after a parsed call expression,
while preserving parenthesized constructor children and optional-child
boundaries as adjacent siblings.

The focused regression covers both the chained call followed by `Text` and a
parenthesized constructor sibling. The focused macro test passed 1/1:

```sh
./scripts/cargo-headstart test -p rustui-macros --lib chained_call --locked --offline
```

No broad workspace or native GUI acceptance was run. The source-level parser
contract remains separate from full Stage 4 acceptance.
