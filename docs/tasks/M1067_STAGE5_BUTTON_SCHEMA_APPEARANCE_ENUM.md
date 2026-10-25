# M1067 — Stage 5 / align Button appearance schema with runtime

The published theme JSON schema now restricts Button appearance selectors to
`any`, `light`, or `dark`, matching runtime validation. The focused regression
passed 1/1 and checks the published enum as well as rejection of `system`.
JSON parsing and scoped whitespace checks passed. Native visual,
accessibility, and cross-platform theme acceptance remain open.
