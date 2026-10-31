# M759 — Stage 7 / fall back on padded ErrorState localization key echoes

ErrorState accessible-name resolution now treats whitespace-padded echoes of
its localization message ID as unresolved and preserves the explicit-label or
visible-title fallback.

The focused locked/offline
`padded_localized_key_echo_uses_explicit_or_default_fallback` test passed 1/1;
scoped `git diff --check` passed. Native accessibility-tree, screen-reader,
locale-switching, and cross-platform acceptance remain open.
