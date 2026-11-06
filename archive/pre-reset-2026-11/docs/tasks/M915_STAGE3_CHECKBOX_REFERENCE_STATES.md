# M915 — Stage 3 Checkbox reference states

## Gap

The partial `checkbox` catalog row points to `docs/ui/macos-27/Form.svg`. Its
`Checkboxes` row contains one checked checkbox and one unchecked checkbox.
The shared Input story had an active checkbox and a disabled checkbox sharing
the same initially-false binding, so it could not show the reference's distinct
checked state.

## Change

Added a small stateful checkbox reference specimen to the Input story with
independent checked and unchecked controls. The controls keep separate state
and have stable IDs and explicit accessibility labels. Existing disabled
checkbox coverage remains intact.

## Focused evidence

- `./scripts/cargo-headstart test -p rustui-playground checkbox_reference_gallery_starts_with_checked_and_unchecked_states --locked --offline`
  passed: 1 test; 138 filtered out.
- The shared-target invocation of the independent-binding regression initially
  stopped before the test runner with exit 101 and `E0463` for `gpui` while
  compiling `rustui`.
- Retrying only that test with
  `CARGO_TARGET_DIR=target/headstart/verification/stage3-m915` passed: 1 test,
  0 failures, 183 filtered out. This is source-level binding evidence. Native
  visual, interaction, and accessibility acceptance remain open.
