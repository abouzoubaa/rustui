# D1c TabView preview specimen

The canonical compiled-preview gallery registers an opt-in, signal-free public
RustUI `TabView` specimen. Its descriptor exposes one editable string property,
`selection`, constrained to the stable tab IDs `overview`, `activity`, and
`settings`. The factory renders three `TabItem`s through the public `TabView`
API and binds the selected ID through the existing generation-stamped preview
feedback path. No event or action payload is added. The existing descriptor
codec carries the selection through normalized export/import.

## Validation

The focused regression covers the schema and allowed values, accepted and
rejected selection feedback, revision advancement, and export/import generation
behavior. Focused locked/offline test and Playground all-targets check passed in
`target/headstart/candidates/tab-view-preview-validation/evidence/`. Rustfmt and
scoped diff checks passed. M280 native verification selected Overview,
Activity, and Settings and observed the bound selection/content updates. Export
preserved `selection: settings`; importing remounted generation 3 with Settings
still selected and its content visible. Keyboard, accessibility, locale, theme,
RTL, and cross-platform acceptance remain open. The disposable validation app
was removed, and the canonical Playground process was left running.
