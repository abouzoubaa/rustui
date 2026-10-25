# M872 — Stage 10 Boolean performance evidence integers

## Change

The M276 performance evidence validator now requires exact integer types for
process exit codes, cold iteration counts, summary build exit status, and
provenance verified-row pair indices. Python treats `bool` as an `int`, so
equality-only checks could accept JSON `false` as exit code `0`, or Boolean
values as iteration/pair indices.

## Focused evidence

- Four focused regressions reject Boolean values in those integer fields.
- All four rejection regressions passed.
- A valid-summary baseline regression passed: 1 passed.
- Scoped whitespace checks produced no findings.

## Acceptance boundary

This hardens performance-evidence field types. It does not establish candidate
provenance, correctness, GUI/performance acceptance, review, approval, or
promotion.
