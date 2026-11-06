# M926 — Stage 3 / regress Search Field recipient TokenField bindings

M899 added the public controlled `TokenField`, and the Search Field reference
gallery now uses it for editable recipient tokens and the draft query. This
milestone adds focused regression coverage that writes both bindings and
confirms the gallery state receives those updates.

The locked, offline focused test passed 1/1:

```sh
./scripts/cargo-headstart test -p rustui-playground --lib \
  recipient_token_field_bindings_update_the_gallery_state --locked --offline
```

This verifies the gallery binding contract. Native token entry and removal,
visual appearance, and accessibility acceptance remain open.
