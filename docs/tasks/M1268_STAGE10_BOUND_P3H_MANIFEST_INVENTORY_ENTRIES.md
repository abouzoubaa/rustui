# M1268 — Stage 10 bound P3H manifest inventory entries

P3H retained directory and `Cargo.toml` records for the complete workspace
without a total inventory limit. Both standalone and descriptor-pinned scans
now cap direct entries across the workspace at 100,000 and fail as soon as the
first excess entry is observed. Per-directory entry lists are built
incrementally under the same budget, bounding memory for wide directories.

The candidate-source reporter suite passed 35/35, including new standalone and
pinned over-limit regressions. Candidate execution, correctness, GUI/performance
acceptance, independent review, approval, and promotion remain open.
