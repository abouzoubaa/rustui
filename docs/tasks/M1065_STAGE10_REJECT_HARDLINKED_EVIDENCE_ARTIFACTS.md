# M1065 — Stage 10 / reject hard-linked candidate evidence artifacts

Candidate evidence validation now rejects an artifact with more than one
hard link when it is opened, after its contents are hashed, and when its path
is reopened. This prevents an external alias from changing evidence outside
the candidate tree, including a hard link added during hashing. The focused
hard-link and symlink-escape regressions passed 2/2. No candidate was run,
approved, or promoted.
