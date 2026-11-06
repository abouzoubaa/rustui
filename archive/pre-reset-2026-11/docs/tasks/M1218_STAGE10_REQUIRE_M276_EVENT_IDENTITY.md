# M1218 — Stage 10 / require M276 success-event identity

The P3M evidence aggregator previously accepted a hash-bound M276 success
event when its `candidate_id` or `manifest_sha256` was absent. The M276 summary
binds the current candidate manifest, but its source-identity check covers only
the baseline and candidate scene files. An identity-free event from an earlier
correctness run could therefore be reused after another candidate source file
changed, while the scene files remained unchanged.

M276 success events must now include both identity fields and match the
summary. A focused regression removes each field in turn and requires
aggregation to fail with every reported gate left `not_run`. Historical
identity-free M276 event records no longer establish a passed correctness
gate. This is evidence-integrity validation only; no candidate or acceptance
gate was run.
