# Localization targets

Framework message catalogs use RustUI's initial Fluent-compatible subset under `en/`,
`fr/`, and `ar/`. The subset accepts comments, blank lines, and one-line
`message-id = value` entries. Runtime locale records declare direction and catalog format.

`python3 scripts/validate_locales.py` validates metadata, requires framework and
Playground catalog directories to match the runtime locale records, and checks
that each catalog group has the same message IDs. Named text, integer,
and decimal interpolation is available through `LocalizedText::with_arguments`;
runtime resolution uses the active locale and catalog fallback before formatting.
Missing interpolation arguments are surfaced as visible localization diagnostics.

The message subset supports nested plural/select expressions with bounded integer
and decimal cardinal rules for English, French, and Arabic, plus locale-aware
integer and decimal formatting for those languages. Integer formatting honors
`nu-latn` and `nu-arab` overrides. Cardinal and ordinal branches support
locale-formatted `#` and `\#` as a literal pound sign. Ordinal selection supports
exact integer branches, English `one`/`two`/`few`/`other`, French `one`/`other`,
and `other` for Arabic and other locales. Full CLDR data and locale-sensitive
date/time formatting remain unsupported.
This describes catalog/runtime capability; it does not establish native
locale-switching or RTL GUI acceptance, which require separate runtime checks.
