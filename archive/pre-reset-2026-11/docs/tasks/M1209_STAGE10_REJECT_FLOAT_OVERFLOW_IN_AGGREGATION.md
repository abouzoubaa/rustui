# M1209 — Stage 10 reject float overflow during evidence aggregation

The aggregation JSON reader now rejects numeric exponents that overflow
Python's finite float range, matching the adjacent evidence validator's
policy. The regression places raw `1e999` in both a hash-bound event and its
embedded pipeline result; aggregation now fails closed instead of treating the
two parsed infinities as consistent.

Verification: `python3 -m unittest
scripts.tests.test_aggregate_candidate_evidence.AggregateCandidateEvidenceTests.test_pipeline_event_rejects_float_overflow`
passed (1 test). No candidate was executed; review, approval, promotion, and
GUI/performance acceptance remain open.
