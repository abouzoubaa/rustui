# Overlay alert and popover accessible names

## Scope

`Alert` uses GPUI Components' `Role::Alert`, so its message needs nonempty
text. `Popover` exposes its trigger label on a button, so a blank literal or a
blank resolved translation can leave the button without a useful name. Context
menus do not add a separate trigger and retain their child content.

## Contract

- Alert builders reject blank literal messages and localization keys. If a
  resolved message is blank, Alert uses its nonblank title and then its
  validated identity as the visible-message fallback.
- Popover builders reject blank trigger labels and localization keys. A blank
  resolved translation keeps the validated visible trigger label.
- Menu action fallback and native-row styling constraints are documented in
  `docs/tasks/MENU_ACCESSIBLE_NAME_FALLBACKS.md`.

## Verification

The focused Dialog and Sheet suites passed 9/9 and 11/11. The current combined
overlay suite passed 53/53, including Menu regressions. Fresh isolated locked/offline
checks passed for RustUI and Playground. Exact logs are listed in the overlay and
Menu task notes. These source/build checks do not establish native accessibility
tree or screen-reader acceptance.

## M366 — Alert close-button resolved accessible name

Alert now checks the resolved `rustui-action-close` label before using it for
both visible text and the close button's accessible name. A whitespace-only
translation falls back to `Close`; nonblank translations are preserved. This
closes the source-level empty-name path through the current pinned button API.
The focused Alert suite passed 9/9, and Rust 2024 formatting plus scoped diff
checks passed. Native accessibility-tree, VoiceOver, localization, and visual
acceptance remain open.

Evidence: `target/headstart/evidence/alert-close-accessible-name-m366-test.log`.
