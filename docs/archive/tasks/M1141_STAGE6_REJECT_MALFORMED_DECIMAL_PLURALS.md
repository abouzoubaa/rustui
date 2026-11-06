# M1141 — Stage 6: reject malformed decimals for plural selection

Plural category selection now validates decimal arguments against the existing
decimal-literal grammar before reading integer operands. Malformed values no
longer select French `one` or Arabic `zero`; they fall back to `other`.

Focused regression: `malformed_decimal_arguments_fall_back_to_other_plural_category`
passed 1/1 with
`./scripts/cargo-headstart test -p rustui-i18n malformed_decimal_arguments_fall_back_to_other_plural_category`.

This does not implement full CLDR rules and does not close native localization
or screen-reader acceptance.
