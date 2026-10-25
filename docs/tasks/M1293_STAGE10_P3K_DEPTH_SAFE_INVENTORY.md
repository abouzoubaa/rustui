# M1293 — Stage 10: make P3K inventory depth-safe

P3K `_inventory_fd` recursively opened one descriptor per nested directory.
The existing entry-count cap did not bound traversal depth, so inventorying a
deep package exhausted descriptors even though P3K copying already handled
the same tree iteratively (M1260).

`_inventory_fd` now reuses `PinnedDir.inventory`, which walks iteratively with
bounded active descriptors and retains the configured entry/byte limits and
stable file/directory identity checks. The helper's hard-link rejection stays
enabled by default; P3K opts out to preserve its prior accepted-input behavior.
This also retains M1292's file-mutation rejection through the shared inventory
implementation.

The focused regression builds a 1,100-level directory chain and inventories it
under a 32-descriptor soft limit. Before the fix, it failed with `EMFILE`; after
the fix, the deep-inventory and M1292 same-inode mutation regressions passed.
A hard-link regression also verifies that both P3K inventory entry paths still
accept a hard-linked file while the shared helper rejects it by default. The
focused checks passed (3/3):

`python3 -m unittest scripts.tests.test_apply_candidate_patch.CopyDirectoryDescriptorTests.test_inventory_rejects_in_place_file_mutation_while_hashing scripts.tests.test_apply_candidate_patch.CopyDirectoryDescriptorTests.test_inventory_handles_deep_directory_chain_with_bounded_descriptors scripts.tests.test_apply_candidate_patch.CopyDirectoryDescriptorTests.test_inventory_preserves_p3k_hardlink_behavior -v`

This closes a local inventory traversal gap only. It runs no patch or
candidate and does not establish candidate correctness, GUI/performance
acceptance, review, approval, or promotion.
