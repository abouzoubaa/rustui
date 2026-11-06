# M871 — Stage 6 unclassified core locale subtags

## Change

Locale parsing now rejects core subtags that do not match an allowed BCP 47
core class. Previously, malformed mixed-length values such as `a1b`, `a123`,
and `12a` could pass the general alphanumeric check without being a valid
extlang, script, region, variant, or singleton. Extension and private-use
payload validation remains separate.

## Focused evidence

- The regression rejects `en-a1b`, `en-a123`, and `en-12a`.
- The regression still accepts `en-u-a1b` and `en-9-a1b` as Unicode and
  generic extension payloads, and `en-x-a1b` as a private-use payload.
- `./scripts/cargo-headstart test -p rustui-i18n locale::tests::rejects_unclassified_core_subtags_but_allows_extension_values --lib --locked --offline` passed: 1 passed, 0 failed, 59 filtered.
- `git diff --check -- crates/rustui-i18n/src/locale.rs` exited successfully.

## Acceptance boundary

This closes a structural locale-parser gap. Full registered-tag coverage,
ICU/CLDR behavior, RTL, accessibility, and cross-platform acceptance remain
open.
