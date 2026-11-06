# M1199 — Stage 10 reject source parent traversal

## Scope

Reject manifest source paths containing parent components before canonicalizing
them for snapshot validation.

## Change

`_resolve_source_path` now rejects `..` components before `abspath` can erase
them. A path such as `candidate/source/../source` is therefore invalid even
though it resolves back inside the candidate root.

## Verification

Focused regression command:

```sh
python3 scripts/tests/test_validate_candidate_snapshot.py ValidateCandidateSnapshotTests.test_rejects_source_path_with_parent_traversal_even_when_it_resolves_in_root
```

Result: 1 test passed. Candidate execution, independent review, approval,
promotion, and GUI/performance acceptance remain open.
