# M810: exact types in cache workload evidence

## Scope

Require cache calibration and comparison reports to preserve JSON value types.
Python equality treats booleans as integers, so plain equality could accept
`false` for an expected zero counter or `true` for an expected schema version
of one. This task hardens evidence validation; it does not claim native GUI or
performance acceptance.

## Change

The shared cache workload checker now compares JSON values recursively and
requires matching Python types for scalars, arrays, and objects. Both the
calibration and paired comparison paths use it.

## Evidence

- Regression: `test_rejects_boolean_where_integer_count_is_required`
- Regression: `test_workload_rejects_boolean_aliases_for_integer_evidence`

Native keyboard/focus and accessibility evidence, controlled performance
comparison, candidate review, approval, and promotion remain open.
