# M728 — Stage 6 scientific exponent digit localization

## Contract

The bounded decimal formatter localizes digits for Arabic, including decimal
arguments rendered in scientific notation. This extends the existing
locale-aware decimal formatting contract in
`docs/tasks/I1B_PARAMETERIZED_CATALOG_CONTRACT.md`; it does not add new locale
families or claim full CLDR number formatting.

## Change

`format_decimal` now applies the selected numbering system to exponent digits
as well as mantissa digits. Arabic `1e+21` renders as `١e+٢١`; the exponent
marker and sign are preserved.

## Verification

- Focused locked/offline `rustui-i18n` formatter regression passed.
- Scoped rustfmt check and `git diff --check` passed.

Full CLDR, native locale switching, RTL GUI, and cross-platform acceptance
remain open.
