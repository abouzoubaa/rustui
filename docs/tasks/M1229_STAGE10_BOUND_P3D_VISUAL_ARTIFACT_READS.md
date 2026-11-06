# M1229 — Bound P3D visual evidence artifact reads

P3D now hashes screenshot and binary artifacts in 1 MiB chunks instead of
retaining their full contents. JSON evidence records are read in chunks with a
1 MiB record limit, matching the neighboring candidate-evidence contract.

The focused regression verifies bounded artifact reads and rejects an
oversized JSON record:

```sh
python3 -m unittest \
  scripts.tests.test_validate_p3d_visual_evidence.ArtifactReadBoundsTests.test_artifact_hashing_uses_bounded_reads_and_json_records_are_limited
```

Result: passed (1/1). This is evidence-reader hardening only. The existing M582
capture manifest has a binary hash mismatch against the current example, so it
does not establish current-source visual acceptance. Current-source screenshots,
native interaction/accessibility review, performance evidence, independent
review, approval, and promotion remain open.
