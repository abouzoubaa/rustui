# NavigationLink accessible-name contract

`NavigationLinkView::accessibility_label` lets callers provide a descriptive accessible name without changing the visible label. The visible label remains the default name. Blank visible labels and blank explicit accessible-name overrides are rejected; accepted overrides are preserved verbatim.

## Validation

- Focused RustUI unit tests cover the visible-label fallback, explicit override, and blank visible/override rejection.
- GUI and screen-reader acceptance were not run; native name announcements and keyboard behavior remain open.

## Evidence

- `target/headstart/evidence/navigation-link-accessibility-name-m332-test.log`
