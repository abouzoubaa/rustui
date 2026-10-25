# M1032 — Reject duplicate enabled features in candidate sources

The candidate source reporter now rejects repeated enabled feature names
instead of silently deduplicating them. This keeps the submitted feature list
explicit and prevents duplicate input from being normalized into apparently
valid evidence.

Focused validation:

- `python3 scripts/tests/test_report_candidate_sources.py` — 18 tests passed.

This records the implementation slice only. It does not complete the Stage 10
candidate run, evidence approval, or promotion gates.
