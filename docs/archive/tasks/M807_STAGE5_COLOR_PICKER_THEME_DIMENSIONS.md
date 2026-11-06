# M807 — Stage 5 ColorPicker renderer dimensions

ColorPicker resolves its swatch style and directly indexes `width` and
`height`. Theme validation now rejects every ColorPicker style rule that omits
either renderer-required property.

The focused regression removes `height` from the `swatches` rule and confirms
validation reports the missing property. The focused schema test command
`./scripts/cargo-headstart test -p rustui theme_requires_renderer --lib --locked --offline`
exited successfully. Native visual and accessibility acceptance remain open.
