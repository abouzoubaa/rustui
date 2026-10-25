# M878 — Stage 5 icon clear style requirement

## Change

Theme validation now requires an `icon/clear` rule. The renderer always
resolves this style and reads its vector path; previously, removing every
clear rule let validation pass and deferred failure to rendering.

## Focused evidence

- `icon_theme_requires_a_renderer_clear_style_rule` passed 1/1 after reproducing the validator hole before the fix.
- Scoped `git diff --check` passed.

## Acceptance boundary

This closes one vacuous theme-validation path. Broader native visual and
accessibility acceptance remains open.
