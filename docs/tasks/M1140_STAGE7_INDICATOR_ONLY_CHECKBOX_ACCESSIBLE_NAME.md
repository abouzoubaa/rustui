# M1140 — Stage 7: allow an accessible name on indicator-only Checkbox

`CheckboxView::new` accepts an empty visible label so callers can create an
indicator-only checkbox and provide a non-empty name with
`.accessibility_label(...)`. Rendering still rejects an empty resolved
accessible name.

Focused regression: `indicator_only_checkbox_can_use_an_explicit_accessible_name`
passed 1/1 with
`./scripts/cargo-headstart test -p rustui --lib components::input::checkbox::accessibility_tests::indicator_only_checkbox_can_use_an_explicit_accessible_name`.

Native screen-reader acceptance remains open.
