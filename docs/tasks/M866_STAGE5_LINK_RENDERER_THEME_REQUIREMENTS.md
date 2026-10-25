# M866 — Stage 5 / require Link renderer theme fields

## Contract

Every `link/body` theme rule provides `foreground` and `typography`, which the
Link renderer reads when constructing the element.

## Implementation

`ThemeDocument::validate` now requires both renderer-consumed fields for Link
styles. The regression removes each property in turn and requires validation
to reject the document.

## Evidence

- `./scripts/cargo-headstart test -p rustui link_theme_requires_renderer_foreground_and_typography --lib --locked --offline` exited successfully.
- `git diff --check -- crates/rustui/src/theme/schema.rs` passed.

This prevents accepted Link themes from reaching the renderer's missing-field
panic path. Native visual and accessibility acceptance remain open.
