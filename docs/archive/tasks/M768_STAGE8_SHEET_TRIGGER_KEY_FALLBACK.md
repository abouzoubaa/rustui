# M768 — Stage 8 / fall back from unresolved Sheet trigger labels

Sheet now treats a blank translation or a translation equal to its message key
(including surrounding whitespace) as unresolved and falls back to the visible
trigger label. Real translations remain unchanged. The shared resolver feeds
both the rendered trigger and its accessible-name fallback.

The focused regression passed (1/1); scoped formatting and `git diff --check`
passed. Native accessibility-tree, screen-reader, locale-switching, RTL, and
cross-platform acceptance remain open.
