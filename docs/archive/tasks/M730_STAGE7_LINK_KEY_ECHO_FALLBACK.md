# M730 — Stage 7 / Link localized key-echo fallback

The Link accessible-name contract says blank and untranslated key-echo values
fall back to the explicit accessible name or visible label. The resolver now
recognizes a key echo even when the catalog value has surrounding whitespace,
so an unlocalized message key cannot leak as the control's accessible name.

The focused Link unit test covers padded key-echo fallback alongside localized,
blank, and literal fallback behavior. Validation: `./scripts/cargo-headstart test
--locked --offline -p rustui --lib localized_accessible_name_uses_catalog_and_retains_fallbacks`.

This source-level check does not establish native accessibility-tree,
screen-reader, locale-switching, or cross-platform acceptance. Contract:
`docs/tasks/LINK_ACCESSIBILITY_NAME.md` (M612); component behavior:
`docs/tasks/C6_LINK_DESIGN.md`.
