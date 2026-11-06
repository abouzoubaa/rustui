# M1260 — Stage 10: bound P3K package-copy traversal depth

`_copy_directory_fd` now copies package trees iteratively, retaining a
constant number of active source and destination directory descriptors.
It verifies source and destination identities while ascending, retains the
existing no-follow, directory metadata, mode, and entry/byte checks, and
closes a partially opened parent descriptor if the paired open fails.

The 1,100-level copy regression ran with a low file-descriptor limit and
verified the copied leaf. The focused test passed 1/1 after the final cleanup
guard change.

This closes a disposable package-copy resource gap only. Candidate execution,
correctness, GUI/performance acceptance, independent review, approval, and
promotion remain open.
