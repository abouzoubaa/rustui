# M1153 — Stage 10: reject hard-linked P3D artifacts

P3D artifact paths were opened beneath a pinned root without following
symlinks, but the validator accepted regular files with hard-link aliases. An
artifact path inside the evidence root could therefore refer to the same inode
as a file outside that root.

The P3D reader now rejects artifacts whose link count exceeds one before and
after hashing and after reopening the recorded path. The new outside-root
hard-link regression failed before the fix. Focused `ArtifactStabilityTests`
passed 2/2, covering both hard-link rejection and replacement of the recorded
path during validation.

This establishes artifact identity and path integrity for structural
validation. It does not establish screenshot or visual correctness, native
input or accessibility acceptance, performance criteria, independent review,
approval, or candidate promotion. No GUI capture or candidate pipeline was
run.
