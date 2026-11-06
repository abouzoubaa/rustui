# Tree leaf activation

## M362 — Stage 8

`TreeView::on_activate` runs the shared RustUI `ControlAction` when an enabled
leaf is selected and activated. Pointer activation runs after GPUI Tree selects
the clicked row. Keyboard `Confirm` uses the same action and is captured only
for an enabled selected leaf. Folder confirmation continues to GPUI Tree's
expansion handler; disabled leaves do not activate. Application actions must
support keyboard invocation so the pointer and keyboard paths have one
consistent contract.

The implementation does not change GPUI Tree's folder behavior or add a
separate activation event. The Playground reconciliation gallery displays the
last activated leaf and activation count.

Focused library tests passed 13/13; see
`target/headstart/evidence/tree-leaf-activation-m362-test-20261021.log`. The
locked/offline Playground all-targets check passed after the Tree fix. Native
pointer/keyboard interaction, focus, accessibility, localization/RTL, and
cross-platform acceptance remain open.

```sh
./scripts/cargo-headstart test -p rustui tree::tests --lib --locked --offline
```

The compile check is recorded in
`target/headstart/evidence/d1c-property-inspector-preview-check-m361-20261021.log`.
