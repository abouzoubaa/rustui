# M1108 — Stage 6 Arabic zero plural with a large decimal exponent

## Contract

Plural selection for an exact decimal zero must preserve the supported locale's
zero category even when its exponent cannot be represented by the parser's
integer exponent type.

## Change

`decimal_integer_operands` now validates the mantissa and recognizes an
all-zero digit sequence before parsing the exponent. Thus
`0e999999999999999999999` resolves as Arabic zero rather than falling through
to `other` because the exponent is too large for `i64`.

## Evidence

`arabic_decimal_zero_plural_ignores_unrepresentable_exponent` passed with
`./scripts/cargo-headstart test -p rustui-i18n arabic_decimal_zero_plural_ignores_unrepresentable_exponent`
(1 passed). `git diff --check` passed for `formatting.rs`.

This is focused source-level evidence; broader localization and native
acceptance remain open.
