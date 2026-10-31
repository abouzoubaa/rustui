# M835 — Stage 10 / reject overlapping P3G snapshot roots

P3G rejected identical baseline and candidate roots, but allowed one root to be
an ancestor of the other. Such snapshots share a mutable directory subtree and
are not independent source snapshots. P3G now rejects ancestor/descendant root
pairs. A regression refreshes both inventories and package-tree hashes for a
nested pair, then verifies that validation rejects the overlap.

Focused validation: `python3 scripts/tests/test_validate_candidate_snapshot.py`.
This is source snapshot integrity evidence only; candidate build, correctness,
GUI, performance, review, approval, and promotion remain separate gates.
