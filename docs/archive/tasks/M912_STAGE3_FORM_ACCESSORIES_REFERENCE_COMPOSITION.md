# M912 — Stage 3 / complete the Form Accessories reference composition

The Playground gallery now presents the eleven control groups named in
`apps/rustui-playground/references/Form.svg`: Button, Checkboxes, Combo Box,
Detail Disclosure, Detail, Detail Info, Detail Menu, Pop-Up Button, Radio
Button, Stepper, and Switch. It uses RustUI's public controls and icons. The
filter now searches these reference labels.

The initial state follows the artwork: the first checkbox and radio are
selected, the second of each is not; the Combo Box and Pop-Up Button show
`Label`; the Stepper starts at 67; the Switch is on; and the disclosure is
closed. The two radio bindings share one selection value so selecting the
other radio keeps the choices mutually exclusive.

The reference shows the menu and disclosure closed but does not specify menu
items or disclosure content. The gallery leaves both contents empty rather
than inventing actions or expanded details. `Form-1.svg` contains the same
control groups with plus signs in three Detail captions; this gallery follows
the captions from the requested `Form.svg`.

Focused checks passed:

- `./scripts/cargo-headstart test -p rustui-playground form_accessories --lib` — 4 passed, 0 failed (route, filter, catalog-to-Input routing, and reference-state regression).
- `rustfmt --edition 2024` was applied to the gallery source.

This is source-level composition evidence. Native visual, accessibility, and
interaction acceptance remain open.
