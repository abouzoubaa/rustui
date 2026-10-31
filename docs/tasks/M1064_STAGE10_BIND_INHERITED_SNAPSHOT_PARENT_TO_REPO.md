# M1064 — Stage 10 / bind inherited snapshot parent to the repository

## Problem

The candidate snapshot validator checked that the supplied root argument named
`target/headstart/candidates/<candidate-id>` and that the inherited candidate
directory matched its inherited parent FD. It did not establish that the
inherited parent itself was the repository's `target/headstart/candidates`
directory.

## Change

The validator now opens `target/headstart/candidates` relative to the inherited
repository FD and compares its directory identity with the inherited candidate
parent FD before opening the candidate ID. An external directory can no longer
masquerade as the repository candidate parent.

## Focused evidence

`python3 scripts/tests/test_validate_candidate_snapshot.py ValidateCandidateSnapshotTests.test_inherited_candidate_root_uses_candidate_fd_and_checks_parent_identity ValidateCandidateSnapshotTests.test_inherited_candidate_root_rejects_parent_outside_repository_candidates`

Result: 2 passed. The first test covers a valid repository-relative parent and
candidate identity; the second rejects an external parent FD. No candidate was
executed or approved, and no patch was promoted.
