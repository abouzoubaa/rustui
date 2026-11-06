# M1130 — localize TokenField action names

## Change

TokenField Add and Remove controls now resolve accessible names through the
active locale catalogs. English, French, and Arabic messages cover adding a
token, removing a named token, and removing an empty token; visible English
fallbacks remain available when a translation is missing or blank.

## Focused evidence

```text
./scripts/cargo-headstart test -p rustui --lib token_field_action_names_use_localized_catalog_messages_with_fallbacks
1 passed
```

Scoped formatting and locale diff checks passed. Native screen-reader and
locale-switching acceptance remain open.
