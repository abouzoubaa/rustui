# M1114 — Stage 10 recheck candidate inputs after P3F

## Contract

The candidate pipeline must not retain a partial-success result if P3F changes
candidate inputs after the pipeline's earlier integrity check.

## Change

After P3F, the pipeline now rechecks candidate path identity, manifest and
immutable-manifest hashes, and both source inventories. A mismatch records a
`P3G-post-P3F-integrity` failure and marks the pipeline and P3F record failed.

## Evidence

The focused new regression `test_p3f_cannot_leave_mutated_source_with_partial_result`,
the existing source-drift regression, and the stage-order test passed (3 tests).
Scoped whitespace checks found no errors.

This closes one pipeline integrity path; no candidate was run or accepted.
