# M1121 — preserve field postfixes on stack constructors

The stack child parser previously consumed method calls, indexes, `?`, and
`.await` as constructor postfixes, but left `.field` and `.0` behind. This
caused a valid constructor with a field access to fail as the parser attempted
to interpret the postfix as a new sibling.

The parser now consumes named and tuple field accesses as postfixes. The
constructor classifier and attribute accessors recognize field expressions,
preserving conditional attributes attached to the full expression.

Focused regression:

```text
./scripts/cargo-headstart test -p rustui-macros field_suffix_stays_with_its_constructor_before_the_next_sibling
1 passed; 0 failed
```

The regression includes `.0.view`, a following sibling, and a `cfg` attribute
on the field expression. It failed before the parser fix and passed afterward.
`rustfmt --check` reports existing formatting differences in other portions of
the already-dirty source file; no whole-file formatting rewrite was made.
