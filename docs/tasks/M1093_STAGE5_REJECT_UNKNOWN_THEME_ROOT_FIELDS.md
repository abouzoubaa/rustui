# M1093 — Stage 5: reject unknown root fields in theme schemas

The published theme JSON Schema now rejects unknown root properties, matching
`ThemeDocument`'s `deny_unknown_fields` deserialization contract. A focused
Rust regression verifies both the schema constraint and runtime rejection of
an unknown root field.

The focused locked/offline regression passed 1/1. Native visual and
accessibility theme acceptance remain separate.
