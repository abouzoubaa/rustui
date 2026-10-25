# M1252 — Stage 10 P3H rejects blank enabled feature names

The P3H Cargo metadata reporter now rejects enabled feature entries that are
empty or contain only whitespace. This keeps emitted resolved-graph reports
within the P3J comparison contract, which already rejects blank feature names.

The focused regression covers empty, space-only, and tab-only values. The
focused check passed 1/1:

```sh
python3 -m unittest scripts.tests.test_report_candidate_sources.ReportCandidateSourcesTests.test_rejects_blank_enabled_features
```

This validates malformed metadata handling only. It does not establish
candidate provenance, source equivalence, build or correctness, GUI behavior,
performance, approval, or promotion.
