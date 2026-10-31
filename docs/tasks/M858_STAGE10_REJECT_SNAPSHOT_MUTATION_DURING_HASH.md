# M858 — Stage 10 / reject snapshot mutation during hashing

The P3G candidate snapshot validator now compares a regular file's device,
inode, size, modification time, and change time before and after hashing. It
rejects a source file that changes while the snapshot inventory is being
computed. The regression mutates already-read and not-yet-read regions of a
2 MiB file during hashing and verifies rejection. The focused snapshot suite
passed 21/21. The manifest read-spy wrapper now delegates `fileno()` so the
existing hard-link check remains covered. See
`scripts/validate_candidate_snapshot.py` and
`scripts/tests/test_validate_candidate_snapshot.py`.

This closes a local snapshot-integrity gap only. Candidate provenance,
build/correctness, native GUI/performance acceptance, review, approval, and
promotion remain open.
