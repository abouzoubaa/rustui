# M1119 — Stage 3 / search Playground coverage in the active locale

The Playground displayed localized coverage suffixes, but its catalog filter
searched only the English coverage label. French and Arabic status text visible
to users could not find matching catalog rows.

Filtering now searches the coverage messages resolved from the existing
catalogs for the available locales and unions those matches with the existing
key, API-name, and English status matches. The catalog list and its selection
binding use the same filtered row set; translations are not duplicated in the
filter implementation.

The focused regression `french_coverage_label_filters_and_selects_catalog_row`
proves that `Partiel` matches and selects the Button row while the previous
English-only filter had no match. The locked/offline focused Playground test
passed (1 passed, 0 failed; 173 filtered out).

Browser filtering, keyboard, and accessibility acceptance remain open.
