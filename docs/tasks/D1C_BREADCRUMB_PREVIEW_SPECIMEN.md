# M291 — D1c Breadcrumb preview specimen

The compiled-preview gallery adds an opt-in fixed two-level `Breadcrumb`
specimen. Its descriptor stores only bounded, nonblank `parent_label` and
`current_label` scalar values. The parent item emits a `parent` event with an
empty payload, mapped to the separately allowlisted `breadcrumb_parent` action.
The public factory constructs the two items directly; arbitrary path arrays and
closures are not serialized.

The focused regression covers both schema defaults and bounds, the signal-only
action mapping and dispatch, descriptor round-trip, and rejection of blank
imports for either label without replacing the mounted preview.

Native activation, keyboard/focus, accessibility, localization, and
cross-platform acceptance remain separate gates.

## Verification

The focused locked/offline Breadcrumb regression passed 1/1. The locked/offline
Playground `--all-targets` check passed. Both used the isolated
`target/headstart/candidates/m291-validation/target` Headstart target. Scoped
rustfmt and `git diff --check` passed. Evidence:
`target/headstart/evidence/m291-breadcrumb-preview-test-20261020.log` and
`target/headstart/evidence/m291-breadcrumb-preview-check-20261020.log`.

These source checks do not establish native activation, keyboard/focus,
accessibility, localization, or cross-platform acceptance.
