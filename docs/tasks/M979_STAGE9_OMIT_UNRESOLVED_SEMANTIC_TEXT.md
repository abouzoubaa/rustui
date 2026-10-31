# M979 — Stage 9 / omit unresolved localized semantics text

Semantic labels, descriptions, and live-region values now omit blank resolved
text and untranslated message-key echoes rather than exposing keys as user
content. An empty literal live-region value remains a valid clear operation.

Focused check: `accessibility::semantics::tests` passed (16/16).

Native screen-reader acceptance remains open.
