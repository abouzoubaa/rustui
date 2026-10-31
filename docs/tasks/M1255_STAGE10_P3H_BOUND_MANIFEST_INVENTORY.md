# M1255 — Stage 10 P3H manifest inventory uses bounded descriptors

P3H's pinned source reporter hashed workspace `Cargo.toml` files by recursively
opening one `PinnedDir` per nested directory. A deeply nested source tree could
therefore exhaust the process descriptor limit before producing its report.

`_pinned_manifest_hashes` now traverses iteratively with constant active
directory descriptors. It verifies child identity and directory metadata on
descent and ascent, while preserving the directory records and single-link
manifest hashing.

The regression scans 1,100 nested directories under an `RLIMIT_NOFILE` limit
below tree depth, checks the leaf manifest hash, and requires all directory
records. The new deep-tree test and existing pinned path-swap regression passed
2/2 in 7.111 seconds:

```sh
python3 -m unittest \
  scripts.tests.test_report_candidate_sources.ReportCandidateSourcesTests.test_pinned_manifest_inventory_is_depth_safe_under_low_fd_limit \
  scripts.tests.test_report_candidate_sources.ReportCandidateSourcesTests.test_pinned_report_survives_candidate_path_swap_during_cargo
```

This closes a local P3H inventory resource gap only. Candidate execution,
correctness, GUI/performance acceptance, independent review, approval, and
promotion remain separate gates.
