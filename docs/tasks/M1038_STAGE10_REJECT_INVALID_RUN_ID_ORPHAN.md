# M1038 — Reject unsafe P3K run IDs before candidate creation

## Problem

`run_pipeline` created and pinned the candidate directory before validating an
optional P3K `run_id`. An unsafe identifier raised `PipelineError` outside the
evidence-recording `try` block and left an orphaned candidate directory with no
pipeline failure record.

## Change

Validate `run_id` after the read-only intake checks and before creating the
candidate directory.

## Focused evidence

`python3 scripts/tests/test_run_candidate_pipeline.py CandidatePipelineTests.test_invalid_run_id_is_rejected_before_candidate_creation`

Result: 1 passed, 0 failed. The regression confirms that an unsafe run ID is
rejected and the destination candidate directory is not created.

The existing Quad V2 pipeline record is partial, with P3E–P3K/P3F steps, M274
build, and M276 correctness recorded as passed. GUI/performance acceptance,
independent evidence review, approval, and promotion remain open.
