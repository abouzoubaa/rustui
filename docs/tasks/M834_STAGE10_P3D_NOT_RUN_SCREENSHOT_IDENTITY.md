# M834: Stage 10 P3D not-run screenshot identity

## Scope

The P3D record distinguishes an unattempted state (`not_run`) from failed,
inconclusive, and captured attempts. A `not_run` state cannot identify a
screenshot file: such a path claims capture evidence for a state recorded as
never attempted.

## Change

The P3D visual evidence validator now rejects a screenshot reference on a
`not_run` state. Failed and inconclusive attempts retain their existing
optional screenshot behavior and distinct-file checks.

## Evidence

- `python3 scripts/tests/test_validate_p3d_visual_evidence.py -k not_run` —
  focused regression passed.

This structural consistency check does not inspect screenshot pixels or close
native visual, input, accessibility, performance, review, approval, or
promotion gates.
