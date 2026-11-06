# M1136 — follow RTL in horizontal SplitView

## Change

Horizontal SplitView panels now follow layout direction: left-to-right in
LTR and reversed in RTL. Vertical order remains unchanged. The panel ordering
is applied consistently to native group children and retained state, while
size reconciliation keeps each preference associated with its panel ID.

## Focused evidence

```text
./scripts/cargo-headstart test -p rustui horizontal_panels_follow_layout_direction_while_vertical_panels_do_not
1 passed
```

Scoped formatting and diff checks passed. Native pointer resizing, keyboard
interaction, and RTL GUI acceptance remain open.
