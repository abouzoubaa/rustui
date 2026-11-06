# M974 — Stage 10 / bind P3F command provenance to the pipeline invocation

## Change

P3F command metadata now records the canonical resolved intake path and
`--root` value. It also records `--run-id` when the caller explicitly supplies
one, so evidence identifies the invocation that produced the candidate.

## Focused evidence

The focused `test_copies_snapshot_pins_hashes_and_runs_stages_in_order` test
passed (1/1). Later validator results in the fixture were synthetic; no
candidate was executed.

## Remaining gates

Candidate correctness/build evidence, GUI and performance evidence, independent
review, approval, and promotion remain open.
