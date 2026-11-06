# M879 — Stage 5 checkbox and radio body style requirements

## Change

Checkbox and Radio theme validation now rejects records with no `body` style
rule. Their renderers require that style, while the previous loop over existing
styles could pass vacuously when every body rule was removed.

## Focused evidence

- `theme::schema::tests::checkbox_and_radio_themes_require_body_style_rules` passed 1/1 (779 filtered).

## Acceptance boundary

This closes one vacuous theme-validation path for Checkbox and Radio. Broader
native visual and accessibility acceptance remains open.
