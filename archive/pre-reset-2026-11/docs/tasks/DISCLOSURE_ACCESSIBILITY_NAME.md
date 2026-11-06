# Disclosure accessible-name contract

`DisclosureView::accessibility_label` lets callers give the native disclosure trigger a descriptive accessible name while keeping its visible label unchanged. Without an override, the visible label remains the accessible name. Blank overrides are rejected, and accepted text is retained verbatim.

## Validation

- Focused RustUI unit tests cover the default name, an explicit nonblank override, blank rejection, and disabled activation behavior.
- GUI and screen-reader acceptance were not run; native name announcements and keyboard behavior remain open.

## Evidence

- `target/headstart/evidence/disclosure-accessibility-name-m331-test.log`
