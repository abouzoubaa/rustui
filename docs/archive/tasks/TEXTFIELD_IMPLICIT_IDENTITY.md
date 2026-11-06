# M462 — Stage 7 / implicit TextField retained identity

TextField's old fallback identity used only the field kind and placeholder. Two
same-kind fields with the same placeholder therefore selected the same retained
`InputState`. `RenderContext::retained_entity` reuses an entity for a matching
identity, so their cursor, selection, focus, and IME state could collide.

Id-less TextFields now combine that semantic fallback with a stable render-owner
prefix and sibling slot. The slot resets with each fresh render context and is
carried through environment scopes. Deferred geometry and cell contexts have
distinct owner prefixes. Explicit `.id(...)` values bypass the positional
allocator and retain their existing caller-owned meaning. Use explicit IDs for
fields that can be reordered or conditionally replaced.

## Validation

- `./scripts/cargo-headstart test --locked --offline -p rustui --lib implicit_retained_identity_tests::implicit_identities_are_distinct_by_sibling_slot_and_stable_across_renders` — 1 passed.
- `./scripts/cargo-headstart test --locked --offline -p rustui --lib text_field` — 21 passed.
- `rustfmt --edition 2024 --check crates/rustui/src/view.rs crates/rustui/src/components/input/text_field.rs` — passed.
- `git diff --check -- crates/rustui/src/view.rs crates/rustui/src/components/input/text_field.rs` — passed.

Logs: `target/headstart/evidence/textfield-implicit-identity-m462-test-20261021.log` and `target/headstart/evidence/textfield-implicit-identity-m462-text-field-tests-20261021.log`.

This is source and unit-test evidence. Native keyboard, focus, screen-reader,
and GUI acceptance remain open.
