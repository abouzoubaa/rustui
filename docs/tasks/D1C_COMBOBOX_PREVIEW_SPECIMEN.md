# D1c ComboBox preview specimen

The canonical compiled-preview gallery includes an opt-in public RustUI
`ComboBox` specimen with one serializable `selection` string constrained to
`rust`, `swift`, or `go`, matching the preview schema's lowercase enum-ID
contract. The visible/accessibility label is the fixed
`Choose a language` constructor label. The mounted control uses the shared
gallery string binding, a generation-stamped identity, and the existing
preview-host feedback path.

The descriptor schema has no events under the selected signal-only contract.
Selection edits are validated against `allowed_values`, then preserved by the
existing normalized descriptor export/import path.

## Validation

Focused source test and isolated locked/offline Playground all-targets check are
recorded at:

- `target/headstart/evidence/d1c-combobox-preview-tests-20261020.log`
- `target/headstart/evidence/d1c-combobox-preview-check-20261020.log`

These checks establish source and compilation behavior only. No GUI acceptance
was run; native selection/search, accessibility, locale/theme, and
cross-platform behavior remain open.
