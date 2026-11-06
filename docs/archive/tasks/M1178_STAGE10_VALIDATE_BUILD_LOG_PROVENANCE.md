# M1178 — Stage 10 / validate build-log provenance metadata

The P3D cache-comparison validator did not inspect the top-level `build_log`
record written by the provenance recorder. A record could omit the log or
provide an invalid digest, negative size, or `parsed_for_success: true` while
the other provenance fields remained valid.

The validator now requires an object with a nonempty path, lowercase SHA-256,
nonnegative integer byte size, and exactly `false` for `parsed_for_success`.
The focused regression covers omission and malformed field values; it passed
1/1. The log is not reopened and rehashed, so this is metadata-shape validation,
not proof that the referenced bytes still match.

```sh
python3 scripts/tests/test_compare_cache_workload.py ComparisonTests.test_diagnostics_overhead_rejects_malformed_build_log_record
```

No candidate was executed; candidate build, correctness, GUI/performance,
review, approval, and promotion gates remain open.
