# M1262 — Stage 10: bound P3H Cargo manifest reads

P3H now rejects candidate-local `Cargo.toml` files larger than 32 MiB before
reading them in both the standalone and production pinned paths. Descriptor
reads are capped at the limit plus one byte, so a file that grows during
inspection cannot cause an unbounded allocation. Existing opened/path metadata
comparisons still reject a changing file.

Focused regressions lower the configured bound to 16 bytes and confirm oversized
manifests raise `SourceReportError` through standalone inventory, pinned
inventory, and automatic/explicit pinned workspace selection. All three tests
passed (3/3).

This bounds per-manifest memory use only. Candidate execution, source
provenance, correctness, GUI/performance acceptance, independent review,
approval, and promotion remain open.
