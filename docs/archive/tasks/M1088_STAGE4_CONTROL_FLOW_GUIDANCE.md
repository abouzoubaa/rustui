# M1088 — Stage 4 stack control-flow diagnostics

The stack macro now identifies `while` and `loop` alongside `for` and
`match` as unsupported control-flow children, and tells authors to expand
those branches before the stack macro.

The focused locked/offline `rustui-macros` diagnostic regression passed 1/1.
This clarifies the existing grammar boundary; it does not add control-flow
support.
