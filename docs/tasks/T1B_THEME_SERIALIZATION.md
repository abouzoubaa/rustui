# T1B: Validated theme JSON export

## Contract

`theme::serialize_theme(&ThemeDocument)` validates the current format-v1
document and returns pretty-printed JSON for the typed document. This is a
normalized export, not a byte-preserving copy of the input. Whitespace, object
ordering, numeric spelling, and absent optional fields may be normalized.

Arbitrary JSON stored in `source`, `token_sources`, `components`,
`derivation`, and Button `parts` remains represented as JSON values. Unknown
fields in typed records are rejected during parsing rather than discarded.
The current validator checks document invariants, Button records, and the
platform snapshot; other component records are preserved but not semantically
validated. The format does not use flattening or custom serde defaults/skip
rules; `max_version: None` serializes as `null`.

This API exports theme documents only. It does not serialize RustUI view trees,
component state, runtime caches, or GPUI entities. It also does not write files
or watch for external edits.

## Verification

Focused tests cover typed-document round trip, preservation of arbitrary JSON
records, invalid-document rejection, and rejection of unknown typed fields.
No Cargo command was run for this slice; integration validation remains with the
root task.

## Playground consumer acceptance (M27)

The Theme foundation page includes a read-only inspector that loads macos27.json,
exports normalized JSON, previews its first 18 lines, and resolves Light/Dark
Button metrics. Native inspection showed profile macos27 / format 1 / 41 records
and Button 67 by 24 px, font 13 px, padding 8 px, radius 6 px for both modes.
Reload reads the file without writing or activating it. Evidence:
`target/headstart/milestone27-theme-inspector-gui.json`. Final integration tests
passed 83 across 12 binaries (`target/headstart/milestone27-28-tests-final.log`).
