# M1022 — Stage 10: bind visual evidence to its workload plan

P3D now requires the referenced visual-evidence plan to specify the same
`32-leaf` workload as the evidence manifest, including 32 leaves and 12 text
lines. The focused mismatch regression failed before the validator change and
passed afterward (1/1); the scoped whitespace check passed.

This closes a workload-consistency false pass only. It does not provide paired
native screenshots or prove GUI, accessibility, drag, frame, or GPU acceptance.
