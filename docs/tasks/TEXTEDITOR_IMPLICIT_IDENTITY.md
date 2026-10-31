# M465 — Stage 7 / distinguish implicit TextEditor identities by sibling

TextEditor previously derived its retained identity only from its placeholder.
Two editors with the same placeholder could therefore resolve to one retained
`TextareaState`, sharing selection, cursor, focus, and undo state. TextEditor now
uses the same implicit owner-and-sibling identity allocator as TextField.

Explicit `.id(...)` values continue to bypass implicit allocation and retain
their caller-controlled identity. Use explicit IDs when editors can be
reordered or conditionally replaced.

## Validation

- The focused implicit-identity regression verifies that same-placeholder
  TextEditor siblings receive distinct identities and keep those identities
  across renders; the owner prefix keeps separate owners distinct. The
  explicit-ID builder regression verifies caller identity preservation.
- `./scripts/cargo-headstart test --locked --offline -p rustui --lib implicit_retained_identity_tests::implicit_identities_are_distinct_by_sibling_slot_and_stable_across_renders`
- `./scripts/cargo-headstart test --locked --offline -p rustui --lib text_editor`
- `rustfmt --edition 2024 --check crates/rustui/src/view.rs crates/rustui/src/components/input/text_editor.rs`
- `git diff --check -- crates/rustui/src/view.rs crates/rustui/src/components/input/text_editor.rs docs/tasks/TEXTEDITOR_IMPLICIT_IDENTITY.md`

Evidence logs:
`target/headstart/evidence/texteditor-implicit-identity-m465-test-20261021.log`,
`target/headstart/evidence/texteditor-implicit-identity-m465-text-editor-tests-20261021.log`,
`target/headstart/evidence/texteditor-implicit-identity-m465-rustfmt-20261021.log`,
and `target/headstart/evidence/texteditor-implicit-identity-m465-diff-check-20261021.log`.

These are source and unit-test checks. Native keyboard, focus, IME,
screen-reader, and GUI acceptance remain open.
