# M1254 — Stage 10 P3G intake copy is depth-safe

M1253 made P3G inventory safe for deep snapshots, but intake still copied a
validated source tree with recursive calls that held source and destination
directory descriptors at every depth. A deep tree could therefore fail during
copy after passing inventory.

`PinnedDir.copy_tree_from` now uses an explicit stack and keeps only the
current source/destination directory descriptors. It ascends through
descriptor-relative `..` opens, verifies the recorded source and destination
parent identities, and verifies that each child name still resolves to the
recorded source and destination directory identities. It closes partially
acquired descriptors on setup failure, rejects overlapping source/destination
roots, and retains the prior sanitized directory-mode policy. Existing
100,000-entry and 4 GiB streaming budgets, no-follow opens, source-file checks,
and failed-file cleanup remain in place.

The regression copies a 1,100-level directory chain under an `RLIMIT_NOFILE`
limit far below its depth and reads back the leaf. Focused regressions cover
descriptor setup failure, overlapping roots, destination replacement, and
directory-mode sanitization. All five focused tests passed in 19.8 seconds:

```sh
python3 -m unittest \
  scripts.tests.test_p3_candidate_fs.CandidateFilesystemTests.test_copy_tree_handles_deep_directory_chain_with_bounded_descriptors \
  scripts.tests.test_p3_candidate_fs.CandidateFilesystemTests.test_copy_closes_source_duplicate_when_destination_dup_fails \
  scripts.tests.test_p3_candidate_fs.CandidateFilesystemTests.test_copy_rejects_overlapping_roots \
  scripts.tests.test_p3_candidate_fs.CandidateFilesystemTests.test_copy_rejects_destination_directory_replacement \
  scripts.tests.test_p3_candidate_fs.CandidateFilesystemTests.test_copy_sanitizes_source_directory_modes
```

This closes a local intake-copy resource gap only. Candidate execution,
correctness, GUI/performance acceptance, independent review, approval, and
promotion remain separate gates.
