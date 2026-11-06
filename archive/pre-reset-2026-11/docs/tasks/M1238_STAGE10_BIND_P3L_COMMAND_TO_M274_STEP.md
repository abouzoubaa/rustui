# M1238 — Bind P3L command argv to the M274 event step

P3L now requires a candidate build event to identify the approved
`cargo-headstart --workspace --all-targets --locked --offline` invocation and
the correct `check` or `build` step. The regression changes the candidate-build
command to `check`, refreshes its hash, and confirms aggregation rejects the
event and leaves build evidence `not_run`. The focused regression passed
(1/1); valid linked M274/M276 reconciliation also passed (1/1), along with
`py_compile` and scoped whitespace checks.

This validates local record consistency only. It does not prove the command
actually ran or establish candidate build/correctness, GUI/performance,
independent review, approval, or promotion.
