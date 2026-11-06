# M1115 — Stage 6 / preserve exact-zero plural branch precedence

`exact_integer_from_decimal` parsed the exponent before recognizing an
all-zero mantissa. For a zero value with an exponent outside `i64`, the
`=0` exact plural branch was therefore skipped.

The formatter now validates the mantissa and exponent syntax, recognizes exact
zero before bounded exponent conversion, and preserves exact-branch precedence.

The focused regression
`exact_zero_branch_precedes_category_for_unrepresentable_exponent` passed
1/1 through `./scripts/cargo-headstart test --offline --locked -p
rustui-i18n --lib exact_zero_branch_precedes_category_for_unrepresentable_exponent`.
Scoped `git diff --check` passed. `rustfmt --check` reported two pre-existing
formatting differences elsewhere in the dirty source file; they were left
untouched.

Broader locale, runtime, RTL, and cross-platform acceptance remains open.
