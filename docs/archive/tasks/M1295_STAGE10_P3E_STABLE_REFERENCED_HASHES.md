# M1295 — Stage 10: reject P3E referenced-file mutation during hashing

P3E now binds each streamed checksum or patch digest to one opened file
version. It compares device, inode, size, modification time, and change time
before and after reading, and checks that the bytes read match the initial
size. A referenced file changed during hashing is reported invalid.

The focused regression modifies a later chunk in place after the first chunk
has been read, preserving the inode and size. The pre-fix helper returned no
error because buffered reads could still supply the original bytes. After the
fix, the mutation is detected. The existing M1294 bounded-read and exact-byte
digest regression was rerun alongside it; both passed (2/2):

`python3 -m unittest scripts.tests.test_validate_patch_plan.ValidatePatchPlanTests.test_referenced_file_hash_uses_bounded_reads_and_exact_bytes scripts.tests.test_validate_patch_plan.ValidatePatchPlanTests.test_referenced_file_hash_rejects_in_place_mutation -v`

This binds local digest verification to a stable file read only. No other tests,
candidate commands, builds, or dependency changes were run. Candidate intake,
patch application, candidate build/test/native/performance work, dependency or
revision changes, GPUI Fast, advanced rendering, and promotion remain gated.
