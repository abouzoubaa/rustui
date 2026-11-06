# M1044 Stage 10: bind P3D artifacts to their current paths

`validate_p3d_visual_evidence.py` hashed bytes from an already-open descriptor
and checked that descriptor for in-place changes, but did not confirm that the
manifest path still named that inode. A screenshot could be replaced after
open and the validator would accept the old bytes although the recorded path
now referred to different content.

After reading, the validator now reopens the artifact through the same pinned,
no-follow directory walk and requires the current path's device, inode, size,
and timestamps to match the descriptor that was hashed. Replacing the path
during validation is rejected.

Focused regression:

```text
python3 scripts/tests/test_validate_p3d_visual_evidence.py ArtifactStabilityTests.test_rejects_screenshot_path_replaced_after_open
Ran 1 test ... OK
```

This closes path-to-bytes identity during structural validation only. The
four-state P3D record still needs human pixel review and native input and
accessibility acceptance; performance thresholds, independent review,
approval, and candidate promotion remain separate open gates. No GUI capture
or candidate pipeline was run.
