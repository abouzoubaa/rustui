# M909 — Stage 10 / bind P3E plan to its prepared manifest

The P3E plan schema carries `candidate.parent_manifest_sha256`. P3E validates
that it is a well-formed declared digest, but treats it as a declarative pin.
The local intake contract binds the approved P3E plan to the prepared source
candidate. Before this change, the intake runner checked the plan's candidate
ID and ordered patch declarations but did not verify that the plan's parent
manifest digest identified the exact prepared manifest bytes from intake.

`validate_intake()` now compares `candidate.parent_manifest_sha256` with the
SHA-256 of the exact source-manifest bytes it already parsed and matched to the
intake record. A stale or mismatched plan fails before candidate destination
creation or source copying. This binds the plan to the prepared manifest while
preserving P3E's local-only validation scope.

Focused checks passed:

```sh
python3 -m unittest \
  scripts.tests.test_run_candidate_pipeline.CandidatePipelineTests.test_patch_plan_parent_manifest_hash_must_match_prepared_manifest \
  scripts.tests.test_run_candidate_pipeline.CandidatePipelineTests.test_copies_snapshot_pins_hashes_and_runs_stages_in_order
python3 -m py_compile scripts/run_candidate_pipeline.py scripts/tests/test_run_candidate_pipeline.py
```

The pipeline still does not authenticate approval references or establish
candidate acceptance. Candidate build, production correctness, native GUI,
performance, and promotion gates remain `not_run`.
