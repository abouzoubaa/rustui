# M950 — Stage 6 / align Arabic exponent sign expectation

Updated the legacy Arabic scientific-notation assertion to expect the U+061C
Arabic Letter Mark before a signed exponent, matching the localized sign
behavior covered by M927. This is a test expectation repair; runtime formatting
did not change.

Focused validation: `./scripts/cargo-headstart test -p rustui-i18n --lib formats_decimal_arguments_with_locale_separators_and_digits --locked --offline`.
