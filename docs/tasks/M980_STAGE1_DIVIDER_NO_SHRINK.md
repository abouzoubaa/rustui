# M980 — Stage 1 / keep Divider lines from shrinking

Horizontal and vertical Divider line elements now set `flex_shrink_0()`,
keeping their declared one-pixel thickness in constrained stacks.

Focused check: `components::basic::divider::tests::horizontal_and_vertical_lines_do_not_shrink`
passed (1/1).
