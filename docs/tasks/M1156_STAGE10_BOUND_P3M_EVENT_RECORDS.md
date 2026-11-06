# M1156 — Stage 10 / bound P3M follow-up event records

P3M read hash-bound M274 and M276 follow-up event JSON without a size limit.
Unlike pipeline event records, an oversized but correctly hashed follow-up
event was therefore fully allocated and could still close a gate.

The aggregator now applies its existing 1 MiB record limit when reading both
M274 build events and the M276 correctness success event. Focused regressions
rehash oversized records and confirm aggregation fails closed with every gate
left `not_run`.

Focused validation passed:

- `python3 -m unittest scripts.tests.test_aggregate_candidate_evidence.AggregateCandidateEvidenceTests.test_m274_event_record_size_is_bounded scripts.tests.test_aggregate_candidate_evidence.AggregateCandidateEvidenceTests.test_m276_success_event_record_size_is_bounded` (2 tests).

This checks input bounds and fail-closed gate reporting. It does not accept a
candidate or close GUI, performance, review, approval, or promotion gates.
