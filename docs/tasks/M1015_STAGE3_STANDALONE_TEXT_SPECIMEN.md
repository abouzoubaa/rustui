# M1015 — Stage 3: add the standalone Text specimen

The Playground's `text` catalog route now renders a standalone public RustUI
Text specimen with heading, body, and wrapping examples. `text_style_guide`
continues to render the reference-size and regular/bold typography gallery,
while the framework `StoryId::Text` route remains the catalog-message
localization gallery.

Focused Playground checks passed: `text_catalog` 2/2, `text_specimen` 4/4, and
`text_style_guide` 3/3. Formatting and scoped whitespace checks passed.

Native visual, keyboard, and accessibility acceptance remain open.
