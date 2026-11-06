# M773 — Stage 5 / align theme version parsing with the published schema

The published JSON Schema fixes `format_version` to numeric value `1` without
requiring an integer representation. Runtime deserialization now accepts
integral JSON number forms such as `1.0`, while still rejecting fractional and
out-of-range values.

The focused regression passed (1/1), and scoped `git diff --check` passed.
`rustfmt --check` reports two formatting differences in pre-existing dirty
test blocks in the same file; those unrelated edits were preserved. Broader
theme-schema and native appearance acceptance remain open.
