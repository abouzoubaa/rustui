# M1200 — Stage 10 preserve reused destination descriptor

`_copy_directory_fd` could close a destination descriptor twice when copying
failed: the destination stream owned and closed the descriptor, then the outer
cleanup closed the stale integer again. If another open reused that number,
cleanup could close an unrelated descriptor.

The function now transfers descriptor ownership to the stream before copying
and closes the raw descriptor only if ownership was not transferred. A focused
regression reuses the just-closed descriptor number during copy failure and
confirms it remains open.

Verification: `python3 -m unittest
scripts.tests.test_apply_candidate_patch.CopyDirectoryDescriptorTests.test_copy_failure_preserves_reused_destination_descriptor`
passed (1 test). Candidate execution, independent review, approval, promotion,
and GUI/performance acceptance remain open.
