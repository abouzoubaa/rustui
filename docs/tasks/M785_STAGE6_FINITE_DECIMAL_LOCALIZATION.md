# M785 — Localize finite decimal spellings beyond f64 range

## Scope

`format_decimal` previously treated values that parsed to non-finite `f64` as non-decimal input. Large but syntactically finite decimal strings such as `1e400` therefore bypassed locale formatting.

## Change

The formatter now validates decimal spelling lexically. It can localize separators and digits without converting the value to a bounded floating-point representation; non-number spellings such as `NaN` remain unchanged.

## Evidence

`./scripts/cargo-headstart test --locked --offline -p rustui-i18n formats_finite_decimal_strings_outside_f64_range` passed (1 test).

This does not implement full CLDR/ICU behavior or close broader locale, RTL, accessibility, or cross-platform acceptance.
