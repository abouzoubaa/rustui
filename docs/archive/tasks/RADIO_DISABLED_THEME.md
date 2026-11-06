# Radio Disabled Theme Label

## Scope

Use the active platform profile's `disabled_text` token for a disabled Radio
label in both light and dark appearance. Preserve the resolved theme foreground
for enabled labels and retain the existing disabled indicator dimming.

## Implementation

`RadioView::into_element` resolves the label color from the active theme profile
and appearance. Disabled opacity applies to the indicator only, matching the
checkbox behavior and keeping the semantic disabled label color intact.

## Verification

Focused unit tests cover disabled token selection in light and dark appearance,
and enabled label foreground preservation in both modes. Source tests do not
establish native GUI acceptance.

Focused verification on 2026-10-24:

```text
./scripts/cargo-headstart test -p rustui --lib disabled_label_uses_the_active_platform_disabled_text_color --locked --offline
1 passed, 0 failed, 866 filtered out

./scripts/cargo-headstart test -p rustui --lib enabled_label_keeps_the_resolved_theme_foreground --locked --offline
1 passed, 0 failed, 866 filtered out
```
