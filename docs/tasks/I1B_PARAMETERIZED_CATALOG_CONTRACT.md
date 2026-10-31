# I1B parameterized catalog contract

## M386 — Validate interpolation arguments across locales

`validate_catalogs` now compares the named placeholders in every shared
message against the configured fallback translation. A locale with renamed,
missing, or added argument names is reported in `argument_mismatches`; a
malformed or unmatched brace pattern is reported in `invalid_patterns`. Both
reports make `CatalogValidation::is_valid()` return false. Escaped braces do
not count as arguments, and argument order does not matter.

The check covers RustUI's existing `{name}` interpolation subset only. It does
not add Fluent terms/select expressions, plural rules, or locale-sensitive
number/date formatting. No dependency or vendored source changed.

The locked/offline `rustui-i18n` package suite passed 18/18 (17 library tests
and one catalog contract test), including cases for renamed/missing names,
malformed patterns, escaped braces, and reordered arguments. Evidence:
`target/headstart/evidence/localization-argument-parity-m386-20261021.log`.
The locked/offline RustUI all-targets check also exited 0; it reported existing
unused-variable/assignment warnings in two benchmark examples and the existing
future-incompatibility notice for `block 0.1.6`. Evidence:
`target/headstart/evidence/localization-argument-parity-rustui-check-m386-20261021.log`.

This is catalog validation evidence; runtime locale switching, plural/select
semantics, formatting, native visual behavior, and screen-reader acceptance
remain open.

## Checked-in framework catalog regression

The catalog contract integration test now parses the checked-in English,
French, and Arabic framework catalogs and runs `validate_catalogs` with English
as the fallback. This keeps the `{name}` placeholder parity contract connected
to the actual locale files; the locked/offline integration test passed 2/2.
Evidence: `target/headstart/evidence/framework-catalog-interpolation-contract.log`.
This source-level check does not establish runtime switching or native RTL
layout acceptance.


## M453 — Validate the shipped framework catalog interpolation contract

The integration suite now parses the checked-in English, French, and Arabic framework catalogs and runs the existing supported named-placeholder parity validator with English fallback. The locked/offline contract suite passed 2/2. This verifies catalog parity for the existing interpolation subset; it does not add Fluent, plural/select, or locale-sensitive number/date semantics and does not prove runtime switching or native RTL rendering. Evidence: `target/headstart/evidence/framework-catalog-interpolation-contract.log`.

## M463 — Format interpolated integers for the active locale

`interpolate_with_locale` and `format_integer` add bounded grouping for English
(comma), French (narrow no-break space), and Arabic (Arabic thousands separator
and Arabic-Indic digits). `ActiveLocalization::message_with_arguments` now uses
the active locale. The original `interpolate` remains locale-invariant; other
locale families also retain ASCII decimal digits without grouping. Plural
selection, decimal/fraction formatting, and date formatting remain open.

The locked/offline `rustui-i18n` suite passed 23 unit tests and 2 catalog
contract tests. The locked/offline RustUI library check passed and focused
localization regressions passed 5/5, including active-locale integer formatting.
Evidence: `target/headstart/evidence/locale-integer-formatting-m463-20261021.log`,
`target/headstart/evidence/locale-integer-formatting-rustui-check-m463-20261021.log`,
and `target/headstart/evidence/locale-integer-formatting-rustui-tests-m463-20261021.log`.

### M490 follow-up — Arabic negative sign directionality

Negative Arabic integers now emit `U+061C` before the minus sign, preserving
the expected sign order with Arabic-Indic digits. The focused `ar-EG` regression
is in `formats_integers_for_supported_locale_families`; validation evidence is
recorded in `target/headstart/evidence/m490-arabic-negative-integer-sign-test-20261021.log`.
The focused locked/offline test passed 1/1; formatting and scoped diff checks
passed.

### M550 Playground expression-contract validation

The Playground catalog assertion now checks `expression_mismatches` in addition to message IDs and arguments. The allowlist for the intentionally malformed demo pattern remains limited to that pattern. Focused locked/offline locale-selector tests passed 2/2. Evidence: `target/headstart/evidence/m550-playground-catalog-expression-validation-20261021.log`. Runtime locale switching and native RTL behavior remain separate acceptance gates.

## M566 — Playground plural/select specimen
The localization gallery now renders `lab-i18n-plural` and `lab-i18n-select` through `LocalizedText::with_arguments`, with matching English, French, and Arabic catalog patterns. The Playground catalog contract requires both IDs and rejects cross-catalog argument/expression drift while allowing only the intentional malformed-message specimen. Focused catalog tests passed 2/2; `scripts/validate_locales.py` and scoped Rust formatting passed. Native locale switching, plural-rule output, and RTL visual acceptance remain open. Evidence: `target/headstart/evidence/m566-playground-locale-selector-tests-20261021.log`, `target/headstart/evidence/m566-locales-validation-20261021.log`, `target/headstart/evidence/m566-localization-rustfmt-20261021.log`.
## M570 — Locale-aware decimal interpolation

`MessageArgument` now accepts `f32` and `f64` as decimal arguments. `interpolate_with_locale` groups finite values and applies decimal separators for English, French, and Arabic; Arabic digit selection honors the existing `nu-latn` and `nu-arab` overrides. Float conversions use Rust's shortest round-trip decimal text without rounding; plain `interpolate` remains locale-invariant. Non-finite or invalid decimal text passes through unchanged, and scientific exponent text remains ASCII. M571 extends decimal arguments to bounded English, French, and Arabic cardinal plural selection; ordinal rules remain open. This does not add currency formatting or full CLDR number rules. Locked/offline `rustui-i18n` suite passed 38 unit tests and 5 catalog-contract tests; RustUI active-locale decimal regression passed. Evidence: `target/headstart/evidence/m570-locale-decimal-formatting-rustui-i18n-tests-20261021.log`, `target/headstart/evidence/m570-locale-decimal-formatting-rustui-tests-20261021.log`; implementation: `crates/rustui-i18n/src/formatting.rs`.

## M572 — Demonstrate decimal plural formatting in the Playground

The localization gallery passes `1.5_f64` through `LocalizedText::with_arguments`
using a plural message shared by the English, French, and Arabic catalogs. The
message interpolates the decimal in its selected branch, demonstrating the
bounded fractional categories and locale-aware number formatting together.
The focused Playground regression checks catalog parity and the rendered
strings for all three locales. This sample does not expand ordinal plural,
`#` substitution, or full CLDR support; native locale-switching and RTL visual
acceptance remain separate.

The locked/offline focused Playground test passed 1/1; scoped Rust formatting
and `git diff --check` passed. Evidence: `target/headstart/evidence/m572-playground-decimal-plural-contract-test-20261021.log`.


## M576 — README capability alignment

`locales/README.md` now matches M571’s bounded decimal interpolation/plural support and locale-aware integer/decimal formatting for English, French, and Arabic. Full CLDR and date/time formatting remain unsupported; native locale-switching and RTL GUI acceptance remain open. `python3 scripts/validate_locales.py` and the scoped documentation diff check passed. Whole-worktree `git diff --check` reported a trailing blank line at `crates/rustui/src/runtime/component_host.rs:706`, outside this documentation slice. Evidence: `target/headstart/evidence/m576-locales-readme-capability-alignment-20261021.log`.

### M593 follow-up — plural branch substitution

M593 adds bounded `#` substitution within existing plural expressions and updates the Playground English, French, and Arabic examples. This does not change the M386 named-interpolation contract or add Fluent/ICU dependencies; exact limits and validation are in `docs/tasks/M593_STAGE6_PLURAL_POUND_SUBSTITUTION.md`.
