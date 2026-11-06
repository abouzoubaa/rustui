# M772 — Stage 8 / fall back from unresolved Dialog trigger labels

Dialog now treats blank translations and translations equal to its message key
(including padded echoes) as unresolved and uses the visible trigger label.
Valid translations remain unchanged. This aligns the Dialog trigger path with
the existing Sheet fallback behavior.

The focused locked/offline regression passed (1/1); scoped `git diff --check`
passed. `rustfmt --check` still reports two formatting differences in existing
dirty test blocks in this file, which were left untouched. Native dialog focus
containment and broader accessibility acceptance remain open.
