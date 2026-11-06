# Picker accessible name

`Picker` uses its visible label as the default accessible name and permits an
explicit `.accessibility_label(...)` override. Its
`.localized_accessibility_label(...)` builder resolves through the active RustUI
message catalog and retains the explicit or visible label as a fallback. Blank
literal labels, overrides, and localization keys are rejected; accepted literal
strings are preserved verbatim.

`PickerView::new` validates the visible label before it can become the default
name, and the override builder validates explicit names. Focused unit tests
cover blank rejection, localized message resolution and fallback, and
preservation of nonblank text in
`crates/rustui/src/components/input/picker.rs`.

This is source-level validation only. Existing bounded native Picker
interaction evidence is recorded in `docs/tasks/C10_COMBO_BOX_SELECT.md`;
native accessible-name announcement, full keyboard and focus behavior,
VoiceOver, RTL, cross-platform, and visual acceptance remain open.

## Validation

- `CARGO_TARGET_DIR=target/headstart/picker-accessibility-validation ./scripts/cargo-headstart test --quiet -p rustui --lib picker --locked --offline` — passed, 5 tests, 0 failed.
- `CARGO_TARGET_DIR=target/headstart/picker-accessibility-validation ./scripts/cargo-headstart check --quiet -p rustui --all-targets --locked --offline` — passed.
- `rustfmt --check --edition 2024 crates/rustui/src/components/input/picker.rs` — passed.
- `git diff --check -- crates/rustui/src/components/input/picker.rs docs/tasks/PICKER_ACCESSIBILITY_NAME.md` — passed.

## M321 — Localized accessible name

Picker accessible names can resolve through the active RustUI message catalog.
The explicit label or visible Picker label is the fallback when the resolved
message is blank; blank message IDs are rejected. The focused Picker suite
passed 13/13 and an isolated locked/offline RustUI all-targets check passed.
Native name announcement and broader acceptance remain open. Logs:
`target/headstart/evidence/picker-localized-name-m321/focused-test.log` and
`target/headstart/evidence/picker-localized-name-m321/all-targets-check.log`.
# M402 — retain Picker confirmation binding
When a stable Picker identity is rendered with a replacement selection `Binding`, retained confirmation events now write through the latest Binding rather than the first one captured at subscription setup. The Picker module suite passed 7/7, including a latest-binding regression. Native interaction and accessibility acceptance remain open. Evidence: `target/headstart/evidence/picker-tests-20261021.log`.

## Retained writer reentrancy audit (2026-10-21)

`with_current_writer` now clones the retained writer into a local before calling
it. This ends the `RefCell` borrow before user binding code runs, so a callback
that synchronously replaces the retained writer cannot trigger a borrow panic.
A focused regression exercises replacement from inside the callback.

The focused Headstart test command passed 17/17, including the reentrancy
regression. Raw output: `target/headstart/evidence/picker-writer-reentrant-m425-20261021.log`.
Native Picker interaction, accessibility announcement, VoiceOver, RTL,
cross-platform, and visual acceptance remain open.
