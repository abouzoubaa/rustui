# M1173 — Stage 7 basic component ID validation

## Scope

Validate explicit IDs on the Stage 7 basic/content components `Icon`, `Image`,
and `Badge`. `Identity` stores blank strings, so each public builder must reject
empty and whitespace-only IDs consistently with neighboring components.

## Change

- Reject blank IDs in `IconView::id`, `ImageView::id`, and `BadgeView::id`.
- Add focused regressions for both empty and whitespace-only values.
- Preserve nonblank ID text unchanged.

## Evidence

`./scripts/cargo-headstart test -p rustui --lib rejects_blank_explicit_identity`
passed: 16 matching tests, including all three new regressions and neighboring
component checks. This verifies builder validation only; it does not establish
native or browser accessibility-tree behavior.
