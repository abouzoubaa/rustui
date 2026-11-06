# M1253 — Stage 10 P3G inventory avoids depth-dependent file descriptor use

The iterative P3G inventory previously retained one open directory descriptor
for every active depth. A valid snapshot could therefore fail at the process
file-descriptor limit before reaching P3G's existing 100,000-entry and 4 GiB
budgets.

The inventory now retains directory names, identities, and before-metadata in
its explicit stack. It keeps one active directory descriptor, descends into
children, then ascends with descriptor-relative `..` opens that verify the
recorded parent and child identities. This avoids both depth-proportional open
descriptors and reopening the full ancestor chain for every entry.

A regression builds 1,100 nested directories, lowers `RLIMIT_NOFILE` below
the tree depth while retaining descriptor headroom, checks the resulting file
hash, then restores the limit for cleanup. The focused regression passed twice;
the latest 1/1 run completed in 5.9 seconds:

```sh
python3 scripts/tests/test_validate_candidate_snapshot.py -k depth
```

The production P3G pipeline also inventories pinned source and destination
trees through `PinnedDir.inventory`. That entry point now uses the same
iterative, identity-checked traversal, so the pipeline cannot exhaust file
descriptors before reaching `copy_tree_from`. Its focused inventory tests
passed 5/5, including a 1,100-level tree under a low `RLIMIT_NOFILE` limit.
The `_copy_source` integration regression also passed 1/1, covering source
inventory, copy, post-copy source inventory, destination inventory, path
hashes, and the copied leaf.

This addresses local inventory recursion and descriptor use only. Candidate
execution, provenance, correctness, GUI/performance acceptance, review,
approval, and promotion remain separate gates.
