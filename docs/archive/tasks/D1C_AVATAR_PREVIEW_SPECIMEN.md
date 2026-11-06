# D1c Avatar preview specimen

## Scope

Add an opt-in compiled-preview catalog specimen for the public RustUI `Avatar`.
Its serializable inspector contract is deliberately bounded to the normalized
`name` shown by the initials avatar and a `diameter` in the supported 24–64 px
range. The component has no user action, so its event catalog remains empty
under the selected signal-only preview policy.

## Implementation

`apps/rustui-playground/src/stories/compiled_preview_gallery.rs` registers
`avatar-specimen` using only the public `Avatar` API. The focused regression
checks both schema bounds, empty events, defaults, and descriptor export/import
of changed name and diameter values.

## Validation

- Focused locked/offline regression passed 1/1 with an isolated target directory.
- Isolated locked/offline Playground all-targets check passed. It reports the
  existing unused `THEME_CATALOG_LEN` and `table_binding` warnings and the
  pinned `block 0.1.6` future-incompatibility notice.
- `rustfmt --edition 2024 --check` and scoped `git diff --check` passed.
- No GUI/native acceptance was run.

Evidence: `target/headstart/evidence/d1c-avatar-preview-tests-20261020.log` and
`target/headstart/evidence/d1c-avatar-preview-check-20261020.log`.

## Follow-up

Avatar native rendering, VoiceOver, and cross-platform behavior remain
unverified.
