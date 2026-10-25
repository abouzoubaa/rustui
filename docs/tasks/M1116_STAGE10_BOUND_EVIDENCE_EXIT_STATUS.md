# M1116 — Stage 10 / constrain candidate process exit statuses

P3F candidate evidence validation accepted arbitrary nonnegative exit statuses,
including 256, although process exit statuses are limited to 0–255 and the
build-provenance parser already enforces that contract.

The validator now rejects values outside 0–255. The focused regression checks
that 255 remains valid and 256 is rejected. The focused
`scripts/tests/test_validate_candidate_evidence.py` suite passed 20 tests.

No candidate was run or approved. Candidate build/correctness, GUI/performance
evidence, independent review, approval, and promotion remain open.
