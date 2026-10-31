# M732 — Reject hard-linked files in P3G snapshots

P3G already rejected a regular file whose inode appeared in both baseline and
candidate snapshots. It still accepted a snapshot file with another hard link
outside the paired trees. A later write through that external path could mutate
a snapshot after its digest was checked.

The inventory now requires each opened regular snapshot file to have exactly one
hard link. The paired-inode check runs first so the existing baseline/candidate
specific diagnostic remains intact. A focused regression hard-links a baseline
snapshot file to a path outside both snapshots and confirms the validator
rejects it.

Validation: `python3 scripts/tests/test_validate_candidate_snapshot.py`
passed 17/17; `python3 -m py_compile scripts/validate_candidate_snapshot.py
scripts/tests/test_validate_candidate_snapshot.py` passed; scoped `git diff
--check` reported no whitespace errors for the validator and test. The validator
and test are currently untracked in this shared worktree, so Git's ordinary diff
contains no patch for those existing files. No files were staged or committed.
