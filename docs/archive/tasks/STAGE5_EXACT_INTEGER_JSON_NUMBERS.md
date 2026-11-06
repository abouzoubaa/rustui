# Stage 5 — preserve exact JSON integer semantics

Theme integer fields now parse their original JSON number spelling instead of
rounding decimal numbers through `f64`. This prevents tiny fractional tails
such as `1.0000000000000000001` from being accepted as an integer after IEEE
754 rounding, while retaining integral decimal and exponent forms such as
`1.0` and `1e0`.

Focused validation:

```text
./scripts/cargo-headstart test -p rustui theme::schema::tests --lib
19 passed; 0 failed
```

This closes a parser correctness gap only. Broader native visual theme
acceptance, accessibility, platform coverage, and SearchField's approval-gated
upstream integration remain open.

## M1122 follow-up — count significant digits before exponent expansion

The range guard had a second boundary defect: it counted leading fractional
zeroes before appending exponent zeroes. It now counts significant digits, so
`0.` followed by 32 zeroes and `1e34` is accepted as the in-range integer 10.
The focused `parse_theme` regression passed (1 passed, 0 failed), and scoped
`git diff --check` passed. Broader theme acceptance remains open.
