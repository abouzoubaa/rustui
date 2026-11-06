# M1226 — Require P3G path-source tree hashes

P3G snapshot validation now rejects snapshot records that omit
`path_tree_sha256`. Without this field, a record could report a valid file
inventory without binding the path-source package tree expected by the P3G
contract.

The focused regression removes the candidate record's field and verifies that
validation rejects it.

```sh
python -m unittest -q \
  scripts.tests.test_validate_candidate_snapshot.ValidateCandidateSnapshotTests.test_rejects_missing_path_tree_hash
```

Result: passed (1/1). Candidate execution and all acceptance/promotion gates
remain open.
