# M790 — Reject Non-String P3H Package Sources

**Stage:** 10 — Candidate validation and promotion gates  
**Status:** Implemented; focused regression passed

## Defect

`report_candidate_sources.validate_metadata` accepted malformed Cargo metadata
where `packages[].source` was a JSON object or array far enough to use it in a
normalized identity tuple. An object or array is unhashable, so the report CLI
raised `TypeError` instead of returning an invalid P3H report with a diagnostic.

## Change

Reject non-null, non-string source values at the metadata boundary. The report
remains invalid and carries an actionable error instead of terminating with a
traceback. `null` and string source values continue through the existing path,
registry, and git-source validation.

## Evidence

Before the fix, the existing P3H fixture with
`packages[0].source = {"unexpected": "object"}` raised `TypeError: unhashable
type: 'dict'` from the normalized identity set membership check. The focused
P3H reporter suite now covers object, array, and integer source values.

Run:

```sh
python3 scripts/tests/test_report_candidate_sources.py
```

This fixes report generation for malformed metadata. It does not establish
candidate provenance, build or runtime correctness, GUI acceptance,
performance, or promotion approval.
