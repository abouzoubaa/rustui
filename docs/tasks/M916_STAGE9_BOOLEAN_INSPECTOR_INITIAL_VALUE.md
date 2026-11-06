# M916 — Stage 9 / initialize Boolean inspector rows from mounted properties

Ordinary Boolean preview inspector rows initialized their binding from the
gallery-wide `enabled` state, regardless of the property being inspected. For
example, Badge's editable `dot` property defaults to `false` while shared
`enabled` defaults to `true`; the binding's initial value therefore disagreed
with its mounted-property reader. Ordinary Boolean rows now initialize through
that reader. ColorPicker's `has_color` row keeps its specialized feedback
binding.

The focused regression
`boolean_inspector_reads_the_mounted_property` passed 1/1 with:

```text
./scripts/cargo-headstart test -p rustui-playground --lib stories::compiled_preview_gallery::tests::boolean_inspector_reads_the_mounted_property --locked --offline
```

This is source-level inspector coverage only. Native rendering, interaction,
accessibility, locale, theme, RTL, and cross-platform acceptance remain open.
