# M869 — Stage 5 Label renderer theme requirements

## Change

`ThemeDocument::validate` now checks the renderer-required `foreground` and
`typography` properties for Label style rules. This aligns accepted Label theme
documents with the values consumed by the Label renderer.

## Focused evidence

- The regression removes `foreground` and `typography` separately from a Label
  style and verifies validation rejects each case.
- `./scripts/cargo-headstart test -p rustui theme::schema::tests::label_theme_requires_renderer_foreground_and_typography --lib --locked --offline` passed: 1 passed, 0 failed, 776 filtered.
- `git diff --check -- crates/rustui/src/theme/schema.rs` passed.

## Acceptance boundary

This closes theme-input validation for these two Label properties. Native visual
and accessibility acceptance remain open.
