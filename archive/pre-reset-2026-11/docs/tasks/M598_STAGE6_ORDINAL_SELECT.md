# M598 — bounded Stage 6 ordinal selection

## Contract

Support `{count, selectordinal, ...}` alongside the existing message
expressions. Preserve the expression kind in catalog contracts; accept the
English ordinal branches `one`, `two`, `few`, and `other`; retain exact
`=integer` branch precedence; and allow `#` formatting and escaping inside
ordinal branches.

The implemented locale rules are intentionally bounded to the shipped catalogs:
English uses the CLDR integer ordinal suffix categories, French selects `one`
for 1 and `other` otherwise, and Arabic and other locales select `other`.
Integral decimal strings, including exponent notation, use exact operands;
fractional decimals select `other` unless an exact integer branch applies.

## Limits

This does not implement full CLDR ordinal/cardinal data, other locales' ordinal
rules, a new Playground sample, VoiceOver, RTL layout, or cross-platform
acceptance. Existing catalog syntax remains unchanged apart from accepting
`selectordinal` expressions.

## Verification

- Three focused ordinal regressions passed, including English 11–13 exceptions,
  French and Arabic fallbacks, integral decimals/exponents, exact-branch
  precedence, pound substitution, and catalog-contract distinction.
- The locked/offline `rustui-i18n` suite passed 48 unit tests and 7
  catalog-contract tests.
- `rustfmt --edition 2024 --check crates/rustui-i18n/src/formatting.rs` passed.

Evidence: `target/headstart/evidence/m598-selectordinal-focused-20261021.log`
and `target/headstart/evidence/m598-rustui-i18n-suite-20261021.log`.

## M613 — Verify escaped pound in ordinal branches

Added a regression for ordinal `\#` literal output alongside `#` substitution
and a pound outside the branch. The implementation already shares pound-token
handling across cardinal and ordinal branches; this milestone closes the
contract-test gap without changing runtime behavior. `locales/README.md` now
documents the shipped ordinal subset: exact integer branches, English
`one`/`two`/`few`/`other`, French `one`/`other`, and `other` for Arabic and
other locales. Full CLDR remains unsupported.

Validation: locked/offline `rustui-i18n` passed 49 unit and 8
catalog-contract tests; locale validation and scoped rustfmt passed. Evidence:
`target/headstart/evidence/m613-stage6-ordinal-pound-suite-20261021.log` and
`target/headstart/evidence/m613-stage6-locale-doc-validation-20261021.log`.

## M614 — Verify ordinal pound locale formatting

Added a regression that formats an ordinal branch's active `#` value using the selected locale: English `1,234`, French `1 234`, and Arabic `١٬٢٣٤`. This confirms ordinal substitution uses the same locale-aware number formatter as cardinal plural substitution without expanding the supported ordinal-rule set. The locked/offline `rustui-i18n` suite passed 50 unit tests and 8 catalog-contract tests; locale validation, scoped rustfmt, and scoped diff checks passed. Full CLDR and native locale/RTL acceptance remain open. Evidence: `target/headstart/evidence/m614-stage6-ordinal-locale-test-20261021.log`, `target/headstart/evidence/m614-stage6-locale-validation-20261021.log`, and `target/headstart/evidence/m614-stage6-rustfmt-20261021.log`.
