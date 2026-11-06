# M942 — Stage 10 / reject duplicate P3D evidence JSON keys

P3D visual-evidence records and referenced build/plan provenance JSON now
reject duplicate object keys. Python's default JSON decoder kept the last value,
which allowed an ambiguous, hash-consistent provenance record to pass through
validation. A focused regression confirms duplicates are rejected both in the
primary visual record and in a referenced build artifact.

This is local evidence-parser integrity only. It does not establish candidate
correctness, build, GUI, performance, review, approval, or promotion.
