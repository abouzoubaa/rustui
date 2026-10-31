# M1192 — Stage 9 compiled-preview SearchField query binding

## Scope

Wire the compiled-preview SearchField specimen's controlled text to the
descriptor schema's `query` property.

## Change

The production `text_binding()` mapping now resolves SearchField reads and
writes through `query`. The regression exercises that production binding,
including stamp validation and descriptor export/import round trips.

## Verification

Focused check passed:

```sh
./scripts/cargo-headstart test -p rustui-playground --lib stories::compiled_preview_gallery::tests::search_field_specimen_uses_stamp_checked_query_and_round_trips --locked --offline
```

Result: 1 passed. Broader inspector and native accessibility acceptance remain
open.
