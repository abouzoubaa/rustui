# M758 — Stage 9 / fall back on padded Icon localization key echoes

Icon accessible-name resolution now treats whitespace-padded echoes of its
localization message ID as unresolved and preserves its explicit fallback.
Resolved translations remain unchanged.

The focused locked/offline localization test filter passed 2 tests (Icon and
Image); scoped `git diff --check` passed. Native accessibility-tree,
screen-reader, locale-switching, and cross-platform acceptance remain open.
