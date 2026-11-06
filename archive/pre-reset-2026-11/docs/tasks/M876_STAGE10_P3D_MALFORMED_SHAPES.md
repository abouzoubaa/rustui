# M876 — Stage 10 P3D malformed shapes

## Change

The P3D evidence validator now checks that the decoded root is a JSON object
before reading fields. It also checks state `status` values are strings before
set membership. Non-object roots and malformed statuses now produce
`InvalidEvidence` instead of uncaught `AttributeError` or `TypeError`.

## Focused evidence

- Regressions cover a list-valued JSON root and a list-valued state status.
- `python3 scripts/tests/test_validate_p3d_visual_evidence.py` passed: 11 tests.
- `python3 -m py_compile scripts/validate_p3d_visual_evidence.py scripts/tests/test_validate_p3d_visual_evidence.py` passed.
- Scoped `git diff --check` passed.

## Acceptance boundary

This hardens malformed-input handling for the P3D evidence record. It does not
establish candidate provenance, correctness, GUI/performance acceptance,
review, approval, or promotion.
