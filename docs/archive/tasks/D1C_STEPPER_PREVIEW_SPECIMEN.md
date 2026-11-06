# D1c Stepper preview specimen

## Contract

The compiled-preview gallery exposes an opt-in `stepper` descriptor with a
serializable `label` string and integer `value`. The label defaults to
`Preview count`, is bounded to 1–80 characters, and rejects whitespace-only
imports. The integer value defaults to 4 and is constrained to 0–10. The
descriptor declares no events under the selected signal-only D1b contract.

The public RustUI `Stepper` receives the descriptor label and keeps its value
connected through the existing generation/revision-stamped binding. Accepted
feedback updates the mounted preview and round-trips through the descriptor
codec. Invalid imports leave the mounted preview and its stamp unchanged.

## Verification

- Focused gallery test passed 1/1. It covers bounds/defaults, empty events,
  accepted and stale feedback, descriptor round-trip, and rejection of blank or
  oversized labels and malformed or out-of-range integer values.
- Isolated locked/offline Playground all-targets check passed.
- `rustfmt --check --edition 2024 apps/rustui-playground/src/stories/compiled_preview_gallery.rs` passed.
- Scoped `git diff --check` passed.

Logs: `target/headstart/evidence/d1c-stepper-preview-specimen-test-20261020.log`
and `target/headstart/evidence/d1c-stepper-preview-specimen-check-20261020.log`.
No GUI or native accessibility acceptance was run.
