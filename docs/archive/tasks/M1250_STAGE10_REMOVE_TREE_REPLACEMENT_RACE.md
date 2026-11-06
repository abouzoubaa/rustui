# M1250 — Stage 10 P3 cleanup directory replacement race

`PinnedDir.remove_tree` now captures each directory entry's `(st_dev, st_ino)`
identity before opening it and verifies the opened descriptor has the same
identity before recursively deleting contents. This prevents cleanup from
recursing into a replacement directory swapped in after the entry was inspected.

A focused regression simulates replacing `victim/nested` between inspection and
open and verifies that both the detached original and replacement contents
survive. `python3 scripts/tests/test_p3_candidate_fs.py` passed 12/12.

This is local P3 cleanup hardening. Candidate execution, current-source
GUI/performance evidence, independent review, approval, and promotion remain
separate open gates.
