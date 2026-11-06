# M791 — Reject Invalid P3H Packages Containers

**Stage:** 10 — Candidate validation and promotion gates  
**Status:** Implemented; focused reporter suite passed 13/13

## Defect

P3H reports a diagnostic when Cargo metadata `packages` is not an array, but
then continued to iterate the malformed value. `null` and integer values
therefore raised `TypeError` instead of returning the invalid report; strings
and objects could generate misleading per-item diagnostics.

## Change

Normalize only the invalid container to an empty local iteration list after
recording the existing error. Valid arrays retain their existing validation
path, while malformed metadata stays an invalid report.

## Evidence

Focused regression covers null, object, integer, and string containers:

```sh
python3 scripts/tests/test_report_candidate_sources.py
```

This validates local P3H error handling only. It does not establish candidate
provenance, build or runtime correctness, GUI acceptance, performance, or
promotion.
