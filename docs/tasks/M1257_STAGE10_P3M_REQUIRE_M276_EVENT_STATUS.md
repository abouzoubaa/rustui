# M1257 — Stage 10: require explicit M276 event status

The P3M aggregator previously treated an M276 success event with no `status`
field as `passed`. M1234 requires explicit event status, so a hash-bound event
must not gain success from an omitted field.

The M276 success check now requires `status: "passed"`. Its fixture records
that field, and a regression removes it, updates the event digest, and confirms
aggregation remains invalid with every gate at `not_run`.

The focused aggregate-evidence suite passed **71 tests**; Python bytecode
compilation passed. This verifies local evidence-record consistency only. No
candidate was executed; correctness, GUI/performance acceptance, independent
review, approval, and promotion remain open.
