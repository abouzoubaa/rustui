# Stepper accessible name

`Stepper` uses its visible label to derive the Increase and Decrease button
accessible names by default. Callers may override the name with
`.accessibility_label(...)`; the override must contain a non-whitespace
character and is retained exactly as supplied. Both button names use the
override while the visible label remains unchanged. The constructor also
rejects empty and whitespace-only visible labels.

Focused source tests cover blank visible-label and override rejection,
preservation of nonblank labels, default button-name derivation, and custom
override propagation. These checks do not establish native accessibility-tree
naming, screen-reader announcements, full keyboard behavior, visual rendering,
or cross-platform acceptance.

## Validation

- `CARGO_TARGET_DIR=target/headstart/stepper-accessibility-validation-20261020 ./scripts/cargo-headstart test --quiet -p rustui --lib stepper::tests --locked --offline` — passed, 6 tests.
- `CARGO_TARGET_DIR=target/headstart/stepper-accessibility-validation-20261020 ./scripts/cargo-headstart check --quiet -p rustui --all-targets --locked --offline` — passed.
- `rustfmt --check --edition 2024 crates/rustui/src/components/input/stepper.rs` — passed.
- Scoped `git diff --check` — passed.

Evidence log: `target/headstart/evidence/stepper-accessibility-name-20261020.log`.

## M316 — Explicit accessible-name override follow-up

`StepperView::accessibility_label(...)` now provides a nonblank override for
the derived Increase and Decrease button names while preserving the visible
label. Focused source tests cover default derivation, override propagation, and
blank override rejection. Native accessibility-tree and screen-reader
acceptance remain open.

- `CARGO_TARGET_DIR=target/headstart/stepper-accessibility-override-20261020 ./scripts/cargo-headstart test --quiet -p rustui --lib stepper::tests --locked --offline` — passed, 8 tests.
- `CARGO_TARGET_DIR=target/headstart/stepper-accessibility-override-20261020 ./scripts/cargo-headstart check --quiet -p rustui --all-targets --locked --offline` — exited 0.
- `rustfmt --check --edition 2024 crates/rustui/src/components/input/stepper.rs` — passed.
- `git diff --check -- crates/rustui/src/components/input/stepper.rs docs/tasks/STEPPER_ACCESSIBILITY_NAME.md` — passed.

## Localized accessible-name follow-up

`StepperView::localized_accessibility_label(...)` resolves a message ID through
`RenderContext::localized_message` and uses the resolved name for both Increase
and Decrease buttons. A nonblank explicit `.accessibility_label(...)` is kept
as the localization fallback; otherwise the visible Stepper label is used.
Blank localization IDs are rejected, and blank or unresolved catalog results
fall back to the selected fallback label. Focused tests cover key validation,
fallback selection, message resolution, and both derived button labels.

- `./scripts/cargo-headstart test --quiet -p rustui --lib stepper::tests --locked --offline` — passed, 11 tests.
- `./scripts/cargo-headstart check --quiet -p rustui --all-targets --locked --offline` — passed.
- `rustfmt --check --edition 2024 crates/rustui/src/components/input/stepper.rs` — passed.
- Scoped `git diff --check -- crates/rustui/src/components/input/stepper.rs docs/tasks/STEPPER_ACCESSIBILITY_NAME.md` — passed.

Native accessibility-tree naming, screen-reader announcements, catalog coverage
for application-provided IDs, and cross-platform acceptance remain open.
