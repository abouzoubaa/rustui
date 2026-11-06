# M1206 — Stage 10 record candidate pin failures

After exclusive candidate-directory creation, the pipeline now records a
failure beside that directory if it cannot pin the new candidate. It writes
through the already-pinned candidates parent, does not enter candidate steps,
and re-raises the original pin error. A secondary evidence-write failure is
attached as an error note without masking that original error.

Verification: `python3 -m unittest
scripts.tests.test_run_candidate_pipeline.CandidatePipelineTests.test_candidate_pin_failure_keeps_failure_evidence_in_pinned_parent`
passed (1 test). No candidate was executed; review, approval, promotion, and
GUI/performance acceptance remain open.
