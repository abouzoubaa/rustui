# M896: Require Stepper Theme Rules for Both Appearances

Stage 5 schema/renderer contract follow-up.

The Stepper renderer resolves `increment_button`, `decrement_button`,
`separator`, `increment_glyph`, and `decrement_glyph` styles. Theme validation
now requires each part to have a matching rule for both light and dark
appearances (an `any` appearance rule can cover either mode), preventing a
theme from validating and then failing during rendering because a required
style cannot resolve.

Focused regression: `theme::schema::tests::stepper_theme_requires_renderer_rules_for_both_appearances`.
