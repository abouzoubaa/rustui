# M982 — Stage 10 / require integer schema versions in performance evidence

The quad-sort performance evidence validator now requires exact Python
`int` type for `schema_version`. This rejects JSON `2.0`, which previously
compared equal to integer `2`.

Focused check: `ValidateQuadSortPerformanceTests.test_rejects_non_integer_raw_schema_version`
passed (1/1).

Candidate execution, review, approval, and promotion remain open.
