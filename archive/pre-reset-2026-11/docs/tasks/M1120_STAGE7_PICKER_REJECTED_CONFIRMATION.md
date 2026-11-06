# M1120 — Stage 7 / reconcile rejected Picker confirmations

Picker confirmation previously called the owner's binding writer without first
updating the retained selection's last-observed value. When the owner rejected
the write and kept the old controlled value, the next render did not have a
reliable mismatch to reconcile against.

The event path now records the confirmed value before calling the latest owner
writer. On the next render, a rejected write is detected and the retained
`SelectState` is restored to the controlled owner value.

The focused regression `rejected_confirmation_reconciles_to_controlled_owner_value`
passed through `./scripts/cargo-headstart test -p rustui
rejected_confirmation_reconciles_to_controlled_owner_value`. The runner exited
successfully; its pass count was not available from the tool output. Scoped
`git diff --check` passed.

Native interaction, keyboard, and accessibility acceptance remain open.
