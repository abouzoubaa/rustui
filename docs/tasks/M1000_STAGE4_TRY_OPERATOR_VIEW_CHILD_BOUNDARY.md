# M1000 — keep postfix `?` with its stack child

The Stage 4 view-builder parser now keeps a postfix try operator attached to
its constructor expression across the adjacent-child boundary. Constructor
classification and outer-attribute handling recognize `Expr::Try`, so
conditional attributes gate the generated child-builder call as intended.

The focused macro regression passed:

```text
./scripts/cargo-headstart test -p rustui-macros try_operator_stays_with_its_constructor_before_the_next_sibling
1 test passed
```

Formatting and scoped whitespace checks also passed. Broader macro behavior and
Stage 4 acceptance remain open.
