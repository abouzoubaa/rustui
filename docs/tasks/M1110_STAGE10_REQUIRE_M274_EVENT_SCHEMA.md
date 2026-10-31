# M1110 — Stage 10 require the M274 event schema

## Contract

P3L follow-up aggregation must only accept a valid, supported M274 event record
when evaluating the build gate. An absent or unsupported schema version cannot
count as successful build evidence.

## Change

The M274 follow-up event reader now requires `schema_version` to be the exact
integer `1`, rejecting missing, Boolean, and unsupported values. The regression
reproduces the prior false acceptance.

## Evidence

`test_m274_event_rejects_unsupported_schema_version` and the existing positive
follow-up build test passed. Scoped whitespace checks found no errors.

This is focused aggregation evidence. Candidate execution, complete build and
correctness review, GUI/performance evidence, approval, and promotion remain
open.
