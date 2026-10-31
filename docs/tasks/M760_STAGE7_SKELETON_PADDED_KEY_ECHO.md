# M760 — Stage 7 / fall back on padded Skeleton localization key echoes

Skeleton accessible-name resolution now treats whitespace-padded echoes of its
localization message ID as unresolved and preserves its explicit or default
fallback.

The focused locked/offline
`localized_accessibility_label_uses_explicit_fallback` test passed 1/1; scoped
formatting and diff checks passed. Native accessibility-tree, screen-reader,
locale-switching, and cross-platform acceptance remain open.
