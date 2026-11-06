# M921 — Stage 7 / keep implicit TextEditor identity stable across localization

TextEditor previously included its placeholder in its implicit retained
identity. A locale change therefore selected a different retained
`TextareaState` before M806's placeholder-refresh path could update the current
state. Implicit identity now uses a stable `text-editor` base with the existing
owner/sibling slot allocator; explicit `.id(...)` remains caller-controlled.

The focused regression
`implicit_identity_survives_placeholder_localization_and_separates_siblings`
passed 1/1 with:

```sh
./scripts/cargo-headstart test --locked --offline -p rustui --lib \
  components::input::text_editor::tests::implicit_identity_survives_placeholder_localization_and_separates_siblings
```

Scoped `rustfmt --check` and `git diff --check` passed. Native localization,
keyboard, IME, and accessibility acceptance remain open.
