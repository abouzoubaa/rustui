# M884 — Stage 9 ColorPicker inspector properties

## Change

The compiled-preview ColorPicker inspector now exposes all six editable schema
properties (`alpha`, `has_color`, `hue`, `label`, `lightness`, and
`saturation`). The `has_color` binding uses the existing color feedback path
without mutating the gallery's unrelated `enabled` state.

## Focused evidence

- `./scripts/cargo-headstart test -p rustui-playground --lib stories::compiled_preview_gallery::tests::color_picker_inspector_exposes_each_editable_property --locked --offline` passed (1 test).
- Scoped `git diff --check` passed. Existing unrelated formatting differences in the dirty gallery file were preserved.

## Acceptance boundary

This closes a local inspector/schema mismatch. Native inspector behavior and
broader UI Studio host, transport, and serialization acceptance remain open.
