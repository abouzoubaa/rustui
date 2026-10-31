# M851 — Stage 8 Tree arrow navigation without a selection

When no row was selected, the Tree keyboard helper treated row zero as the
current row and advanced before selecting. This made Down skip the first
enabled row.

The helper now starts Down at the first enabled row and Up at the last enabled
row. The regression covers both directions, disabled entries, wraparound, and
the all-disabled case.

Focused evidence:

```text
./scripts/cargo-headstart test -p rustui tree::tests::tree_arrow_navigation_skips_disabled_entries_and_wraps --lib --locked --offline
1 passed; 772 filtered out
```

Scoped rustfmt and `git diff --check` passed. This is source-level keyboard
logic evidence; native keyboard, focus, screen-reader, and cross-platform
acceptance remain open.
