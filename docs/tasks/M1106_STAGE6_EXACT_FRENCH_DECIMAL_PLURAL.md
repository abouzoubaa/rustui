# M1106 — Stage 6: preserve exact decimal operands for French plural rules

`plural_category_decimal` now checks the decimal's exact integer digits for
French `one` selection. Converting an arbitrary-precision decimal just below
two to `f64` rounded it to `2.0` and selected `other` incorrectly.

The focused regression reproduced that failure before the fix and passed
afterward: `./scripts/cargo-headstart test -p rustui-i18n french_decimal_plural_uses_exact_integer_part_near_two`
(1 passed). No ICU/Fluent dependency or locale coverage expansion was made.
