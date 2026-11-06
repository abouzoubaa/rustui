# M841 — Stage 10 P3K reject no-op patches

The strict patch applicator now rejects a patch whose validated hunks produce
the original target bytes. This closes the schema 1/2 path where a context-only
patch could report success when the candidate scene was unchanged from the
baseline. The focused regression passed (1 test) and confirms the rejected run
retains its `result.json`. This is local synthetic patch-integrity evidence;
candidate provenance, build/correctness, GUI/performance acceptance, review,
approval, and promotion remain separate gates.
