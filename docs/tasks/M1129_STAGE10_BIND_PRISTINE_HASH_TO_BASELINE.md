# M1129 — bind pristine snapshot hash to baseline source

## Change

`scripts/validate_candidate_snapshot.py` now compares a declared
`pristine_path_tree_sha256` with the actual captured baseline path-source tree
hash. A pristine hash also requires exactly one path-source package. This
prevents a forged candidate pristine hash from passing a shape-only check.

## Focused evidence

The direct regression sets the candidate pristine hash to a forged digest and
confirms that snapshot validation rejects it:

```text
python3 -m unittest scripts.tests.test_validate_candidate_snapshot.ValidateCandidateSnapshotTests.test_rejects_pristine_path_tree_hash_that_differs_from_baseline
Ran 1 test — OK
```

This validates snapshot metadata consistency only. No candidate was run; P3
candidate execution, independent review, approval, and promotion remain open.
