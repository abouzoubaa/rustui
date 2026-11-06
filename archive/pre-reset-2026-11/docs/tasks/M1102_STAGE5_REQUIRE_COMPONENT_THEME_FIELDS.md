# M1102 — Stage 5: require component-theme runtime fields in the schema

The published `componentTheme` JSON Schema now requires `rustui_type` and
`parts` and constrains them to string and object types. Rust deserialization
requires both; `styles` remains optional because its Rust field has a default.
The schema previously accepted empty and styles-only records that runtime
deserialization rejects.

The focused schema-contract test passed. Native visual, accessibility, and
cross-platform theme acceptance remain open.
