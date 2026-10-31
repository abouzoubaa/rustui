# M1198 — Stage 10 close rename parent descriptors

## Scope

Release descriptors owned by `PinnedDir.rename` when destination validation
fails before the rename operation.

## Change

The destination parent descriptor is now initialized before validation and
closed only when acquired; the source parent descriptor is always closed in the
same `finally` path. Invalid destination input no longer leaks the opened
source-parent descriptor.

## Verification

Focused regression command:

```sh
python3 scripts/tests/test_p3_candidate_fs.py CandidateFilesystemTests.test_rename_invalid_destination_closes_source_parent
```

Result: 1 test passed. No candidate execution or promotion was performed.
