# M753 — Stage 7 / fall back on padded EmptyState localization key echoes

`EmptyState` accessible-name resolution now treats whitespace-padded echoes of
the localization message ID as unresolved, preserving the explicit accessible
label or visible title fallback. Nonblank translations remain unchanged.

The focused regression `components::feedback::empty_state::tests::localized_accessibility_name_uses_catalog_and_preserves_fallback` passed
with:

```sh
./scripts/cargo-headstart test -p rustui components::feedback::empty_state::tests::localized_accessibility_name_uses_catalog_and_preserves_fallback --lib --locked --offline
```

The test passed 1/1 (743 filtered out). Scoped `git diff --check` passed.
Native accessibility-tree, screen-reader, locale-switching, and cross-platform
acceptance remain open.
