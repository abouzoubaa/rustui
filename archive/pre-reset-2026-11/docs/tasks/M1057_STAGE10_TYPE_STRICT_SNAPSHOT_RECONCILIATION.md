# M1057 — Stage 10 / type-strict snapshot reconciliation

Candidate evidence aggregation compared candidate and source-hash snapshot file
maps with Python `==`. JSON booleans and integers compare equal in Python
(`True == 1`), so a malformed source map could be treated as matching the
candidate map.

Aggregation now uses recursive JSON type-strict equality for these maps. The
focused regression verifies that a Boolean file-map value does not match an
integer value.

The targeted check passed:

```text
python3 -m unittest scripts.tests.test_aggregate_candidate_evidence.AggregateCandidateEvidenceTests.test_snapshot_file_map_comparison_distinguishes_boolean_from_integer
```

This establishes type-strict reconciliation only. Candidate acceptance,
independent build/correctness/GUI/performance evidence, and patch approval or
promotion remain separate gates.
