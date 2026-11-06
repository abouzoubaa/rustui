# M1061 — Stage 7 / keep TokenField identity stable across placeholder changes

## Change

TokenField's implicit retained identity now uses the stable `token-field`
component kind instead of its visible placeholder. Changing the localized
placeholder therefore preserves the parent identity and the child TextField
identity. Owner/sibling slot allocation still keeps sibling TokenFields
distinct.

## Focused evidence

`./scripts/cargo-headstart test -p rustui --lib components::input::token_field::tests::implicit_identity_survives_placeholder_localization_and_distinguishes_siblings`

Result: 1 passed. The regression checks same-slot identity across an English to
French placeholder change and separation between sibling slots. Native focus,
keyboard, IME, and screen-reader behavior remain unverified.
