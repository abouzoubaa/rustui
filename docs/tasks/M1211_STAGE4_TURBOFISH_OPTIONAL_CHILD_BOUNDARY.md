# M1211 — Stage 4 preserve turbofish optional-child boundaries

The `view!` parser now recognizes turbofish arguments on parenthesized
`.then(...)` and `.then_some(...)` optional children. Previously
`(visible).then_some::<TextView>(...)` after a constructor could be consumed as
call syntax attached to that preceding child.

The regression confirms the optional child stays between adjacent constructor
siblings. It failed before the fix and passes now.

Verification: `./scripts/cargo-headstart test -p rustui-macros --lib
turbofish_then_some_optional_child --locked --offline` passed (1 passed, 0
failed; 38 filtered). The scoped whitespace check passed.
