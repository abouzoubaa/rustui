# M850 — Stage 10 mixed-batch evidence count types

The quad-sort performance evidence validator previously accepted matching JSON
strings or booleans for `mixed_batch_audit.batches` and
`mixed_batch_audit.oracle_batches`. Python treats booleans as integers, and
matching strings could pass the equality check without representing numeric
counts.

The validator now requires both values to have exact type `int`, be
nonnegative, and match. The focused regression checks matching string and
boolean values and confirms both are rejected.

Focused evidence:

```text
python3 -m unittest scripts.tests.test_validate_quad_sort_performance.ValidateQuadSortPerformanceTests.test_rejects_non_integer_mixed_batch_counts
Ran 1 test
OK
```

This closes a measurement-record validation defect only. Candidate provenance,
build and correctness, native GUI and performance acceptance, review, approval,
and promotion remain open.
