# M867 — Stage 10 / reconcile candidate pipeline commands with gate steps

## Contract

Hash-bound candidate-pipeline event records must not claim a command unrelated
to the declared gate step.

## Implementation

The aggregator maps command-backed P3E–P3K steps to their expected validator
scripts and rejects non-string or mismatched command arguments. Internal and
unknown steps cannot claim a command-backed validator event.

## Evidence

- `scripts.tests.test_aggregate_candidate_evidence.AggregateCandidateEvidenceTests.test_command_must_match_event_step_validator` passed 1/1.
- `python3 -m unittest scripts.tests.test_aggregate_candidate_evidence` passed 46/46.

This is local evidence reconciliation only. Candidate builds, correctness,
GUI/performance acceptance, review, approval, and promotion remain open.
