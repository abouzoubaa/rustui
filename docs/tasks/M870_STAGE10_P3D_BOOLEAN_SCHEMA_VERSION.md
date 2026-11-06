# M870 — Stage 10 P3D Boolean schema version

## Change

The P3D evidence validator now checks that `schema_version` has exact type
`int` before accepting value `1`. Python's `bool` is a subclass of `int`, and
JSON `true == 1`; the previous equality-only check therefore accepted a
Boolean version.

## Focused evidence

- An otherwise-valid `not_run` evidence record with `schema_version: true`
  reproduced the bug before the fix and is rejected afterward.
- The focused regression passed: 1 passed.
- Changed files: `scripts/validate_p3d_visual_evidence.py` and
  `scripts/tests/test_validate_p3d_visual_evidence.py`.

## Acceptance boundary

This closes one evidence-record type check. It does not establish candidate
provenance, candidate correctness, GUI/performance acceptance, review,
approval, or promotion.
