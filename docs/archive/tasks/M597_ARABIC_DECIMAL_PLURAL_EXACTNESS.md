# M597 — exact Arabic decimal plural operands

Arabic decimal plural selection now derives integrality and the last-two-digit
operand from the decimal string, avoiding `f64` rounding for large exact values.
Exponent notation is handled by the same string-based calculation. English and
French selection paths are unchanged.

Regressions verify `10000000000000000000003` selects `few`,
`10000000000000000000011` selects `many`, an integral `.0` representation
selects `few`, and a fractional `.5` representation selects `other`.

The focused exactness regression passed 1/1, the `rustui-i18n` package passed
45 unit and 7 catalog-contract tests, and the locked/offline workspace
all-targets check passed. Full CLDR and ordinal support, VoiceOver, broader RTL,
and cross-platform acceptance remain open. Evidence:
`target/headstart/evidence/m597-arabic-decimal-plural-exactness-focused-20261021.log`,
`target/headstart/evidence/m597-rustui-i18n-suite-20261021.log`, and
`target/headstart/evidence/m596-m597-workspace-all-targets-check-20261021.log`.

## M701 — Preserve exact Arabic plural operands beyond `f64` range

Arabic decimal plural selection now uses the exact decimal-string operands even
when the value is outside the finite `f64` range. English and French retain
their existing finite-`f64` selection rules. A focused regression verifies an
exact Arabic integer ending in `03` selects `few` even though parsing it as
`f64` overflows. The locked/offline Headstart regression passed 1/1; evidence:
`target/headstart/evidence/m701-arabic-exact-decimal-range.log`. This extends
M597's exact-operand guarantee only; full CLDR and native locale, RTL, and
cross-platform acceptance remain open.
