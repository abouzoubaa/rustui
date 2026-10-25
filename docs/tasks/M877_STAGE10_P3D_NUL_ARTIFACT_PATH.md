# M877 — Stage 10 P3D NUL artifact path

## Change

The P3D visual-evidence artifact resolver converts path-resolution failures
into `InvalidEvidence`. A focused regression supplies an embedded NUL in the
JSON artifact path and checks that validation reports the controlled
`artifact path cannot be resolved` error rather than leaking the platform
path exception.

## Focused evidence

- `python3 scripts/tests/test_validate_p3d_visual_evidence.py ArtifactPathValidationTests.test_rejects_embedded_nul_path_without_value_error` passed (1 test).

## Acceptance boundary

This covers malformed artifact-path error handling only. Candidate provenance,
correctness, GUI/performance acceptance, review, approval, and promotion remain
open.
