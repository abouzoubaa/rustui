# M757 — Stage 7 / fall back on padded Progress localization key echoes

Progress accessible-name resolution now treats whitespace-padded echoes of its
localization message ID as unresolved and uses the explicit or default label
fallback. Genuine translations remain unchanged.

The focused localization filter passed (2 tests, Progress and Spinner); scoped
`git diff --check` passed. Native accessibility, screen-reader,
locale-switching, and cross-platform acceptance remain open.
