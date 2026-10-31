# M784 — Atomically rewrite failed P3F evidence

## Scope

When the P3F evidence validator rejects its record, the pipeline updates `exit_status` to reflect the failure. Reopening that path with truncation could follow a symlink substituted after validation.

## Change

Serialize the failed record to a uniquely named, exclusively created temporary file, flush it, then atomically replace the report path. A symlink at the destination is replaced as a directory entry; its target is not opened or modified.

## Evidence

- `python3 -m unittest scripts.tests.test_run_candidate_pipeline.CandidatePipelineTests.test_atomic_p3f_record_rewrite_replaces_symlink_without_following_it scripts.tests.test_run_candidate_pipeline.CandidatePipelineTests.test_pipeline_refuses_symlink_at_p3h_report_output` — 2 passed.
- `python3 -m py_compile scripts/run_candidate_pipeline.py scripts/tests/test_run_candidate_pipeline.py` — passed.
- Scoped `git diff --check` — passed.

Candidate provenance, isolated build/correctness, GUI visual acceptance, performance comparison, approval, and promotion remain separate open gates.
