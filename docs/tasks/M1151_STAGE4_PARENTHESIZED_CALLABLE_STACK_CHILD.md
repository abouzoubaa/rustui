# M1151 — Stage 4 / keep calls on parenthesized stack constructors

The stack child parser stopped after a leading parenthesized callable, so a
valid call such as `(make_view)("first")` was rejected before its arguments
could be parsed. This conflicted with the conditional-stack contract, which
retains ordinary constructor calls alongside adjacent DSL children.

The parser now consumes the immediate call-argument group when the initial
parenthesized expression is callable rather than an already-constructed view.
It then resumes the existing postfix and sibling-boundary handling.

The new regression failed before the fix. Afterward, the focused `parenthesized`
view-builder tests passed 3/3, covering callable arguments, a parenthesized
constructor with a cfg boundary, and an adjacent parenthesized optional child.
`git diff --check` passed for the macro source. `rustfmt --check` still reports
pre-existing formatting differences in the import block and an unrelated test;
those shared hunks were left untouched. No broader macro, Playground, or native
acceptance is claimed.
