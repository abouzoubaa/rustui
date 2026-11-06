# M997 — pin the approved P3E plan in the P3F record

The candidate pipeline already copied the approved P3E patch plan into the
candidate evidence directory and checked its digest during intake. The P3F
record now also lists that copy and its approved SHA-256 digest, so later
evidence validation can bind the retained plan to the pipeline record.

The focused synthetic regression passed:

```text
python3 scripts/tests/test_run_candidate_pipeline.py CandidatePipelineTests.test_p3f_record_hash_pins_copied_approved_patch_plan
1 test passed
```

All pipeline steps in this regression are mocked. No candidate validators,
builds, correctness runs, GUI captures, or performance measurements were run.
Candidate review, approval, and promotion remain open.
