# M783 — Refuse symlinked P3H report outputs

## Scope

The Stage 10 candidate pipeline stores the P3H baseline and candidate validator reports in the candidate evidence directory. A pre-existing symlink at one of those output paths could redirect the pipeline's write to a different file.

## Change

Create each P3H report with exclusive file creation (`open("x")`). Existing paths, including symlinks, are rejected rather than followed. The regression plants a symlink at the baseline report path and verifies that the pipeline raises `FileExistsError` without changing the symlink target.

## Evidence

`python3 -m unittest scripts.tests.test_run_candidate_pipeline.CandidatePipelineTests.test_pipeline_refuses_symlink_at_p3h_report_output` passed (1 test).

This proves the local output-path integrity case only. Candidate provenance, isolated build/correctness, GUI visual acceptance, performance comparison, approval, and promotion remain separate open gates.
