# M874 — Stage 5 Menu renderer background

## Change

Theme validation now requires the renderer-required `background` property on
each Menu `body` style rule. Before this change, the activation path rejected a
missing background later, but `ThemeDocument::validate` accepted the document.
The shared renderer-required-property checker now covers Menu body rules.

## Focused evidence

- `menu_body_theme_requires_renderer_background` removes the Menu body
  background and confirms validation rejects the theme.
- `./scripts/cargo-headstart test -p rustui --locked --offline menu_body_theme_requires_renderer_background` exited successfully.
- Scoped `git diff --check -- crates/rustui/src/theme/schema.rs` passed.
- Whole-file `rustfmt --edition 2024 --check` still reports pre-existing
  formatting differences in the dirty schema file. The file was not broadly
  reformatted.

## Acceptance boundary

This aligns Menu schema validation with theme activation. It does not establish
native visual or accessibility acceptance.
