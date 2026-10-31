# M1263 — Stage 10: bound P3H supporting Cargo inputs

P3H now caps each candidate `Cargo.lock` and Cargo config read at 32 MiB in
standalone and production pinned paths. Reads stop at the limit plus one byte,
so oversized inputs are rejected while growth during inspection is detected.
Existing input hashes and before/after drift checks remain in place.

Four focused regressions cover oversized lockfile and config inputs through
standalone and pinned paths; all four passed (4/4).

This bounds supporting-file reads only. Cargo metadata stdout/stderr capture
is handled separately: M1264 bounds the dependency-boundary checker and M1265
bounds the candidate-source reporter. Candidate execution, correctness,
GUI/performance acceptance, independent review, approval, and promotion remain
open.
