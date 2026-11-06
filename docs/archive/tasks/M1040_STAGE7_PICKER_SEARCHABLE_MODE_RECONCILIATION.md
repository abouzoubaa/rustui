# M1040 — Reconcile retained Picker searchable mode

## Problem

`PickerView::searchable(bool)` affected only initial `SelectState` creation.
Rebuilt views with the same retained identity ignored subsequent changes.

## Change

When searchable mode changes, Picker now replaces its retained `SelectState`
using the latest options and selected value and registers confirmation
handling for the replacement state.

## Focused evidence

`./scripts/cargo-headstart test -p rustui --lib components::input::picker::tests::searchable_changes_rebuild_retained_select_state --offline`

Result: 1 passed, 0 failed. `rustfmt --check --edition 2024
crates/rustui/src/components/input/picker.rs` passed. The test covers the
searchable-mode change predicate; retained native search interaction, focus,
keyboard, and accessibility acceptance remain open.
