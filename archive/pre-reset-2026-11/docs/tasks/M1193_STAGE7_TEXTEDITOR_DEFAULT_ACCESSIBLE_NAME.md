# M1193 — Stage 7 TextEditor default accessible name

## Scope

Give a TextEditor without an explicit or localized accessible-name override a
nonempty name when its visible placeholder is blank.

## Change

The render path now always applies `resolved_accessibility_label` to the
underlying GPUI Components Textarea. With a blank placeholder, the existing
fallback resolves to `Text editor`; nonblank placeholders and explicit names
retain their current behavior.

## Verification

The focused regression passed:

```sh
./scripts/cargo-headstart test -p rustui --lib components::input::text_editor::tests::localized_accessible_name_defaults_when_placeholder_is_blank -- --exact
```

Result: 1 passed, 0 failed. Native accessibility-tree, screen-reader, and
cross-platform acceptance remain open.
