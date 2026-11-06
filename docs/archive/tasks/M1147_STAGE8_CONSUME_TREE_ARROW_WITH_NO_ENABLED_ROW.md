# M1147 — Stage 8: consume Tree arrows with no enabled destination

TreeView consumes Up/Down actions even when there is no enabled row to move to,
preventing GPUI's fallback handler from selecting a disabled raw index.

The focused regression `tree_arrow_key_is_consumed_even_when_no_enabled_entry_is_available`
passed 1/1. Broader native keyboard and screen-reader acceptance remain open.
