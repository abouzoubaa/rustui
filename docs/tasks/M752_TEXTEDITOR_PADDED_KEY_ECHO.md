# M752 — Stage 8 / fall back on padded TextEditor localization key echoes

TextEditor accessibility names now treat a localized value as unresolved when
its trimmed text equals the message ID. The resolver then uses the preserved
literal label, visible placeholder, or default name, per the existing
TextEditor accessible-name contract.

Regression: `padded_localization_key_echo_uses_accessible_name_fallback`.
Validation: `./scripts/cargo-headstart test --locked --offline -p rustui --lib
padded_localization_key_echo_uses_accessible_name_fallback` passed (1 passed,
743 filtered out). Scoped rustfmt and diff whitespace checks passed.

This is source-level evidence only. Native accessibility-tree, screen-reader,
locale-switching, and cross-platform acceptance remain open; see
`TEXTEDITOR_ACCESSIBILITY_NAME.md`.
