# M464 — Stage 6 plural and select messages

`rustui-i18n` parses each formatted message into an internal tree. Catalog
validation uses the same parser to compare message argument and expression
contracts. Supported expressions are:

```text
{name}
{name, select, key {message} other {message}}
{count, plural, category {message} =integer {message} other {message}}
```

Select keys contain ASCII letters, digits, `_`, or `-`. Plural categories are
`zero`, `one`, `two`, `few`, `many`, and `other`; exact signed `i64` branches
take precedence over locale categories. Every expression requires one `other`
branch. Duplicate keys, unknown plural categories, malformed expressions,
missing arguments, and wrong argument types return typed errors.

Expression bodies support placeholders, nested expressions, doubled opening braces, and `\}` as a literal closing-brace escape. Outside expression bodies, doubled braces escape literal braces.
Integer cardinal rules use the primary language: English selects `one` for
absolute value 1; French selects `one` for absolute values 0 and 1; Arabic uses
the integer `zero`, `one`, `two`, `few`, and `many` categories. Decimal
cardinal operands now use a bounded subset: English selects `one` only for 1
without fractional digits; French selects `one` when the absolute integer part
is 0 or 1; Arabic selects integer categories only, with non-integer values
falling through to `other`. Decimal values with an integral value still match
the existing exact integer branches. Other languages use `other`. At M464, ordinal
plurals, `#` substitution, and full CLDR data remained out of scope. M598 later adds bounded `selectordinal` support for the shipped English, French, and Arabic locale families; full CLDR ordinal data remains open. See `docs/tasks/M598_STAGE6_ORDINAL_SELECT.md`.

Catalog validation compares argument names, expression kinds, selector names,
select keys, and exact-number plural keys against the fallback catalog.
Locale-specific plural category branches may differ. Malformed patterns and
contract mismatches are reported separately.

Locale-aware integer formatting honors Unicode `nu-latn` and `nu-arab`
overrides for English, French, and Arabic while preserving the locale grouping
separator. Extension parsing stops at the next singleton subtag, so a `nu`
sequence in private-use subtags is ignored. Other numbering systems retain the
locale's existing default digit set.

Validation on 2026-10-21 (M464):

- Locked/offline `rustui-i18n`: 29 unit tests and 2 catalog contract tests passed: `target/headstart/evidence/stage6-plural-select-m464-final-rustui-i18n.log`.
- RustUI active-localization regression: 1 passed: `target/headstart/evidence/stage6-plural-select-m464-final-rustui-localization.log`.
- `cargo-headstart fmt --package rustui-i18n --package rustui -- --check` passed: `target/headstart/evidence/stage6-plural-select-m464-final-format.log`.
- No vendored dependency source changed.

Stage 6 closing-brace escape follow-up:

- Added a regression for `\}` inside a plural branch; focused test passed 1/1.
- Locked/offline `rustui-i18n` library suite passed 35/35.
- Evidence: `target/headstart/evidence/stage6-expression-closing-brace-escape-tests.log`.
## M571 — Decimal cardinal operands

Plural expressions accept decimal arguments and choose the bounded English,
French, and Arabic cardinal categories above; exact signed integer branches
retain precedence. Non-finite or unparsable decimal text selects `other`.
Locked/offline `rustui-i18n` tests passed 40 unit tests and 5 catalog-contract
tests. RustUI active-localization decimal plural integration is covered in the
RustUI library suite. See `target/headstart/evidence/m571-decimal-plural-rules-rustui-i18n-tests-20261021.log` and
`target/headstart/evidence/m571-decimal-plural-rules-rustui-tests-20261021.log`.

## Follow-up — M593

The `#` substitution limit above describes M464's bounded delivery. M593 adds a separate formatter feature that expands `#` inside plural branches using the active numeric value and existing locale formatter; see `docs/tasks/M593_STAGE6_PLURAL_POUND_SUBSTITUTION.md`. M593 did not add ordinal support. M598 later adds bounded `selectordinal` support; full CLDR ordinal data remains open. See `docs/tasks/M598_STAGE6_ORDINAL_SELECT.md`.
