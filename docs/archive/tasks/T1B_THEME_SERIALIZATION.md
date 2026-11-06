# T1B: Validated theme JSON export

## Contract

`theme::serialize_theme(&ThemeDocument)` validates the current format-v1
document and returns pretty-printed JSON for the typed document. This is a
normalized export, not a byte-preserving copy of the input. Whitespace, object
ordering, numeric spelling, and absent optional fields may be normalized.

`source`, `token_sources`, and `derivation` are object-valued metadata. Every
component entry is an object. Unknown component objects without typed `parts`
or `styles` fields remain opaque JSON and round-trip unchanged. Typed component
records are semantically validated through the shared theme resolver, including
their selectors, declared parts, and recognized style properties; custom
metadata and unknown property payloads remain preserved. Button `parts` values
also remain JSON. Unknown fields in strict typed records are rejected during
parsing rather than discarded. The format does not use custom serde
defaults/skip rules; `max_version: None` serializes as `null`.

This API exports theme documents only. It does not serialize RustUI view trees,
component state, runtime caches, or GPUI entities. It also does not write files
or watch for external edits.

## Verification

Focused tests cover typed-document round trip, preservation of opaque component
objects, invalid-document rejection, and rejection of unknown typed fields.
The root task also ran the focused loader and resolver suites under the
persistent Headstart profile; native visual acceptance remains separate.

## Playground consumer acceptance (M27)

The Theme foundation page includes a read-only inspector that loads macos27.json,
exports normalized JSON, previews its first 18 lines, and resolves Light/Dark
Button metrics. Native inspection showed profile macos27 / format 1 / 41 records
and Button 67 by 24 px, font 13 px, padding 8 px, radius 6 px for both modes.
Reload reads the file without writing or activating it. Evidence:
`target/headstart/milestone27-theme-inspector-gui.json`. Final integration tests
passed 83 across 12 binaries (`target/headstart/milestone27-28-tests-final.log`).
