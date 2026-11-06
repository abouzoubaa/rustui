# M1062 — Stage 10 / close temporary evidence-directory owners

## Problem

The Stage 10 candidate evidence validator opened each path component through a
temporary owning `PinnedDir`. `_open_child` returned the nested directory's
file descriptor without closing that owner, leaking one descriptor per
directory component processed.

## Change

For directory opens, `_open_child` now duplicates the child's descriptor and
closes the temporary `PinnedDir` in a `finally` block. The returned descriptor
remains valid for the caller, while the temporary owner releases its handle.

## Focused evidence

`PYTHONPATH=scripts/tests python3 -m unittest test_validate_candidate_evidence.ValidateCandidateEvidenceTests.test_valid_record_reports_identity_and_gate_states test_validate_candidate_evidence.ValidateCandidateEvidenceTests.test_rejects_symlink_swap_after_parent_directory_is_opened test_validate_candidate_evidence.ValidateCandidateEvidenceTests.test_directory_component_open_closes_temporary_pinned_owner`

Result: 3 passed. The new test tracks opened child owners and verifies each is
closed after the evidence file is read. No candidate was executed or approved,
and no patch was promoted.
