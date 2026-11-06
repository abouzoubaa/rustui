# M1104 — Stage 8: constrain divider range by adjacent panel capacity

GPUI expands a panel by taking space from panels to its right, but shrinking a
panel transfers space only to the immediately adjacent right panel. The
`SplitView` lower divider bound now accounts for that panel's remaining
maximum-size capacity instead of summing slack from all later panels.

The focused three-panel regression passed with
`./scripts/cargo-headstart test -p rustui divider_range_respects_container_and_sibling_constraints`
(1 passed, 839 filtered). Native visual, keyboard, and accessibility
acceptance remain open.
