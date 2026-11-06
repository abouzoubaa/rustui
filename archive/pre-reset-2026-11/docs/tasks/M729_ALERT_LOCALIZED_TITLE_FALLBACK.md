# M729 Stage 8 Alert localized-title accessible-name fallback

When a localized Alert title resolves to its own message key, the visible native
Alert still receives that title, but the RustUI wrapper must not use the raw key
as its accessible group name. The wrapper now treats blank and unresolved
localized titles as unavailable accessible-name candidates, so its accessible
name falls back to the Alert message and does not demote that message to a
description under an untranslated key. Successfully translated titles retain
the existing title-name and message-description behavior.

Focused locked/offline regression passed:

```sh
./scripts/cargo-headstart test -p rustui --lib components::overlay::alert::tests::unresolved_localized_title_does_not_name_alert_with_translation_key --locked --offline
```

Result: 1 passed, 733 filtered. `rustfmt --edition 2024 --check` and scoped
`git diff --check` passed. Native accessibility-tree and screen-reader
acceptance remain open.
