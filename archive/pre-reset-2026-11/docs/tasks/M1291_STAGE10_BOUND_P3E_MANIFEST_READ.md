# M1291 — Stage 10: bound P3E patch-plan manifest reads

The read-only patch-plan validator now reads at most 32 MiB plus one detection
byte before decoding and parsing the manifest. Oversized input produces the
existing structured invalid-report path. This follows the 32 MiB metadata
capture boundary used elsewhere in Stage 10 and does not change manifest
schema or approval semantics.

The focused regression accepts an input exactly at a patched-in test limit and
rejects the same input when it is one byte over. The focused test passed 1/1:

`python3 -m unittest scripts.tests.test_validate_patch_plan.ValidatePatchPlanTests.test_manifest_read_accepts_exact_limit_and_rejects_one_byte_more -v`

This bounds manifest input memory only. Referenced patch and checksum hashing,
candidate execution, correctness, GUI/performance acceptance, review, approval,
and promotion remain separate.
