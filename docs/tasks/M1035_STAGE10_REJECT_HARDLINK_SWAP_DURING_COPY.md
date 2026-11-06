# M1035 — Reject hard-link swaps during source copying

## Scope

`PinnedDir.copy_tree_from` rejected hard-linked files using a no-follow stat,
then opened the pathname and copied from the resulting descriptor. Replacing a
regular source file with a hard link between those operations could bypass the
link-count check.

## Change

After opening a source file, the copy now verifies that the descriptor is a
regular file, has exactly one link, and has the same device/inode as the
initial no-follow stat. Otherwise it rejects the source before creating a
destination file.

## Regression

The test replaces a source entry with a hard link to an outside file just
before the open. It asserts that the copy raises `CandidateFSError`, does not
create the destination entry, and leaves the outside file's contents intact.

## Verification

`python3 scripts/tests/test_p3_candidate_fs.py`

Result: 7 passed, 0 failed.

No candidate was run. Candidate build/correctness, GUI/performance evidence,
review, approval, and promotion remain separate Stage 10 gates.
