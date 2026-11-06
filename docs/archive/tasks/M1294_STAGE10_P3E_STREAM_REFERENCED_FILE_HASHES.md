# M1294 — Stage 10: stream P3E referenced-file hashing

P3E now hashes referenced checksum and patch files in 64 KiB chunks instead of
loading each full file with `Path.read_bytes()`. SHA-256 still covers the exact
raw file bytes; manifest schema, validation results, and candidate gates are
unchanged.

The focused regression hashes a 1.5 MiB patch fixture while recording every
read size and checks the exact expected digest. Before the fix, it failed
because the helper made one unbounded read. After the fix, it passed 1/1:

`python3 -m unittest scripts.tests.test_validate_patch_plan.ValidatePatchPlanTests.test_referenced_file_hash_uses_bounded_reads_and_exact_bytes -v`

Each recorded read is positive and no larger than 65,536 bytes. No other tests,
candidate commands, builds, or dependency changes were run.

This bounds memory used for referenced-file digesting only. It does not authorize
candidate intake, patch application, candidate build/test/native/performance
work, dependency or revision changes, GPUI Fast, advanced rendering, or
promotion.
