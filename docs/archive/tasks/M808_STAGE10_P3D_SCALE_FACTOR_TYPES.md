# M808 — Stage 10 P3D scale-factor types

The P3D visual-evidence validator previously coerced values with `float()`,
which accepted numeric strings. It now accepts only integer/float values,
continues to reject booleans, and converts overflow into `InvalidEvidence`.

The focused validator suite passed (2 tests):
`python3 scripts/tests/test_validate_p3d_visual_evidence.py`.
This is input validation for evidence artifacts, not native visual or
performance acceptance.
