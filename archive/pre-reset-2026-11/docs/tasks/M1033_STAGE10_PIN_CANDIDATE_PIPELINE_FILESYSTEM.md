# M1033 — Pin candidate pipeline filesystem operations

## Problem

Candidate tooling validated paths under `target/headstart/candidates` and then
reopened them by name. A directory could be renamed or replaced with a decoy
symlink between those operations, then restored before final path checks. The
final checks could report the original identity even though an intermediate
read or write had followed the replacement path. Persisted `/dev/fd/<n>` paths
also were not stable manifest references.

## Change

- Added `scripts/p3_candidate_fs.py` with descriptor-relative directory and
  file operations that reject symlinks and unsafe relative paths.
- Pinned repository, candidate parent, candidate directory, and prepared
  source roots in the runner and passed descriptors explicitly to child tools.
- Launched Python validators from the pinned repository directory and Cargo
  from a pinned workspace descriptor.
- Kept manifest paths stable and relative; no ephemeral descriptor path is
  persisted.
- Updated snapshot validation, source reporting/comparison, evidence
  validation, provenance, patch-plan validation, visual-evidence validation,
  evidence aggregation, and patch application to use pinned roots.
- Added deterministic rename/symlink-swap regressions around filesystem reads,
  writes, and subprocess launches.

## Focused evidence

The combined affected Stage 10 test modules passed **265 tests**. Python
compilation passed for the changed scripts and tests. Whitespace checks reported
no errors. The candidate intake/run, evidence review, approval, and promotion
gates remain open and were not exercised by this milestone.
