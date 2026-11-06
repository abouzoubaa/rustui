# Spinner accessible-name contract

## Scope

`Spinner()` keeps its default accessible name, `Loading`, and keeps the same
GPUI Components spinner rendering. An explicit `accessibility_label(...)`
override must contain at least one non-whitespace character. Nonblank labels
are retained as supplied, including their surrounding whitespace.

## Implementation

`SpinnerView::accessibility_label` validates the explicit label before storing
it. The default is established separately by `Default`, so it is unaffected.

`SpinnerView::localized_accessibility_label(message_id)` resolves the ID
through the active render locale. A prior explicit label remains the fallback;
otherwise the existing `Loading` label is used. Blank IDs are rejected, and
blank or unresolved catalog results use the fallback. This leaves Spinner's
identity and GPUI Components rendering unchanged.

## Validation

- Focused RustUI unit tests: `./scripts/cargo-headstart test -p rustui --lib components::feedback::spinner::tests --locked --offline` passed (3 passed, 0 failed). Evidence: `target/headstart/evidence/spinner-nonblank-accessible-name-m424-20261021.log`.
- `rustfmt --edition 2024 --check crates/rustui/src/components/feedback/spinner.rs` passed; `git diff --check -- crates/rustui/src/components/feedback/spinner.rs` passed.
- GUI accessibility acceptance: not performed.

## Catalog-backed name validation

- `./scripts/cargo-headstart test -p rustui --lib
  components::feedback::spinner::tests --locked --offline` passed (5 passed,
  0 failed).
- `rustfmt --edition 2024 --check
  crates/rustui/src/components/feedback/spinner.rs` and
  `git diff --check -- crates/rustui/src/components/feedback/spinner.rs
  docs/tasks/SPINNER_ACCESSIBILITY_NAME.md` passed.
- Native accessibility-tree, screen-reader, and cross-platform acceptance
  remain open.

## M605 — Catalog-backed accessible name

`SpinnerView::localized_accessibility_label(message_id)` resolves the message
through the active render locale. A prior explicit label remains the fallback;
otherwise the existing `Loading` default is used. Blank keys are rejected, and
blank or unresolved catalog results fall back without changing Spinner identity
or rendering. The focused Spinner suite passed 5/5. Native accessibility-tree,
screen-reader, and cross-platform acceptance remain open.
