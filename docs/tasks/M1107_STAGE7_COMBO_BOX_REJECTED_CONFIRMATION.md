# M1107 — Stage 7 ComboBox rejected confirmation reconciliation

## Contract

The ComboBox keeps its selection controlled by the owner's binding. A retained
GPUI selection event may move the control before the owner accepts the write.
On the next render, the control must reconcile to the owner's current value,
including when the owner rejects a confirmation and keeps its previous value.

## Change

Record the confirmed value in retained ComboBox state before invoking the latest
owner binding writer. If the owner rejects the write, the retained value differs
from the next owner render and the existing reconciliation path restores the
controlled selection.

## Evidence

`rejected_confirmation_reconciles_to_controlled_owner_value` passed with
`./scripts/cargo-headstart test -p rustui rejected_confirmation_reconciles_to_controlled_owner_value`
(1 passed). `git diff --check` passed for `combo_box.rs`.

This is focused source-level evidence. Native interaction, keyboard, and
accessibility acceptance remain open.
