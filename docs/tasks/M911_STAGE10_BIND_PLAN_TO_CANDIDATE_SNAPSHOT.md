# M911 — Stage 10 / bind P3E plan to the candidate snapshot

The P3E plan declares `candidate.snapshot_sha256`; P3E validates its digest
syntax but treats the value as a declarative pin. The prepared candidate
manifest records the candidate package tree digest as
`snapshots.candidate.path_tree_sha256`. The M272 local-intake record
confirms these fields represent the same candidate tree. Before this change,
the intake runner did not compare them, allowing a hash-pinned plan to declare
a different candidate snapshot while matching the candidate ID, parent
manifest, and patch series.

`validate_intake()` now requires the plan's snapshot digest to equal the
prepared manifest's candidate `path_tree_sha256` before copying source bytes.
P3G still verifies that the prepared snapshot files match their manifest
inventory and path-tree digest; this change binds the plan pin to that
manifest declaration.

Focused checks passed:

```sh
python3 -m unittest \
  scripts.tests.test_run_candidate_pipeline.CandidatePipelineTests.test_patch_plan_snapshot_hash_must_match_prepared_candidate_tree \
  scripts.tests.test_run_candidate_pipeline.CandidatePipelineTests.test_patch_plan_parent_manifest_hash_must_match_prepared_manifest \
  scripts.tests.test_run_candidate_pipeline.CandidatePipelineTests.test_copies_snapshot_pins_hashes_and_runs_stages_in_order \
  scripts.tests.test_run_candidate_pipeline.CandidatePipelineTests.test_intake_accepts_schema_three_multi_file_patch_bindings
python3 -m py_compile scripts/run_candidate_pipeline.py scripts/tests/test_run_candidate_pipeline.py
```

This remains local metadata and patch-reproduction validation. Candidate build,
production correctness, native GUI, performance, and promotion gates remain
`not_run`; this change does not accept or promote a candidate.
