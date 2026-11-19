# Inactive experiments

`gpui-fast-bounded-wheel-offset-v1-fast` clamps a wheel offset before layout. Its intermediate-offset assertion failed without the patch and passed with it, but this did not resolve the grid candidates: Grid and List intentionally use `ScrollBounce` while their logical position remains clamped. This experiment is reversed and excluded from the active stack; it needs a confirmed rendering regression before activation. Its manifest and the generic candidate applier still prove exact apply/reverse.
