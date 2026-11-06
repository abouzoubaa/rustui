# M593 — Stage 6 plural pound substitution

The RustUI-owned message formatter expands `#` to the numeric value of the innermost selected plural expression. It formats that value through the same locale-aware integer or decimal formatter used by ordinary named arguments. `interpolate` retains invariant number formatting; `interpolate_with_locale` uses its supplied locale.

A nested `select` inside a plural branch inherits that plural value. A nested plural expression shadows the outer value within its own branches. `#` outside a plural branch remains literal. Within plural context, `\#` emits a literal pound sign.

This is a bounded addition to the existing message subset. It does not add Fluent or ICU dependencies, ordinal rules, full CLDR plural or number data, date or currency formatting, or new catalog syntax beyond the existing plural expressions.

The Playground English, French, and Arabic integer and decimal plural messages exercise the substitution. `scripts/validate_locales.py` continues to validate the checked-in locale records and catalog group.

## Validation

- Focused locked/offline `rustui-i18n` pound substitution tests passed 4/4: `target/headstart/evidence/m593-plural-pound-focused-tests-20261021.log`.
- Locked/offline `rustui-i18n` suite passed 44 unit tests and 7 catalog contract tests: `target/headstart/evidence/m593-rustui-i18n-tests-20261021.log`.
- Focused Playground decimal plural catalog/rendering test passed 1/1: `target/headstart/evidence/m593-playground-plural-pound-test-20261021.log`.
- Locked/offline all-targets check for `rustui-i18n` and `rustui-playground` passed: `target/headstart/evidence/m593-workspace-check-20261021.log`.
- `python3 scripts/validate_locales.py` passed, validating 3 runtime locale records and catalog groups.
- `rustfmt --edition 2024 --check crates/rustui-i18n/src/formatting.rs` passed.

Native locale switching, broader RTL visual behavior, VoiceOver, cross-platform behavior, full CLDR, ordinal plurals, and date/time formatting remain separate open limits.

## M619 — Unicode numbering-system override regression

The cardinal plural `#` regression also covers `ar-EG-u-nu-latn` and
`en-US-u-nu-arab`, confirming the active formatter keeps the requested digit
system. The focused test passed (1/1), and scoped rustfmt passed. This verifies
existing formatter plumbing without expanding catalog syntax or locale data.

## Native Playground locale verification

Launched the disposable Playground bundle `target/headstart/verification/M56 Review.app` with bundle ID `dev.rustui.playground.validation`. The Text framework demo rendered the decimal plural as `1.5 notes` in English, `1,5 note` in French, and `١٫٥ ملاحظة` in Arabic. Locale changes were observed in the macOS accessibility tree. The app was quit and the disposable bundle removed; no standalone test executable was created. This verifies the bounded localized output, not broader RTL layout, VoiceOver, or platform coverage. Evidence: `target/headstart/evidence/m593-native-locale-verification-20261021.json`.
