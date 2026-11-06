# M782 — Stage 10: report artifact read I/O errors

## Gap

The P3F candidate evidence validator promised structured JSON and a nonzero exit for malformed local evidence. `_artifact` converted `EvidenceError` into an invalid report, but an `OSError` during the read of an already-opened regular artifact escaped and produced a traceback.

## Change

Convert the artifact read `OSError` into an invalid-evidence error at the `_artifact` boundary. The regression simulates the read failing after descriptor acquisition and verifies the validator reports `status: invalid`.

## Focused evidence

- `python3 scripts/tests/test_validate_candidate_evidence.py ValidateCandidateEvidenceTests.test_reports_artifact_read_io_error_as_invalid_evidence`: passed 1/1.
- The test covers this read-failure path only; it does not close broader P3 provenance, build/correctness, GUI/performance, approval, or promotion gates.
