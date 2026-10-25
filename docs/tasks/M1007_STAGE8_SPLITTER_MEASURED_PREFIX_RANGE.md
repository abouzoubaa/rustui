# M1007 — constrain SplitView range by measured prefix panels

The divider range calculation used configured capacities for every panel to
the right of the handle, but GPUI's resize handle can change only the current
panel and panels to its right. Panels before the handle keep their measured
sizes. Subtracting the measured prefix prevents infeasible minimums and
incorrect splitter ARIA ranges.

Added a three-panel regression where the measured prefix is 300 points while
its configured maximum is 500 points. Corrected the existing three-panel
expected minimum from 170 to 210 points.

Focused test passed 1/1:

```text
./scripts/cargo-headstart test -p rustui divider_range_respects_container_and_sibling_constraints --lib
```

Native drag, keyboard, screen-reader, and visual acceptance remain open.
