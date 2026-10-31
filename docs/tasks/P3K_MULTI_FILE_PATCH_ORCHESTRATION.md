# P3K schema 3: ordered multi-file candidate patch application

**Status: bounded implementation.** `scripts/apply_candidate_patch.py` retains
schema 1 single-patch and schema 2 ordered same-file compatibility. Schema 3
applies an ordered series to multiple existing files inside the prepared
`gpui-pre-0.3.9` package, using only a fresh disposable copy of the baseline.

## Schema 3 contract

The manifest has `schema_version: 3`, the existing candidate identity and
baseline/candidate snapshot records, `patch_sha256`, and an ordered `patches`
array. Each entry contains exactly `path`, `file`, `sha256`,
`preimage_sha256`, and `postimage_sha256`. `path` is a unique safe patch filename;
`file` is a slash-separated package-relative path. `patch_sha256` hashes the
UTF-8 newline-joined ordered `filename:patch_sha256` entries, as in schema 2.
Each patch header must name exactly `vendor/gpui-pre-0.3.9/<file>` and every
hunk must match its declared line and exact source bytes.

The series must name exactly the files whose hashes differ between the
prepared baseline and candidate package inventories. Additions, deletions,
symlinks, and non-regular files remain unsupported and fail closed. Each entry's
preimage must match the prior series state for that file; each postimage is
checked before the next entry. The final complete disposable package inventory
must equal the prepared candidate inventory. Result records include patch path,
target path, patch hash, per-patch preimage/postimage hashes, and each changed
file's baseline and final hashes.

Schema 3 stages all patch outputs in memory before writing. If patching or
post-check integrity fails after copy creation, the disposable package copy is
removed and the retained `result.json` records `rollback_status`; a failed
rollback is reported explicitly. Prepared snapshots, active workspace sources,
`vendor/`, Cargo caches, and promotion state are not modified. Source intake
accepts schema 3 and retains its ordered patch declarations. P3E's separate
patch-plan schema remains unchanged; it binds approved patch files and hashes,
not candidate-package target files.

## Evidence and limits

The focused application suite passed 18 tests, including schema 1/2
compatibility, schema 3 two-file application with path/hash/preimage/postimage
evidence, path traversal rejection, and rollback after a later exact-hunk
failure or a diff-header target mismatch. The 22-test candidate-intake suite
also passed with schema 3 source-manifest bindings. Raw outputs are
`target/headstart/evidence/p3k-multi-file-patch-tests-20261020.log` and
`target/headstart/evidence/p3k-schema3-candidate-intake-tests-20261020.log`.

This validates the orchestration against temporary synthetic package trees. It
does not demonstrate a real multi-file upstream candidate, source provenance,
patch approval, candidate build or correctness, GUI behavior, performance,
acceptance, or promotion. It only edits existing UTF-8 regular files with one
target per patch entry; file creation/deletion and a single unified diff spanning
multiple targets require a separately reviewed schema extension.
## End-to-end validation follow-up — M234

Added focused applicator regressions for repeated-file postimage hash chaining
and rollback after an injected second-file staging failure. The applicator
suite passed 20/20. The full synthetic pipeline suite passed 23/23, including a
new schema-3 two-file case that invokes the real P3K applicator, verifies both
postimages in the disposable copy, and confirms prepared baseline/candidate
snapshots are unchanged. The schema-3 intake suite passed 22/22. Logs:
`target/headstart/evidence/p3k-schema3-multifile-pipeline-tests-20261020.log`,
`target/headstart/evidence/p3k-schema3-multifile-pipeline-case-20261020.log`, and
`target/headroom/evidence/p3k-multi-file-patch-tests-20261020.log`. This remains
synthetic fixture evidence; no real upstream candidate or downstream acceptance
gate was run.

## M708 — reject empty ordered series at pipeline binding

The P3K applicator requires at least one patch, but the candidate pipeline's
plan/manifest binding helper accepted empty ordered lists and deferred rejection
until application. The pipeline now rejects an empty list while binding either
the P3E plan or schema-3 source manifest. A focused regression covers both
inputs and passed 1/1; scoped whitespace validation passed. Evidence:
`target/headstart/evidence/m708-p3k-nonempty-series-binding.log`.

This only moves malformed-input rejection to the binding boundary. Candidate
provenance, real upstream patch application, build/correctness/GUI/performance
evidence, and promotion remain separate gates.
