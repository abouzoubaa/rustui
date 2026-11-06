# M906 — Stage 10 / bound candidate pipeline source inventory

The local candidate intake runner inventoried and SHA-256 hashed prepared
baseline and candidate source trees before invoking P3G. That inventory had no
entry or byte limits, so an oversized source tree could consume unbounded work
before P3G's bounded snapshot validator rejected it. P3G defines limits of
100,000 filesystem entries and 4 GiB of regular-file content per snapshot.

`run_candidate_pipeline.inventory()` now reuses P3G's descriptor-relative,
no-follow inventory implementation, including both limits and its source-file
integrity checks. The pipeline resolves the existing root before inventory so
macOS `/var` and `/private/var` path spellings work with the descriptor walk.
P3G inventory errors are surfaced as `PipelineError`; filesystem errors are
reported as inventory failures. This bounds repeated source hashing during
intake as well as the later standalone P3G check.

Focused checks passed:

```sh
python3 -m unittest \
  scripts.tests.test_run_candidate_pipeline.CandidatePipelineTests.test_pipeline_inventory_enforces_snapshot_entry_budget \
  scripts.tests.test_run_candidate_pipeline.CandidatePipelineTests.test_pipeline_inventory_enforces_snapshot_byte_budget
python3 -m unittest \
  scripts.tests.test_run_candidate_pipeline.CandidatePipelineTests.test_copies_snapshot_pins_hashes_and_runs_stages_in_order
python3 -m py_compile scripts/run_candidate_pipeline.py scripts/tests/test_run_candidate_pipeline.py
```

The candidate pipeline remains a local metadata and patch-reproduction chain.
Build, production correctness, native GUI, performance, and promotion gates
remain `not_run`; this change does not establish candidate approval or
acceptance.
