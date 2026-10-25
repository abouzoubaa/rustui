# M875 — Stage 10 exact P3D JSON value types

## Change

The P3D evidence validator now compares the fixed `workload` and
`capture_layout` records recursively using exact JSON types and values.
Previously, Python equality let integral floating-point values such as `32.0`
match the integer `32` in fixed evidence fields.

## Focused evidence

- A regression changes each fixed integer field to its integral-float
  equivalent and confirms validation rejects the record.
- `python3 scripts/tests/test_validate_p3d_visual_evidence.py` passed: 9 tests.
- `python3 -m py_compile scripts/validate_p3d_visual_evidence.py scripts/tests/test_validate_p3d_visual_evidence.py` passed.
- Scoped `git diff --check` passed.

## Acceptance boundary

This closes fixed-record JSON type matching. Malformed evidence-root/state
shape handling and candidate provenance, correctness, GUI/performance
acceptance, review, approval, and promotion remain open.
