# M756 — Stage 7 / fall back on padded Spinner localization key echoes

Spinner accessible-name resolution now treats whitespace-padded echoes of its
localization message ID as unresolved and uses the explicit or default label
fallback. Genuine translations remain unchanged.

The focused `localized_accessibility_label_uses_catalog_and_fallback`
regression passed (1/1); scoped `git diff --check` passed. Native accessibility,
screen-reader, locale-switching, and cross-platform acceptance remain open.
