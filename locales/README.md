# Localization targets

Framework message catalogs use RustUI's initial Fluent-compatible subset under `en/`,
`fr/`, and `ar/`. The subset accepts comments, blank lines, and one-line
`message-id = value` entries. Runtime locale records declare direction and catalog format.

`python3 scripts/validate_locales.py` validates the metadata and checks that each
framework and Lab catalog group has the same message IDs. Interpolation, plurals, terms,
select expressions, and locale-sensitive date/number formatting are not implemented yet.
