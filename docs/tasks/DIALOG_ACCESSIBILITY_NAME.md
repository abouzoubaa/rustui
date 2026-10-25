# Dialog accessible-name validation

## M632 — Skip unresolved localized name candidates

Resolved explicit accessible-name and title candidates are skipped when their translations are blank or return the localization key unchanged. Resolution then uses the next nonblank title or trigger candidate, with stable Dialog identity as the final fallback. The focused unresolved-key regression passed 1/1. Native accessibility-tree and screen-reader acceptance remain open.

`DialogView::accessibility_label` rejects whitespace-only literal overrides
and preserves nonblank input verbatim. The optional override remains optional:
when absent, the accessible name still falls back to the visible title and
then the trigger label. The localized accessible-name builder also rejects whitespace-only message keys.

Dialog trigger labels and localization keys must be nonblank. If a localized
trigger resolves blank, the visible trigger label remains in use for both the
button and the dialog-name fallback.

Focused tests cover blank rejection, exact text preservation, blank-translation
fallback, and the existing localized and accessible-name fallback behavior.

- `./scripts/cargo-headstart test --locked --offline -p rustui --lib components::overlay::dialog::tests`: 13 passed.
- `rustfmt --check --edition 2024 crates/rustui/src/components/overlay/dialog.rs`: passed.
- `git diff --check -- crates/rustui/src/components/overlay/dialog.rs docs/tasks/DIALOG_ACCESSIBILITY_NAME.md`: passed.
- Test log: `target/headstart/evidence/dialog-trigger-accessible-name-20261021.log`.

These source tests do not establish native accessibility-tree or screen-reader
acceptance.

## Localized accessible-name key validation (2026-10-21)

`DialogView::localized_accessibility_label` now rejects whitespace-only message
keys, matching the explicit literal accessible-name and localized-trigger
builders. This prevents an invalid explicit name from being silently skipped by
the resolved-name fallback. The focused Dialog suite passed 14/14; the locked,
offline RustUI all-targets check passed with existing example warnings. Evidence:
`target/headstart/evidence/dialog-localized-name-key-contract-20261021.log` and
`target/headstart/evidence/overlay-localized-name-key-rustui-check-20261021.log`.
Native accessibility-tree and screen-reader acceptance remain open.
