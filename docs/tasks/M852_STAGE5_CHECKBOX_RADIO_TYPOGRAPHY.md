# M852 — Stage 5 Checkbox and Radio typography requirement

Checkbox and Radio label renderers consume the resolved `typography` property
when present. Their theme validation previously allowed it to be absent,
leaving those labels on GPUI's default font styling despite having renderer
theme records.

Theme validation now requires `typography` for both components. The regression
removes the property from each component and confirms validation rejects the
theme.

Focused evidence:

```text
./scripts/cargo-headstart test -p rustui --lib theme::schema::tests::checkbox_and_radio_themes_require_renderer_typography --locked --offline
1 passed; 773 filtered out
```

This closes the schema/runtime contract gap only. Native appearance,
accessibility, RTL, and cross-platform acceptance remain open.
