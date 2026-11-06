# M817 Stage 5: Toggle renderer requirements

`ToggleView::into_element` indexes `background`, `width`, `height`, and
`corner_radius` from the resolved `toggle/body` theme style. Theme validation
now rejects a typed Toggle theme when any renderer-required property is absent,
instead of allowing the renderer to panic later.

Focused regression:

```text
./scripts/cargo-headstart test -p rustui theme::schema::tests::toggle_theme_requires_renderer_geometry --lib --locked --offline
1 passed; 0 failed
```

This closes the schema/runtime requirement mismatch. Native visual rendering,
accessibility, and cross-platform theme acceptance remain open.
