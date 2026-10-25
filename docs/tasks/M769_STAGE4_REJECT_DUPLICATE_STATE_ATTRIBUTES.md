# M769 — Stage 4 / reject duplicate component state attributes

The component macro now reports a targeted error on a field that declares
`#[state(...)]` more than once, rather than allowing an extra state attribute
to leak into generated output. A focused macro regression covers the duplicate
case.

The focused macro test passed (1/1); scoped rustfmt and `git diff --check`
passed. Broader macro integration and Playground acceptance remain open.
