# Progress accessible-name contract

`ProgressView::accessibility_label` rejects whitespace-only names at the
builder boundary while preserving valid caller-provided text exactly. The
default remains the nonempty `Progress` label. This is source-level validation;
it does not establish native accessibility-tree or screen-reader behavior.

## Validation

- `CARGO_TARGET_DIR=target/headstart/progress-accessibility-validation
  ./scripts/cargo-headstart test --quiet -p rustui --lib accessibility_label
  --locked --offline` — passed, 7 tests, 0 failed.
- `rustfmt --edition 2024 crates/rustui/src/components/feedback/progress.rs`
  completed before compilation.

## Remaining acceptance

Native AX-tree, screen-reader, and cross-platform observations remain open.


## M512 — validate Progress retained identity

`ProgressView::new(...)` rejects empty and whitespace-only IDs before they
reach the upstream progress element; valid caller IDs are preserved verbatim.
The focused Progress suite passed 7/7 and the full RustUI library suite passed
583/583. Native visual, accessibility, and cross-platform acceptance remain
open. Evidence: `target/headstart/evidence/m512-progress-identity-validation-20261021.log`
and `target/headstart/evidence/rustui-lib-regression-after-m512-20261021.log`.

## M602 — catalog-backed accessible name

`ProgressView::localized_accessibility_label(message_id)` resolves the supplied
RustUI message ID through the active render locale. A prior explicit
`.accessibility_label(...)` remains the fallback; otherwise the existing
`Progress` default is used. Blank IDs are rejected, and blank or unresolved
catalog results use the fallback. The behavior leaves the progress value,
identity, and determinate/indeterminate mode unchanged.

Validation: the focused Progress suite passed 9/9; the locked/offline RustUI
all-targets check passed with existing example warnings; formatting and scoped
diff checks passed. This is source-level validation; native accessibility-tree,
screen-reader, and cross-platform acceptance remain open. Evidence:
`target/headstart/evidence/m602-progress-localized-accessibility-tests-20261021.log`
and `target/headstart/evidence/m602-progress-localized-accessibility-check-20261021.log`.
