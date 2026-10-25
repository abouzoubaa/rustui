# Property Inspector accessible row labels

`PropertyInspector` derives each editor's accessible name from its visible row
label. Whitespace-only labels therefore create unnamed controls. Construction
now rejects a blank row label with `PropertyInspectorError::EmptyRowLabel`,
including the row index, alongside existing inspector and row-ID validation.

## Evidence — M237 (2026-10-20)

- Focused regression: `components::advanced::property_inspector::tests::rejects_blank_accessible_row_labels` passed 1/1.
- Command: `./scripts/cargo-headstart test -p rustui --lib components::advanced::property_inspector::tests::rejects_blank_accessible_row_labels --locked --offline`.
- Native accessibility-tree and screen-reader acceptance were not run.
# M410 — refresh retained PropertyInspector editor binding
The retained `IntegerValue` editor now dispatches input changes through the latest owner writer after inspector rerenders. Focused PropertyInspector tests passed 7/7, including the latest-writer regression. Native editing, cursor/undo, row removal, and full keyboard/screen-reader acceptance remain open. Evidence: `target/headstart/property-inspector-latest-writer-tests.log`.
