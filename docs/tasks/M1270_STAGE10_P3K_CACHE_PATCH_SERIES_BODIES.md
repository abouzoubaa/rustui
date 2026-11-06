# M1270 — Stage 10 bound resident P3K patch-series bodies

P3K now copies each patch from its descriptor-pinned candidate directory into a
private temporary cache in 1 MiB chunks, verifies the cached bytes against the
manifest hash, and retains only patch metadata for the complete series. Each
patch is limited to the existing 32 MiB intake ceiling, including standalone
applicator calls. Application reads and rechecks one cached body at a time, so a
candidate-path swap cannot change which bytes are strictly applied. The final
descriptor-relative source-path integrity recheck also hashes in 1 MiB chunks
rather than loading a changed patch body into memory. The cache is removed on
successful application and rejection.

This preserves ordered multi-patch acceptance without introducing an
aggregate-series contract limit. It bounds resident patch bodies to one 32 MiB
patch plus the copy chunk. It does not bound aggregate temporary-disk use or
schema 3's separately specified in-memory staging of patched package outputs.

Eight focused application regressions passed, covering cache cleanup/storage,
the per-patch limit before output creation, ordered series application, hash
mismatch rejection, schema 2/3 path swaps, and schema 3 success and rollback.
The changed-during-application regression also asserts that the final source
integrity check does not call the unbounded file reader for patch bodies. A
focused five-test rerun and Python bytecode compilation passed. This is local
synthetic P3K evidence only;
candidate execution, downstream acceptance, review, approval, and promotion
remain open.

Current primary-checkout revalidation (2026-10-24):
`python3 scripts/tests/test_apply_candidate_patch.py` passed all 59 tests.
This remains synthetic local coverage; no candidate was executed.

## M1271 — close candidate descriptor on cache-construction failure (2026-10-24)

`_patch_series` now creates its temporary cache inside the `try/finally` that
owns `candidate_fd`. A cache-construction exception therefore closes either
the standalone-opened descriptor or the duplicate of a pinned descriptor; the
caller's pinned descriptor remains open. The focused regression covers both
paths. `PYTHONPATH=/Users/Abium/Developer/rustui/scripts python3 -m unittest
scripts.tests.test_apply_candidate_patch.CandidatePatchApplicationTests.test_patch_series_closes_owned_descriptor_if_cache_creation_fails -v`
passed (1/1).

The test injects an `OSError` from temporary-directory creation. It exercises
no patch application and is not candidate-execution evidence. The isolated
worktree lacks `scripts/p3_candidate_fs.py` and
`scripts/validate_candidate_snapshot.py`; those import helpers came from the
primary checkout via `PYTHONPATH`. The applicator and test under test were from
the isolated worktree. Aggregate temporary-disk use and schema 3 in-memory
output staging remain unchanged and open.

## M1272 — verify the exact 32 MiB per-patch boundary (2026-10-24)

`PatchCacheLimitTests.test_cache_patch_accepts_exact_32_mib_and_rejects_one_byte_more`
calls `_cache_patch_at` directly. It generated a synthetic patch file in
chunks, verified that exactly 32 MiB is accepted with the expected SHA-256 and
cached size, then appended one byte and verified rejection. The focused test
passed (1/1):

`PYTHONPATH=/Users/Abium/Developer/rustui/scripts python3 -m unittest scripts.tests.test_apply_candidate_patch.PatchCacheLimitTests.test_cache_patch_accepts_exact_32_mib_and_rejects_one_byte_more -v`

This exercises intake only: it applies no patch and runs no candidate. Import
helpers came from the primary checkout because they are absent in this isolated
worktree; the applicator and regression test came from this worktree. Aggregate
temporary-disk use and schema 3 in-memory output staging remain unchanged and
open.
