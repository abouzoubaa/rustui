# M596 — typed non-Button theme validation

`ThemeDocument::validate` now validates typed non-Button component records at
load time by sharing selector and recognized-property checks with the runtime
resolver. It rejects empty type names, undeclared nonempty rule parts,
equal-specificity overlapping selectors, and malformed recognized properties
in any rule, including non-default interaction states.

Unknown/custom component objects without the typed `parts` or `styles` markers
remain opaque and round-trip unchanged. Unknown property payloads remain
metadata. Typed records with intentionally empty style lists remain accepted.
`themes/theme.schema.json` was not tightened because it cannot express the
cross-field checks and must preserve custom component extensions.

The focused theme loader/schema group passed 11/11 tests; the RustUI library
suite passed 681/681; the locked/offline workspace all-targets check passed.
Existing example warnings and the `block` future-incompatibility notice remain.
Native theme visual acceptance is still open. Evidence:
`target/headstart/evidence/m596-theme-validation-focused-20261021.log`,
`target/headstart/evidence/m596-m595-rustui-library-suite-20261021.log`, and
`target/headstart/evidence/m596-m597-workspace-all-targets-check-20261021.log`.
