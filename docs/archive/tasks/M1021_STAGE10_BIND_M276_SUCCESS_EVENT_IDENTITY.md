# M1021 — Stage 10: bind the M276 success event to candidate identity

The P3M aggregator now rejects an M276 success event when its explicit
`candidate_id` or `manifest_sha256` conflicts with the pipeline summary. Legacy
events that omit those optional fields remain compatible. The focused test
rejected both transplanted identities; it failed before the fix and passed
afterward.

This hardens provenance consistency only. No candidate was run; candidate
build/correctness, GUI/performance evidence, review, approval, and promotion
remain open.
