# M907 — Stage 10 / bound candidate pipeline JSON inputs

The candidate intake runner's `load_json()` previously read intake, prepared
source-manifest, and P3E patch-plan files without a size limit. The P3G
snapshot validator already defines a 32 MiB manifest cap
(`MAX_MANIFEST_BYTES = 32 * 1024 * 1024`), so the runner now applies the same
32 MiB bound to each JSON input before parsing it.

`load_json()` reads at most `MAX_JSON_BYTES + 1` bytes. Inputs larger than
32 MiB fail with `PipelineError`; the extra byte distinguishes an exact-limit
file from an oversized one without loading the rest of the file. Inputs at the
limit continue through duplicate-key and UTF-8 validation as before.

Focused checks passed:

```sh
python3 -m unittest \
  scripts.tests.test_run_candidate_pipeline.CandidatePipelineTests.test_load_json_enforces_p3g_manifest_size_limit \
  scripts.tests.test_run_candidate_pipeline.CandidatePipelineTests.test_copies_snapshot_pins_hashes_and_runs_stages_in_order
python3 -m py_compile scripts/run_candidate_pipeline.py scripts/tests/test_run_candidate_pipeline.py
```

This bounds JSON parsing for local intake only. The pipeline remains a partial
metadata and patch-reproduction chain; candidate build, production correctness,
native GUI, performance, and promotion gates remain `not_run`. No candidate
approval or acceptance is established by this change.
