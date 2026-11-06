# M920 — Stage 8 / propagate Tree arrow keys when no enabled row is available

The Tree captures Up and Down to select and scroll to the next enabled entry.
When there is no enabled entry, the key must remain available to an ancestor
handler. The capture handlers now stop propagation only when an enabled target
exists. A focused regression covers both the available and unavailable cases.

The focused test passed 1/1 (790 filtered), recorded at
`target/headstart/evidence/tree-arrow-consumption-contract-followup-20261023.log`.
Native keyboard routing and broader accessibility acceptance remain open.
