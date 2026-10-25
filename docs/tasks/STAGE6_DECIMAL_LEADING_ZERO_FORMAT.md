# Stage 6 decimal formatting: leading zeroes

`format_decimal` now removes redundant leading zeroes from a finite decimal's
integer part before applying locale grouping. It keeps one zero when the
integer part is zero, preserves the sign, and retains the fractional digits.
For example, `0001234.50` formats as `1,234.50` in English and `-0000.50`
formats as `-0,50` in French.

Focused validation passed:

```text
./scripts/cargo-headstart test -p rustui-i18n --lib --locked --offline formats_decimal_integer_part_without_grouping_leading_zeroes
1 passed; 0 failed; 50 filtered out
```

This covers formatter output for these two cases only. Full CLDR behavior,
other locale families, date/time and currency formatting, and native locale,
RTL, accessibility, and cross-platform acceptance remain open.
