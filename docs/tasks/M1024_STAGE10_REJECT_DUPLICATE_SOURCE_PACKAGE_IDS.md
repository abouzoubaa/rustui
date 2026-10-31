# M1024 — Reject duplicate Cargo package IDs in source reports

The candidate-source comparator's report reader accepted duplicate Cargo
package IDs even though the source reporter rejects them. A report could
therefore claim `status: valid` while carrying an ambiguous package inventory.

`scripts/compare_candidate_sources.py` now rejects repeated package IDs while
reading the report. The focused comparator test suite passed **28 tests**:

```text
python3 scripts/tests/test_compare_candidate_sources.py
```

No candidate was run. Candidate correctness, GUI/performance evidence, review,
approval, and promotion remain open.
