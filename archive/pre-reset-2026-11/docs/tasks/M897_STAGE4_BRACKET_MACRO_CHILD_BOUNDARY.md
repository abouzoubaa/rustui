# M897 — Stage 4 / stop bracket-delimited macro children at their boundary

`parse_view_child` previously stopped constructor parsing at parenthesis and
brace groups, but not a bracket group following `!`. As a result, a
bracket-delimited macro child could consume the adjacent DSL sibling.

The parser now recognizes the closing bracket of a macro invocation as the
constructor boundary. The regression
`bracket_delimited_macro_child_stops_before_the_next_sibling` passed 1/1.
`rustfmt --check` and scoped `git diff --check` passed.

This verifies the parser boundary only. Broader macro and workspace acceptance
remains open.
