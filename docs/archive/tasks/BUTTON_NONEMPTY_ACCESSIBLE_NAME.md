# Button nonempty accessible-name contract

At render time, `ButtonView` resolves its accessible name from an explicit `accessibility_label` override or the visible label. The resolved value must contain a non-whitespace character. This keeps icon-only buttons valid when callers supply a name and rejects unnamed buttons before they reach the accessibility tree.

## Validation

- Focused RustUI tests cover the icon-only override and rejection of an empty visible label without an override.
- Native accessibility-tree, screen-reader, and cross-platform acceptance were not run.

## Evidence

- `target/headstart/evidence/button-nonempty-accessible-name-m338-test.log`
