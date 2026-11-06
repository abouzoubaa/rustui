# M891 — Stage 5 / require SearchField theme renderer requirements

Theme validation now requires `search_field/list_search` rules for the light
and dark themes at small and large sizes, with the width, height, corner radius,
background, foreground, and typography properties consumed by the renderer.

The focused regression passed:

```sh
./scripts/cargo-headstart test -p rustui search_field_theme_requires_renderer_rules_and_properties --lib
```

This closes a theme-schema validation gap. Broader native visual, accessibility,
and cross-platform theme acceptance remain open.
