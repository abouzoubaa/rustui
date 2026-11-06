# M839 — Stage 10 P3G candidate manifest hard-link rejection

## Defect

The paired snapshot validator rejected symlinked manifest paths but accepted a regular `manifest.json` with multiple hard links. An alias outside the candidate root could therefore mutate the manifest that defines the snapshot inventories.

## Change

`_load_manifest` now requires the opened manifest descriptor to have one hard link, checking both before and after the bounded read. The regression hard-links the candidate manifest outside its root and requires validation to fail.

## Validation

- `python3 -m unittest scripts.tests.test_validate_candidate_snapshot.ValidateCandidateSnapshotTests.test_rejects_manifest_hard_linked_outside_candidate_root` — passed (1/1).

This validates local manifest and snapshot integrity only; it does not establish candidate provenance, approval, build, GUI, performance, or roadmap acceptance.
