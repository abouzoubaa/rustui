# M993 — Stage 10 / reject empty P3K unified-diff hunks

P3K strict patch application now rejects unified-diff hunks that consume and
produce zero rows.

Focused check: a malformed `@@ -0,0 +0,0 @@` hunk regression failed before
the guard and passed afterward (1/1).

No candidate was run. Review, approval, and promotion remain open.
