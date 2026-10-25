# M761 — Stage 8 / fall back on padded Badge localization key echoes

Badge accessible-name resolution now treats whitespace-padded echoes of its
localization message ID as unresolved, preserving the explicit or default
fallback while keeping genuine translations.

The focused locked/offline regression filter passed 2 tests; rustfmt and scoped
`git diff --check` passed. Native accessibility-tree, screen-reader,
locale-switching, and cross-platform acceptance remain open.
