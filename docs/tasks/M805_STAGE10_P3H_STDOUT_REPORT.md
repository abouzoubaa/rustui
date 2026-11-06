# M805 — Stage 10 P3H stdout report separation

P3H previously used a log with stderr merged into stdout as its machine-readable
report. A compiler warning could therefore invalidate an otherwise valid JSON
report. The pipeline now retains combined output in the evidence log and writes
stdout alone to the P3H report, using the existing exclusive output path.

Focused report-content and symlink-output regressions passed (2 tests):
`python3 scripts/tests/test_run_candidate_pipeline.py CandidatePipelineTests.test_p3h_report_keeps_stderr_out_of_machine_readable_stdout CandidatePipelineTests.test_pipeline_refuses_symlink_at_p3h_report_output`.
Candidate acceptance and promotion gates remain open.
