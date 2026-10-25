# Stage 6 locale runtime implementation

The user approved Strategy A on 2026-10-25. RustUI owns the public locale and
catalog API; ICU4X and Project Fluent supply formatting and message evaluation.
The same Rust implementation targets macOS and Web/Wasm. GPUI still owns UI
state, scheduling, layout and rendering.

## Supported contract

- Existing `Catalog::parse`, borrowed lookup and legacy interpolation preserve
  their grammar and fallback behavior. New full FTL resources are explicit:
  `Catalog::parse_fluent`. Fluent messages, attributes, terms, references,
  multiline values, text selectors and cardinal/ordinal selectors use the
  established Fluent evaluator. `Catalogs::format` takes named arguments;
  `format_message` evaluates a message without arguments. Borrowed lookup never
  exposes unevaluated FTL. The active RustUI localization coordinator and
  `LocalizedText` route through this same catalog selection path.
- Default `NUMBER` formatting uses the resource language and a thread-safe ICU
  formatter cached by Fluent's memoizer. Only `NUMBER` type options `cardinal`
  and `ordinal` are accepted. Other options and functions, including `DATETIME`,
  return structured errors. Fluent is plain text; no HTML/DOM overlay runtime
  is introduced. Bidi isolation remains enabled.
- Fluent uses f64 numeric operands. Non-finite numbers and integer arguments
  outside the exact f64 integer range are rejected. Use preformatted strings
  from `LocaleFormatter` for larger exact decimals or full locale preferences.
- `LocaleFormatter` preserves decimal-string and i64 precision, supports
  short/medium/long civil dates, hour/minute or hour/minute/second times, and
  combined date/time. Locale preferences include numbering system (`u-nu`),
  calendar (`u-ca`) and hour cycle (`u-hc`); invalid supported preferences and
  civil fields return errors. Inputs are ISO civil fields and local wall-clock
  fields. No timestamp parsing, time-zone conversion, currency/percent/units,
  compact-number options or application-specific option maps are claimed.
- Catalog validation compares identifiers and referenced argument contracts,
  including attributes. Evaluation exposes missing variables/references/cycles.
  Legacy expression checks do not establish selector-structure equivalence
  between arbitrary FTL resources.

## Dependencies and data

Direct exact versions: Fluent Bundle 0.16.0, Fluent Syntax 0.12.0,
intl-memoizer 0.5.3, unic-langid 0.9.6, ICU locale/decimal/datetime/provider 2.4.0,
and writeable 0.6.4. ICU Provider `sync` enables Arc-backed payloads required by
Fluent's concurrent memoizer. Compiled ICU data is embedded on both targets.
All 29 newly needed source packages were extracted from archives whose SHA-256
matched the lockfile, with Cargo vendor checksum manifests generated from the
original bytes. Existing upstream GPUI revisions and license/checksum metadata
remain unchanged. Sources remain in `vendor/`.

ICU4X 2.4.0 formatting data reports CLDR 49 / Unicode 18. Fluent's locked
`intl_pluralrules` 7.0.2 generated rules declare CLDR 37, while its README says
CLDR 36 and packaged generator inputs are absent. This documented subsystem
split is accepted; selector tables are not claimed to match ICU CLDR 49 or to
have independently regenerated data provenance.

## Current evidence and remaining acceptance

Headstart locked/offline tests pass: 82 unit, 9 legacy catalog integration and
10 Fluent/ICU conformance tests. They cover term/reference/attribute evaluation,
text/cardinal/ordinal selectors, Russian categories, errors, fallback, argument
validation, exact decimals, preferences and civil date/time validation. Logs:
`/tmp/rustui-stage6-runtime-tests.log`. The runnable crate example passes 1/1 and public-API Playground localization
checks pass 3/3. A fresh isolated target with the unchanged approved compiler
configuration passes all 102 locale tests/doctest. Reused incremental state
triggered an ICU def_span fingerprint ICE; a per-command i18n-only incremental
disable also passes all 937 RustUI component regressions. Repository compiler
configuration and all shared caches are preserved.

This does not establish native/browser rendering, keyboard/IME/AX/VoiceOver,
locale switching and RTL visuals, cross-browser behavior, package size or
startup/frame performance. Licensing and distributed notices must include the
resolved new dependencies. Further ICU/Fluent upgrades require a reviewed data
and dependency update with conformance, target, size and license checks.

The 29 new packages all declare SPDX metadata. Upstream notice files are absent
from the fluent-langneg and intl_pluralrules archives; versioned verbatim
notice supplements must be captured before distribution.
