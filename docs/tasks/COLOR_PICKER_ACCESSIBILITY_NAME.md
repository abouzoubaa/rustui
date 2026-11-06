# ColorPicker accessible name

`ColorPicker` uses its visible label as the default accessible name. Both the
constructor label and explicit `.accessibility_label(...)` override reject
whitespace-only values; accepted strings are preserved verbatim.

The setter guard and focused tests live in
`crates/rustui/src/components/input/color_picker.rs`.

## Validation

- `CARGO_TARGET_DIR=target/headroom-progress-validation ./scripts/cargo-headstart test --quiet -p rustui --lib color_picker --locked --offline` — passed, 3 tests, 0 failed.
- `CARGO_TARGET_DIR=target/headroom-progress-validation ./scripts/cargo-headstart check --quiet -p rustui --all-targets --locked --offline` — passed (exit code 0).
- `rustfmt --check --edition 2024 crates/rustui/src/components/input/color_picker.rs` — passed.
- `git diff --check -- crates/rustui/src/components/input/color_picker.rs docs/tasks/COLOR_PICKER_ACCESSIBILITY_NAME.md` — passed.

This is source-level validation only. Native trigger naming and screen-reader
announcement, popup accessibility, keyboard behavior, RTL, cross-platform, and
visual acceptance remain open.
# M406 — refresh retained ColorPicker binding
ColorPicker now routes retained Change events through the latest owner `Binding` after a same-identity rerender. The focused suite passed 4/4, including the latest-writer regression. Popup interaction, screen-reader, visual, and cross-platform acceptance remain open. Evidence: `target/headstart/evidence/color-picker-current-binding-tests-m406-20261021.log`.

## Stage 7 bounded contract audit — 2026-10-21

- Current binding writes: each render replaces the retained event writer with
  the current owner's `Binding`; retained `Change` events invoke that writer.
  The existing latest-writer regression covers this path.
- Invalid hex: upstream `ColorPickerState` rejects invalid or incomplete input
  without changing the preview and commits only recognized widths. Its parser
  documents an optional `#`, while its input pattern requires `#`; this upstream
  input-format boundary was left unchanged.
- Accessible name: visible and explicit names reject whitespace-only strings;
  the resolved label is applied to both the RustUI trigger and controlled root.
- Disabled: this RustUI wrapper exposes no disabled builder and does not pass
  the upstream base picker's disabled setting. Disabled use is unsupported at
  this layer; no partial disabled API was added.
- No deterministic RustUI-owned defect was found, so no source change was made.
  Native color panel, screen-reader, visual, and cross-platform gates remain
  open.

Validation: `CARGO_TARGET_DIR=target/headstart ./scripts/cargo-headstart test
--quiet -p rustui --lib color_picker --locked --offline` — passed, 4 tests, 0
failed (2026-10-21).

## Localized accessible-name override — 2026-10-21

Added `.localized_accessibility_label(message_id)`. It rejects blank keys,
keeps an existing literal accessible-name override as its fallback, and uses
the visible ColorPicker label when no literal override exists. Rendering
resolves the key through `RenderContext::localized_message`; missing, key-echo,
or blank translations use the fallback.

Focused validation was attempted with
`./scripts/cargo-headstart test --quiet -p rustui --lib color_picker --locked
--offline`, but compilation stopped on concurrent errors outside this task:
`text_field.rs` imports a missing `accessibility::semantics::resolved_message_label`,
and `stepper.rs` calls `as_deref()` on `Option<StepperAccessibilityLabel>`.
The focused tests for the new builder and fallback behavior have not yet run.
Rustfmt and scoped diff checks are recorded by the current task run.
