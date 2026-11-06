# M292 — D1c ContextMenu preview specimen

The compiled-preview gallery adds an opt-in fixed one-action `ContextMenu`
specimen. Its descriptor stores only bounded, nonblank `target_label` and
`action_label` strings. The parent item is a single public `MenuItem::action`;
its `activate` event has an empty payload and maps to the allowlisted
`context_menu_open_preview` action. Menu structure and closures are fixed in the
compiled factory and are not serialized.

The focused regression covers both schema defaults and bounds, the signal-only
action mapping and dispatch, descriptor round-trip, and rejection of blank
imports for either label without replacing the mounted preview.

Native right-click/long-press behavior, keyboard/focus, accessibility,
localization, and cross-platform acceptance remain separate gates.

## Verification

The focused locked/offline ContextMenu regression passed 1/1. The locked/offline
Playground `--all-targets` check passed. Both used the isolated
`target/headstart/candidates/m292-validation/target` Headstart target. Scoped
rustfmt and `git diff --check` passed. Evidence:
`target/headstart/evidence/m292-contextmenu-preview-test-20261020.log` and
`target/headstart/evidence/m292-contextmenu-preview-check-20261020.log`.

These source checks do not establish native right-click/long-press behavior,
keyboard/focus, accessibility, localization, or cross-platform acceptance.
