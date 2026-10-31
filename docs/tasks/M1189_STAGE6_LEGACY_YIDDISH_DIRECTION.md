# M1189 — Stage 6 legacy Yiddish direction

## Scope

Correct the default layout direction for the accepted legacy BCP 47 language
tag `ji` (Yiddish).

## Change

`LayoutDirection::for_locale` now treats `ji` as right-to-left, alongside the
preferred `yi` tag. Locale parsing continues to preserve the caller's tag.

## Verification

Focused check passed:

```sh
./scripts/cargo-headstart test -p rustui-i18n deprecated_yiddish_language_alias_uses_right_to_left_direction
```

Result: 1 passed. Full localization and native/browser RTL acceptance remain
open.
