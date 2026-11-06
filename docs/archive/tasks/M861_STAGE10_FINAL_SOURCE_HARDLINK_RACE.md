# M861 — Stage 10 / reject hard links added after candidate snapshot

The P3F final source-closure inventory now rejects hard-linked source files
and checks that file device, inode, size, modification time, change time, and
link count remain stable across hashing. This closes the race where an
external hard link could be added after P3G and before final integrity
validation without changing file bytes. A regression adds the external link
during mocked P3H work and verifies the final integrity result is `failed`.
The focused candidate-pipeline suite passed 35/35; scoped whitespace checks
reported no diagnostics. See `scripts/run_candidate_pipeline.py` and
`scripts/tests/test_run_candidate_pipeline.py`.

This closes a source-closure integrity gap only. Candidate build/correctness,
native GUI/performance acceptance, review, approval, and promotion remain open.
