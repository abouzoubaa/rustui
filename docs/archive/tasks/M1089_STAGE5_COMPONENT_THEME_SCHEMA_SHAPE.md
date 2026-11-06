# M1089 — Stage 5 component theme schema shape

The published `componentTheme` JSON Schema now requires component style rules
to include `part`, all eight nonempty selector fields, and `style.properties`.
It constrains `appearance` to `any`, `light`, or `dark`, and rejects unknown
selector fields, matching the Rust deserializer's structural contract.

Focused regression:
`./scripts/cargo-headstart test -p rustui --lib component_theme_schema_restricts_appearance_selectors_for_known_components --locked --offline`
passed 1/1. `python3 -m json.tool themes/theme.schema.json` and scoped
`git diff --check` passed. This aligns the published schema only; native visual
and accessibility acceptance remain open.
