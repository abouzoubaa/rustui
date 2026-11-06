# Table retained identity validation

`TableView::id` supplies the retained `TableState` identity. Reject empty and
whitespace-only values so an invalid explicit ID cannot alias another retained
table in the same scope. This follows the nonblank retained-identity checks in
Tree, KeyedList, and DataGrid.

The focused Table module suite passed 6/6 tests with the locked/offline
Headstart launcher (`target/headstart/evidence/table-id-validation-focused-20261021.log`);
rustfmt and scoped diff checks passed. Native interaction, accessibility, RTL,
cross-platform, and performance acceptance remain separate.

## M455 — retained identity validation (2026-10-21)

The builder rejects blank identity values before they reach the retained
`TableState`. Focused locked/offline Table tests passed 6/6, and rustfmt and
scoped diff checks passed. Evidence:
`target/headstart/evidence/table-id-validation-focused-20261021.log`.

## M491 — implicit sibling table identities

Unkeyed Table identities use the render owner's implicit retained slot, so
same-shaped sibling tables cannot collide in retained `TableState`. The focused
Table tests passed 7/7, and RustUI library/integration tests passed 572/572.
Evidence: `target/headstart/evidence/m491-table-implicit-sibling-identity-tests-20261021.log`
and `target/headstart/evidence/m491-m492-rustui-lib-tests-20261021.log`.

The Table-specific regression suite evidence is
`target/headstart/evidence/m491-table-module-tests-20261021.log`. The existing
render-owner sibling-slot stability test passed in
`target/headstart/evidence/m491-table-implicit-sibling-identity-tests-20261021.log`.
The package-wide `--lib --tests` result is in
`target/headstart/evidence/m491-m492-rustui-lib-tests-20261021.log`; the separate
unrestricted command exposed a Rustdoc/compiler metadata mismatch during
doctests, recorded in `target/headstart/evidence/m491-m492-rustui-tests-20261021.log`.
