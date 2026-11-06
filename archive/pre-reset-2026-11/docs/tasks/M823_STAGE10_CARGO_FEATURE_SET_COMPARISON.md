# M823 — Stage 10 Cargo feature selection comparison

## Finding

P3I/P3J compared the `request.features` arrays by list order, although Cargo feature selection is set-like. Equal baseline and candidate feature selections could therefore be rejected when their names appeared in a different order.

## Change

The source-report comparator canonicalizes only `request.features` as a sorted set before comparing the selection. Other request fields remain exact.

## Evidence

- Regression: `scripts/tests/test_compare_candidate_sources.py::test_feature_selection_order_does_not_change_selection_identity`
- Command: `python3 scripts/tests/test_compare_candidate_sources.py`
- Result: 23 tests passed.
- Scoped `git diff --check` passed.

## Remaining Stage 10 gates

This fixes selection comparison semantics only. Candidate provenance, build and correctness evidence, GUI and performance evidence, review, approval, and promotion remain open.
