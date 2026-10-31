# M1191 — Stage 5 custom profile font family

## Scope

Apply the profile's custom `typography.font_family` to the active GPUI theme,
while preserving the existing platform aliases for `SF Pro` and `System`.

## Change

The adapter resolves the profile family once before updating the theme. Custom
families pass through; native `SF Pro`/`System` use `.SystemUIFont`, and Web
uses Inter.

## Verification

Focused test passed:

```sh
./scripts/cargo-headstart test -p rustui custom_profile_font_family_reaches_gpui_theme --locked --offline
```

Result: 1 passed. The scoped locked/offline library check, module rustfmt, and
whitespace checks also passed. Native visual and cross-platform font
acceptance remain open.
