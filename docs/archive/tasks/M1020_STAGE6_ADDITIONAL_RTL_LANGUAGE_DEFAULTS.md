# M1020 — Stage 6: add confirmed RTL language defaults

Locale direction now defaults `ars`, `ks`, `lrc`, `mid`, `mzn`, and `rhg` to
RTL. A focused regression checks all six defaults and verifies that explicit
`Latn` keeps each locale LTR. The focused test passed 1/1; rustfmt and the
scoped whitespace check passed.

This extends the heuristic language map only. Broader locale/script coverage,
native RTL layout, screen-reader, and cross-platform acceptance remain open.
