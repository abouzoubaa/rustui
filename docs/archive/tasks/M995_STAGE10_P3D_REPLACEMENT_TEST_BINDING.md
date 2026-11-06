# M995 — Stage 10 / bind the P3D replacement-race test mock correctly

The test now patches `Path.read_bytes` with `autospec=True`. This preserves
method binding, so the replacement side effect receives the path, replaces the
artifact after reading, and reaches the validator assertion.

Focused check: `ArtifactStabilityTests.test_rejects_screenshot_replaced_during_hash_read`
passed (1/1).

This is test-harness evidence only. Candidate execution, review, approval, and
promotion remain open.
