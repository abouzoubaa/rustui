# M977 — Stage 4 / recognize parenthesized macro view constructors

The view macro now classifies parenthesized constructor expressions as
constructors. It retains the expression as a child and keeps outer `cfg`
attributes on the generated call.

Focused check: `parenthesized_constructor_remains_a_child_and_keeps_its_cfg_boundary`
passed (1/1).
