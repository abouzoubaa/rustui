# M1030 — Reject build-provenance output hard links

With `--force --output`, the build-provenance recorder compared resolved path
strings. A different path hard-linked to the supplied build log passed that
check, allowing the log to be truncated after its hash had been captured.

The recorder now rejects an existing output that is the same file as the
supplied log. The focused preservation regression passed:

```text
python3 -m unittest scripts.tests.test_record_build_provenance.RecordBuildProvenanceTests.test_force_rejects_output_hard_link_to_build_log
Ran 1 test, OK
```

No candidate run or broader Stage 10 review/promotion was performed.
