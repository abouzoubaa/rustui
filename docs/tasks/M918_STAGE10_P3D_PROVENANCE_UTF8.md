# M918 — Stage 10 P3D provenance artifact UTF-8

`artifact_json()` now normalizes invalid UTF-8 in referenced build and plan
artifacts to `InvalidEvidence`, matching the top-level evidence manifest. This
prevents malformed provenance JSON from escaping the validator's CLI error
handling as an uncaught `UnicodeDecodeError`.

Focused regression passed:

```text
python3 -m unittest scripts.tests.test_validate_p3d_visual_evidence.ExactShapeValidationTests.test_rejects_non_utf8_provenance_artifact_as_invalid_evidence
```

This validates error normalization for nested provenance JSON only. It does
not establish visual, input, accessibility, performance, or promotion
acceptance.
