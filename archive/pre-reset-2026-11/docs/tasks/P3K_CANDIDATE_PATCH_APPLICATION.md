# P3K: Exact candidate patch application check

**Status: bounded implementation.** P3K verifies either a legacy single patch
or an ordered, hash-pinned patch series against the candidate's recorded hashes
and applies it only to a fresh copy of the baseline package. It does not apply
changes to either prepared snapshot, the active workspace, `vendor/`, or Cargo
caches.

## Contract

`scripts/apply_candidate_patch.py` accepts a prepared candidate manifest and a
unique run ID. Schema 1 retains the single `approved.patch` contract. Schema 2
lists ordered patch filenames with individual hashes and scene preimage and
postimage hashes; the manifest's `patch_sha256` is the SHA-256 of the UTF-8
newline-joined `filename:patch_sha256` entries in order. The series must start
at the manifest baseline hash and end at the candidate hash. Every patch must
target only `gpui-pre-0.3.9/src/scene.rs`, and each hunk is checked at its
declared line without search, offsets, or fuzzy matching. The resulting bytes
must match the manifest's candidate postimage hash.

The script copies the baseline package to a new
`<candidate>/patch-checks/<run-id>/package` directory. It writes a per-run
`result.json`; rejected applications after copy creation retain the disposable
copy and failure record for inspection. A run ID cannot be reused. The prepared
baseline and candidate packages and immutable manifest are rehashed after the
check.

## M631 — Read-only copied target mode

P3K temporarily adds owner-write permission to a read-only target in the disposable output package, applies the patch, and restores that target's original mode. The prepared baseline and candidate remain unchanged. The regression passed in the focused patch-application suite (32/32); no candidate was run or promoted.

## Evidence and limits

M186's single-patch exact application check passed for
`quad-stable-sort-20261018-a` using run ID `m186-exact-r2-20261020`. The patch
hash, baseline preimage hash, and
candidate postimage hash matched. The post-check P3G snapshot report still
validates both 42,483-file snapshots with only the intended `scene.rs` delta.
Focused tests cover exact application, patch/preimage/postimage mismatches,
shifted hunk rejection, and run-ID reuse. Raw records:

- `target/headstart/candidates/quad-stable-sort-20261018-a/patch-checks/m186-exact-r2-20261020/result.json`
- `target/headstart/evidence/p3-m186-patch-application-tests-r2-20261020.log`
- `target/headstart/evidence/p3-m186-postcheck-snapshot-r2-20261020.json`

M192 adds synthetic two-patch coverage for ordered application, out-of-order
content rejection, intermediate postimage failure, and per-entry hash rejection;
eleven focused tests pass in
`target/headstart/evidence/p3k-ordered-patch-series-tests-m192-20261020.log`.
The suite also checks preservation of both prepared source packages after a
rejected intermediate postimage. The prepared Quad manifest passed the retained schema
1 compatibility path in run
`target/headstart/candidates/quad-stable-sort-20261018-a/patch-checks/m192-legacy-compat-20261020/result.json`.
This demonstrates the series mechanism against fixtures, not a real multi-patch
candidate. The check does not establish candidate provenance or approval,
candidate build or production correctness, GUI behavior, performance, acceptance,
or promotion. The broader P3 intake and subsequent gates remain separate.


M194 hardens strict manifest parsing (including rejecting Boolean schema versions
and unknown patch-entry fields), labels multi-patch result scope accurately, and
preserves `result.json` when a prepared patch disappears during post-check integrity
verification. Fourteen focused tests pass in
`target/headstart/evidence/p3k-patch-series-hardening-tests-m194-20261020.log`.
The live schema 1 Quad reproduction also passed after these changes; see
`target/headstart/candidates/quad-stable-sort-20261018-a/patch-checks/m194-postcheck-hardening-20261020/result.json`.

## M325 — Reject symlinked patch-check output roots

Before creating a per-run output, schema 3 P3K rejects an existing `patch-checks`
symlink or a non-directory destination root. A temporary fixture confirmed the
outside target remained empty and the helper rejected the operation. Focused
tests passed 21/21; `py_compile` passed. Logs:
`target/headstart/evidence/p3k-patch-checks-symlink-tests-m325-20261020.log`
and `target/headstart/evidence/p3k-patch-checks-symlink-pycompile-m325-20261020.log`.

## M340 — Atomic result publication

P3K writes JSON to an exclusive temporary file and publishes it with a
no-overwrite hard link only after serialization completes. An interrupted
serialization leaves no partial `result.json`. The focused suite passed 22/22
and `py_compile` passed; this is local result-file integrity evidence only.
Logs: `target/headstart/evidence/p3k-atomic-result-publication-m340-candidate-tests-20261021.log`
and `target/headstart/evidence/p3k-atomic-result-publication-m340-candidate-pycompile-20261021.log`.

## M343 — Verify the copied output package inventory

Before P3K reports success, it compares the complete copied output inventory
with the prepared candidate inventory, rejecting drift in unpatched files. The
focused suite passed 23/23 and `py_compile` passed. No candidate ran.
Logs: `target/headstart/evidence/p3k-single-file-output-inventory-audit-tests-20261021.log`
and `target/headstart/evidence/p3k-single-file-output-inventory-audit-pycompile-20261021.log`.

## M349 — Reject symlinked package roots

P3K now requires baseline and candidate package roots to be real directories
before inventory traversal. This closes the gap where `os.walk` could enumerate
a symlink supplied as its starting root. Focused suite passed 25/25 and
`py_compile` passed. No candidate was run.

## M378 — Reject symlinked `path-sources` parents

`_inventory` now rejects a symlinked package root or immediate parent. Regressions cover symlinked `path-sources` parents for baseline and candidate snapshots and confirm failure before result publication. Focused tests passed 26/26. Static symlink parents are covered; concurrent path swaps are not claimed. Evidence: `target/headstart/evidence/p3k-path-sources-parent-symlink-m378-20261021.log`.

## M384 — Pin P3K output-root operations to directory descriptors

P3K now opens or creates `patch-checks`, the per-run directory, and the output
package relative to pinned directory descriptors with no-follow opens. Package
copy, inventory, rollback, and atomic no-overwrite `result.json` publication
also use descriptor-relative operations, so swapping the output-root path for a
symlink at the open boundary is rejected without writing into its target. The
focused patch-application suite passed 27/27, the combined script regression
suite passed 203/203, and Python bytecode compilation passed for all scripts
and script tests. These are local synthetic regression checks; they do not run
or promote a candidate. Evidence:
`target/headstart/evidence/p3k-output-root-descriptor-race-tests-m384-20261021.log`,
`target/headstart/evidence/p3-candidate-pipeline-regression-m384-20261021.log`,
and `target/headstart/evidence/p3k-output-root-descriptor-pycompile-m384-20261021.log`.

## M441 — Descriptor-relative candidate input reads

P3K now traverses the candidate root through opened directory descriptors and
no-follow file opens. `_patch_series` reads each patch's bytes once, verifies
the digest, and carries those cached bytes through strict application and the
post-check so a pathname swap cannot change which patch bytes are consumed.
Schema 2 and schema 3 path-swap regressions cover this read/hash/apply boundary.
The change also fixes `_patch_series` control flow so valid manifests reach
their schema-specific validation path.

The focused patch-application suite passed 29/29 tests, including both schema
2 and schema 3 path-swap cases; Python bytecode compilation passed. These are
local synthetic regression checks, not a candidate promotion or application.
Evidence: `target/headstart/evidence/p3k-m441-tests.log` and
`target/headstart/evidence/p3k-m441-pycompile.log`.

Descriptor-relative no-follow traversal requires `O_NOFOLLOW` and the required
`dir_fd` operations. On platforms without those facilities, P3K fails closed
with an unsupported-platform error; it does not fall back to path-based reads.

## M824 — Reject oversized unified-diff hunk numbers cleanly

Python can raise `ValueError` when converting a hunk number that exceeds its
integer-string digit limit. P3K now turns that malformed hunk into a normal
`PatchApplicationError`, retaining the rejected run's `result.json` as its
contract requires. The focused patch-application suite passed 33/33 tests,
including a regression with a 5,000-digit hunk number. This is local input and
failure-record integrity evidence only; candidate build, correctness, GUI,
performance, approval, and promotion gates remain open.


## M450 — Descriptor-relative manifest reads

P3K now reads the candidate manifest through a no-follow file open relative
to an opened candidate-root descriptor. This closes the race where the input
manifest could be replaced with a symlink after lexical path validation but
before `Path.read_bytes` followed it. The initial read and final integrity
check now use the pinned candidate-root descriptor and reject no-follow open
failures. Focused regressions swap in an outside symlink after path resolution
and during post-check; the former is rejected before output creation, and the
latter records a rejected result. The patch-application suite passed 31/31, and
Python bytecode compilation passed. Evidence:
`target/headstart/evidence/p3k-m450-manifest-descriptor-tests-20261021.log` and
`target/headstart/evidence/p3k-m450-manifest-descriptor-pycompile-20261021.log`.
This remains local path-integrity evidence; no candidate was run or promoted.

## M955 — Require ASCII unified-diff hunk numbers

P3K's hunk-header regex now accepts only ASCII decimal digits. Python's
Unicode-aware `\d` had allowed a header using Arabic-Indic digits to pass the
strict parser and apply successfully, although those digits are outside the
unified-diff header grammar. A focused regression confirms the malformed patch
is rejected and its `result.json` records the failure. This only tightens input
syntax; candidate acceptance, build/correctness, GUI/performance, approval, and
promotion remain open.
