# Stage 4 — generic constructor boundary in stack macros

The conditional-stack grammar retains ordinary Rust constructor expressions
and adjacent DSL children (`docs/tasks/CONDITIONAL_STACKS.md`). The child parser
previously stopped at the first parenthesis or brace group, even when that group
belonged to a generic argument. For example,
`make_view::<fn() -> u8>() Text("sibling")` was split inside the function-pointer
type and failed to parse.

`parse_view_child` now tracks nested angle-bracket arguments while looking for
the constructor delimiter. A `>` in a function return arrow does not close a
generic argument. The focused regression first reproduced the parse error and
then passed with the fix.

Evidence: `target/headstart/evidence/stage4-generic-constructor-boundary-20261023.log`.
The complete focused view-builder suite passed 17/17. This is parser/expansion
evidence; it does not establish Playground compilation or native acceptance.
