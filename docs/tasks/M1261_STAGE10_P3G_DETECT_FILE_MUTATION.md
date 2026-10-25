# M1261 — Stage 10: detect P3G in-place file mutation

`PinnedDir.inventory` now checks device, inode, size, modification time, and
change time across the initial directory entry, opened descriptor, final
descriptor, and final named path. This rejects a same-inode write that occurs
while the file is being hashed.

The deterministic regression mutates the file after the first read block,
confirms inventory rejects it, and verifies the inode stayed the same. The
focused test passed 1/1.

This closes a local snapshot-integrity gap only. Candidate execution,
correctness, GUI/performance acceptance, independent review, approval, and
promotion remain open.
