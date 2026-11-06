# M1202 — Stage 4 preserve arguments on try-wrapped constructors

The `view!` stack-child parser now recognizes constructor calls wrapped in `?`
when deciding whether a following parenthesized group belongs to that
constructor. Previously `make_view()? ("label") Text("sibling")` was split
incorrectly and rejected instead of preserving both adjacent children.

The regression failed before the fix and passed afterward.

Verification: `./scripts/cargo-headstart test -p rustui-macros --lib
try_wrapped_call --locked --offline` passed (1 passed).
