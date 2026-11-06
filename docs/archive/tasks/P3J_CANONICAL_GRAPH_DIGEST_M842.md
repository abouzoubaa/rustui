# M842 — Canonical P3J graph digest

The P3J comparator now computes the schema-v2 normalized graph digest after
canonicalizing node, dependency, dependency-kind, and feature array order. A
valid graph therefore has the same digest when only JSON array order differs,
matching the semantic graph comparison and avoiding a misleading digest
mismatch for equivalent candidate metadata.

The focused comparator suite passed 25/25, including a regression that reverses
the candidate graph's node order while retaining the canonical digest. This
checks report normalization only; it is not source provenance, build, runtime,
GUI, performance, or candidate acceptance evidence.
