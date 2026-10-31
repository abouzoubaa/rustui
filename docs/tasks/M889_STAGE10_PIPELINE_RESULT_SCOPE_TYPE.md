# M889 — Stage 10 / require pipeline-result scope strings

The evidence aggregator now rejects pipeline results whose `scope` field is
not a string. Previously, an object-valued scope could pass through to a
`consistent` aggregate report even though the pipeline producer records a
string. The focused regression passed:

```sh
python3 scripts/tests/test_aggregate_candidate_evidence.py AggregateCandidateEvidenceTests.test_pipeline_scope_must_be_a_string
```

This closes one local aggregation-schema gap. Candidate provenance, build and
correctness, GUI/performance acceptance, review, approval, and promotion remain
open.
