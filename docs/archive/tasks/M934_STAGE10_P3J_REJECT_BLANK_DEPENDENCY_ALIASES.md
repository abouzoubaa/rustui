# M934 — Stage 10 / reject blank P3J dependency aliases

P3J now rejects normalized dependency edges whose alias is empty or contains
only whitespace. Previously a self-consistent P3H schema-v2 report with a
recomputed graph digest could pass validation with an unusable dependency
alias. The comparator now applies the same nonblank rule already used for
dependency-kind and target metadata.

The focused comparator regression covers empty and whitespace-only aliases,
recomputing the graph digest so validation reaches the alias check. The
focused P3I/P3J comparison suite passed; Python syntax validation and a scoped
whitespace check passed. This is local report-integrity validation only. It
does not establish source provenance, candidate correctness, build, GUI,
performance, approval, or promotion.
