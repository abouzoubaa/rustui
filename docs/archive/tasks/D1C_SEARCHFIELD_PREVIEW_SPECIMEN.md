# D1c SearchField preview specimen

The compiled preview gallery includes an opt-in `search-field-specimen` using
the public RustUI `SearchField` API. Its descriptor exposes separate serializable
`placeholder` and controlled `query` strings. SearchField emits no preview event,
so the schema has no events or payloads. The factory reuses the gallery's
mount-scoped query binding, generation-specific component identity,
stamp-checked feedback, and descriptor codec. Editing the placeholder recreates
the retained input identity so its initial placeholder is applied. Its public
SearchField implementation selects the retained text-field search kind and
enables the native clear affordance when the query is nonempty. Its accessible
name stays fixed when the editable placeholder is empty or changed.

The focused test checks both schema defaults, absence of events, accepted query
feedback and revision advance, stale-stamp rejection, independent placeholder
feedback, and descriptor export/import round-trip.

Validation on 2026-10-20:

- `CARGO_TARGET_DIR=target/headstart/d1c-searchfield-specimen-validation ./scripts/cargo-headstart test --quiet -p rustui-playground --lib stories::compiled_preview_gallery::tests::search_field_specimen_uses_stamp_checked_query_and_round_trips --locked --offline` — passed, 1 test, 0 failed (2026-10-20; includes editable placeholder).
- `CARGO_TARGET_DIR=target/headstart/d1c-searchfield-specimen-validation ./scripts/cargo-headstart check --quiet -p rustui-playground --all-targets --locked --offline` — passed (exit code 0).
- Follow-up after TextEditor and editable-placeholder changes: the gallery-focused filter passed 12/12 tests, and an isolated locked/offline Playground all-targets check passed with the existing `table_binding` and catalog constant dead-code warnings.
- `rustfmt --check --edition 2024 apps/rustui-playground/src/stories/compiled_preview_gallery.rs` — passed.
- `git diff --check -- apps/rustui-playground/src/stories/compiled_preview_gallery.rs` — passed.

### Bounded native check (2026-10-21)

Built and packaged the Playground, then exercised the canonical **Text inputs**
story. Typing `clear probe 42` updated the SearchField and its owner value and
exposed a **Clear Search** button in the accessibility tree. Activating it emptied
the field and owner value, removed the button, and left the SearchField focused
with its caret visible. After typing again, the clear button returned; pressing
Tab moved focus to **Notes** while the clear button remained visible, confirming
it is skipped in tab traversal.

The native check did not cover a custom or French clear-button name. The existing
story configures `SearchField("Search", ...)` without
`.clear_button_accessibility_label(...)` and has no locale selector; the compiled
preview specimen exposes only placeholder and query. Observing the custom label
in native accessibility output would need a separately configured specimen.
Cross-platform rendering also remains unverified.

Evidence: [build log](../../target/headstart/evidence/searchfield-clear-build-20261021.log),
[package log](../../target/headstart/evidence/searchfield-clear-package-20261021.log),
and [native acceptance record](../../target/headstart/evidence/searchfield-clear-native-acceptance-20261021.json).
