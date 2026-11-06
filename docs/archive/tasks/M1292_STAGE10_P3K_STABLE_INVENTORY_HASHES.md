# M1292 — Stage 10: reject P3K inventory changes during hashing

P3K `_inventory_fd` now binds each file digest to a stable file version. It
checks the named entry against the opened descriptor before reading, then
compares device, inode, size, modification time, and change time on the final
descriptor and named entry. It also verifies that the number of hashed bytes
matches the initial size. This brings the P3K inventory path in line with the
stable-read invariant already used by `PinnedDir.inventory` for P3G and applies
to pinned and non-pinned P3K inventory calls.

The deterministic regression overwrites the second 1 MiB chunk after the first
chunk has been read. It verifies the same inode and file size remain while the
inventory rejects the changed file. The pre-fix regression failed because
`_inventory_fd` returned a digest; after the fix, the focused test passed 1/1:

`python3 -m unittest scripts.tests.test_apply_candidate_patch.CopyDirectoryDescriptorTests.test_inventory_rejects_in_place_file_mutation_while_hashing -v`

This closes a local inventory-integrity gap only. It runs no patch or candidate
and does not establish candidate correctness, GUI/performance acceptance,
review, approval, or promotion.

M1293 later replaces the inline checks with delegation to `PinnedDir.inventory`,
which retains the same stable-read checks while making the walk iterative.
