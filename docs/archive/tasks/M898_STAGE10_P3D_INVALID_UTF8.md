# M898 — Stage 10 / normalize invalid UTF-8 P3D evidence

The P3D visual-evidence validator previously let `UnicodeDecodeError` escape
when the evidence file contained invalid UTF-8. It now raises the established
`InvalidEvidence` error with a stable message.

Focused regression passed 1/1:

```text
python3 -m unittest scripts.tests.test_validate_p3d_visual_evidence.ExactShapeValidationTests.test_rejects_non_utf8_manifest_as_invalid_evidence
```

This closes malformed UTF-8 error normalization only. Candidate provenance,
build and correctness, GUI and performance acceptance, review, approval, and
promotion remain open.
