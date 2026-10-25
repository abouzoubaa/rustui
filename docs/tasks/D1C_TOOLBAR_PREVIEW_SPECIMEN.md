# M289 — D1c Toolbar preview specimen

The compiled-preview gallery adds an opt-in, two-action RustUI `Toolbar`
specimen. Its descriptor exposes bounded, nonblank `primary_label` and
`secondary_label` strings plus an optional `secondary_disabled` Boolean. The
`primary` and `secondary` events have empty payloads and map to the separately
allowlisted `toolbar_primary` and `toolbar_secondary` actions. No event payload
or executable closure is serialized.

The focused regression covers schema bounds and defaults, the empty-payload
event contracts, distinct action mappings and dispatch, descriptor round-trip
including the disabled state, and rejection of a blank imported label without
replacing the mounted preview.

Native activation, keyboard/focus behavior, accessibility, localization, and
cross-platform acceptance remain separate gates.

## Verification

The focused locked/offline Toolbar regression passed 1/1. The locked/offline
Playground `--all-targets` check passed. Both used the isolated
`target/headstart/candidates/m289-validation/target` Headstart target. Scoped
rustfmt and `git diff --check` passed. Evidence:
`target/headstart/evidence/m289-toolbar-preview-test-20261020.log` and
`target/headstart/evidence/m289-toolbar-preview-check-20261020.log`.

These source checks do not establish native activation, keyboard/focus,
accessibility, localization, or cross-platform acceptance.
