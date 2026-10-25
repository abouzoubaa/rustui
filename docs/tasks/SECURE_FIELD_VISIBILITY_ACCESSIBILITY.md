# SecureField visibility control accessibility

## M618 — Playground catalog sample

The compiled preview SecureField specimen now uses
`localized_password_visibility_accessibility_label("secure_field.toggle_visibility")`.
The English, French, and Arabic Playground catalogs contain this key. Locale
validation and the locked/offline Playground all-targets check passed. This is
sample/catalog coverage; native accessibility and screen-reader acceptance
remain open.

`SecureField` replaces GPUI Components' built-in icon-only mask toggle with a RustUI-owned button. The button has the accessible name `Toggle password visibility`, accepts a literal override through `password_visibility_accessibility_label`, or a catalog message ID through `localized_password_visibility_accessibility_label`. The localized builder resolves through the active RustUI catalog and falls back to the literal override or the default English name when the catalog result is missing, blank, or unchanged from the message ID. Both controls participate in keyboard focus traversal. Activation toggles masking and restores focus to the text field. The visibility choice persists across rerenders; switching between plain and secure modes resets masking to the new mode's default. The password remains masked by default.

## Validation

- Focused RustUI tests cover the default visibility-control name, literal and catalog-backed overrides, catalog fallback, blank-name and blank-key rejection, visibility persistence across same-mode renders, and masking resets on plain/secure mode transitions. The TextField test module passed 29/29 tests with `./scripts/cargo-headstart test -p rustui --lib components::input::text_field::tests --locked --offline --quiet` (M572).
- Source uses a focusable native button for keyboard activation; GUI, screen-reader, and cross-platform acceptance were not run.

## Evidence

- `target/headstart/evidence/secure-field-visibility-accessibility-m341-test.log`
- `target/headstart/evidence/secure-field-mask-persistence-m341.log`
- `target/headstart/evidence/secure-field-mask-transition-regression-m341.log`
# M404 — expose SecureField visibility toggle state
The RustUI-owned password visibility button now exposes its revealed state through GPUI Components' `Button::toggled`, which maps to the accessible pressed state without changing selected styling. Focused TextField tests passed 20/20, including the mapping regression. GUI and screen-reader acceptance remain open. Evidence: `target/headstart/evidence/secure-field-pressed-semantics-m404-20261021.log`.
