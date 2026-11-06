# M1196 — Stage 10 close source descriptors on copy failure

## Scope

Ensure `_copy_source` releases descriptors it owns if destination setup fails
after opening the source.

## Change

The source and destination `PinnedDir` handles are now registered with the
existing `ExitStack` as soon as ownership begins. An error while creating the
destination therefore closes the already-open source handle and owned root
duplicates, while leaving borrowed caller descriptors open.

## Verification

Focused regression command:

```sh
python3 scripts/tests/test_run_candidate_pipeline.py CandidatePipelineTests.test_source_copy_closes_owned_descriptors_when_destination_creation_fails
```

Result: 1 test passed. Full candidate execution, independent review, approval,
promotion, and GUI/performance acceptance remain open.
