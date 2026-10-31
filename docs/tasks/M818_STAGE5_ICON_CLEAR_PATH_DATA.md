# M818 Stage 5: Icon clear artwork path data

The Icon clear-button renderer reads `vector_path.value.data` and requires a
string before constructing its SVG. Theme validation now rejects clear styles
with the path missing, malformed, or blank, preventing invalid theme data from
reaching that direct renderer lookup.

Focused regression:

```text
./scripts/cargo-headstart test -p rustui theme::schema::tests::icon_clear_theme_requires_renderer_path_data --lib --locked --offline
1 passed; 0 failed
```

Native visual rendering, accessibility, and cross-platform theme acceptance
remain open.
