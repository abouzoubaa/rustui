# M981 — Stage 7 / keep Stepper identity stable across label changes

Implicit Stepper identity now derives from its owner- and sibling-scoped slot,
not its visible localized label. Sibling Steppers remain distinct, while
label changes do not replace retained Stepper identity.

Focused check: `components::input::stepper::tests::implicit_identity_survives_localized_label_and_distinguishes_siblings`
passed (1/1). Rustfmt and scoped whitespace checks passed.

Native focus and keyboard acceptance remain open.
