# M776 — Stage 10 keep snapshot descriptor open for final link check

The paired-snapshot validator now performs its final hard-link-count check
while the source file descriptor is still open. Previously, it called
`fstat` on the file object after the `fdopen` context had closed it, causing
valid snapshot pairs to fail validation.

The focused snapshot-validator suite passed 18/18; Python compilation and
scoped whitespace checks passed. This closes the local inventory-validation
defect only. It does not establish candidate provenance, pipeline build or
correctness, GUI or performance acceptance, approval, or promotion.
