# M1101 — Stage 11: bind Wasm initialization to the imported function

The Web package check now requires the awaited Wasm initializer to use the
same identifier imported from the generated Playground module. Importing
`start` while calling `init()` previously passed separate checks and produced
a bundle that could not bootstrap.

The focused packaging harness passed, including rejection while preserving
the previous bundle and cleaning staging output. No Wasm build or browser
acceptance was run; Stage 11 browser interaction, accessibility, and
font/glyph acceptance remain open.
