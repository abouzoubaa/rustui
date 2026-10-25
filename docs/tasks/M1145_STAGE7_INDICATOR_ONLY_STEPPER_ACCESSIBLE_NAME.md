# M1145 — Stage 7: allow an accessible name on indicator-only Stepper

`StepperView::new` accepts an empty visible label so callers can provide an
explicit `.accessibility_label(...)`. Validation applies to the resolved
accessible name; an unnamed indicator-only Stepper still fails.

Focused regressions:

- `explicit_accessible_name_allows_blank_visible_label` passed 1/1.
- `rejects_blank_resolved_accessible_name` exited 0.

Both used `./scripts/cargo-headstart test -p rustui --lib` with their respective
test filters. Native screen-reader acceptance remains open.
