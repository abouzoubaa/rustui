# M1036 — Verify opened source directory identity

## Scope

`PinnedDir.copy_tree_from` checked a child directory with a no-follow stat,
then opened it and recursed without confirming that the descriptor referred to
the same directory. A concurrent rename could replace the entry with a
different real directory from outside the pinned source subtree; no-follow
flags only reject symlinks.

## Change

After opening each child directory, copying now verifies that the descriptor
still names a directory with the same device and inode observed by the initial
no-follow stat. It rejects a changed identity before descending into the
directory.

## Regression

The regression replaces the source entry with a real directory moved from
outside the source tree immediately before open. It verifies that copying
rejects the replacement and does not copy the outside payload.

## Verification

`python3 scripts/tests/test_p3_candidate_fs.py`

Result: 8 passed, 0 failed.

No candidate was run. Candidate build/correctness, GUI/performance evidence,
review, approval, and promotion remain separate Stage 10 gates.
