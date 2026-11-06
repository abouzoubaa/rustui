# TextField clear-button accessible name

## M583 follow-up — catalog-backed accessible name

`TextFieldView::localized_clear_button_accessibility_label(message_id)` now
resolves the clear control's accessible name through the active RustUI catalog.
Explicit literal labels remain supported. Missing, blank, or key-echo catalog
results fall back to the literal label when supplied, otherwise the existing
English `Clear {placeholder}` label. SearchField inherits the same behavior.
English, French, and Arabic catalogs now define
`rustui-text-field-clear-button`.

Focused component, catalog-contract, and locale validation results are recorded
in `target/headstart/evidence/m583-textfield-clear-accessibility-*.log`.
Native AX-tree, VoiceOver, locale-switching, and cross-platform acceptance
remain open.

`TextFieldView::clear_button_accessibility_label` lets callers supply a localized clear-button accessible name. The existing English `Clear {placeholder}` name remains the fallback. Whitespace-only overrides are rejected and accepted text is preserved verbatim. `SearchField` returns the same `TextFieldView`, so the builder is available there as well.

## Original M335 validation

- Focused RustUI tests cover the default English fallback, localized override, and blank override rejection.
- GUI, screen-reader, and cross-platform acceptance were not run.

## Evidence

- `target/headstart/evidence/textfield-clear-accessibility-name-m335-test.log`
