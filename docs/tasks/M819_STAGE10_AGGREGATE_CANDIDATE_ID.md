# M819 Stage 10: validate aggregated candidate identity

The evidence aggregator now rejects a pipeline result whose `candidate_id`
does not match the shared safe-identifier format or contains `..`. The earlier
pipeline stages already validate candidate IDs; this closes the aggregator's
independent input boundary.

Focused regression:

```text
python3 -m unittest scripts.tests.test_aggregate_candidate_evidence.AggregateCandidateEvidenceTests.test_pipeline_candidate_id_must_match_safe_identifier_contract
Ran 1 test; OK
```

The agent also ran the full aggregator suite: 45 tests passed. This is
evidence-integrity validation only. Provenance, candidate build and correctness,
GUI/performance acceptance, review, approval, and promotion remain open.
