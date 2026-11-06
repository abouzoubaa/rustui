# M1143 — Stage 7: allow an accessible name on indicator-only Radio

`RadioView::new` accepts an empty visible label so indicator-only radios can
use an explicit `.accessibility_label(...)`. Rendering still requires a
non-empty resolved accessible name.

Focused regression: `indicator_only_radio_can_use_an_explicit_accessible_name`
passed 1/1 with
`./scripts/cargo-headstart test -p rustui --lib indicator_only_radio_can_use_an_explicit_accessible_name`.

Native screen-reader acceptance remains open.
