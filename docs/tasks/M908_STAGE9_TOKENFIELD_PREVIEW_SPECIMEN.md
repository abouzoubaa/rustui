# M908 — Stage 9 / TokenField compiled-preview specimen

## Change

The canonical D1c compiled-preview gallery now registers the public
`TokenField` API as `token-field-specimen`. The specimen exposes editable
`placeholder`, `draft`, and `tokens_json` properties. Because preview
descriptor values are scalar, `tokens_json` is a bounded JSON string array
decoded by the compiled-local factory into TokenField's controlled
`Vec<String>` binding.

Token and draft feedback is stamp-checked. Descriptor export/import preserves
the placeholder, draft, and token order, including duplicates. Invalid token
JSON is rejected on inspector edits and before import remounts the preview.

## Validation recorded

The focused regression
`token_field_specimen_round_trips_controlled_tokens_and_rejects_invalid_lists`
passed 1/1 with the locked, offline command:

```sh
./scripts/cargo-headstart test -p rustui-playground --lib stories::compiled_preview_gallery::tests::token_field_specimen_round_trips_controlled_tokens_and_rejects_invalid_lists --locked --offline
```

This is source-level coverage only. Native rendering and interaction,
accessibility, locale, theme, RTL, and cross-platform acceptance remain open.
The catalog coverage count is 53 implemented public component APIs and 74
registered specimens including supplemental layouts and local fixtures; see
`D1C_COMPONENT_CATALOG_COVERAGE.md`.
