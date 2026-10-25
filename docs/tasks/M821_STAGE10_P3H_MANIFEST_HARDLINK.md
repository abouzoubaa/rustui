# M821 — Stage 10 P3H hard-linked package manifests

## Finding

P3H source-path validation rejected symlinked package manifests but allowed a regular `Cargo.toml` hard-linked to a file outside the candidate. A candidate could therefore include an external file through a path that looked like a regular in-tree manifest.

## Change

The source reporter rejects a regular file with a hard-link count greater than one while validating the package source path. The regression creates an external Cargo manifest, replaces the candidate manifest with a hard link to it, and requires the report to be invalid.

## Evidence

- Regression: `scripts/tests/test_report_candidate_sources.py::test_rejects_manifest_hard_link_to_file_outside_candidate`
- Command: `python3 scripts/tests/test_report_candidate_sources.py`
- Result: 14 tests passed.

## Remaining Stage 10 gates

This closes one P3H source-integrity input gap. Candidate acceptance, build and correctness evidence, GUI and performance evidence, review, approval, and promotion remain open.
