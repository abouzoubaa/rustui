# P3 local candidate intake and validation chain

**Status: bounded local intake and metadata-validation chain implemented.** This
runner consumes an already-prepared baseline/candidate source pair. It does not
fetch or construct upstream source snapshots, modify `vendor/`, run candidate
builds or correctness suites, perform native GUI or performance acceptance, or
promote a candidate.

## Approved intake record

Run `scripts/run_candidate_pipeline.py <intake.json> --root <repository>` with
an intake JSON object containing exactly:

- `schema_version`: integer `1` (Boolean values are rejected).
- `approved`: literal `true` and `approval_reference`: a non-empty recorded
  approval reference. These are input claims; the runner does not authenticate
  who approved them.
- `candidate_id`: a new safe destination ID, distinct from
  `source_candidate_id`.
- `source_candidate_id`: the ID in the prepared snapshot manifest.
- `source_manifest`: repository-relative `path` and lowercase SHA-256 of the
  prepared manifest.
- `patch_plan`: `null`, or a hash-pinned path relative to the prepared
  manifest's directory. A null plan is recorded as P3E `not_run` and stops
  before source snapshot copying or P3G/P3H/P3I/J/P3K checks.
- `purpose`: a non-empty description.

The approved P3E plan must identify the prepared source candidate. Its ordered
patch path/hash pairs must exactly match the prepared manifest's patch
declaration before patch or source bytes are copied. P3E failure or mismatch
leaves failure evidence and stops the run. Prepared source inventories and
symlink boundaries are rechecked during copying.

## Local stage order and outputs

The runner creates a new destination under
`target/headstart/candidates/<candidate_id>/`, records the approved intake and
plan, then performs P3E plan validation and patch-declaration binding before
copying prepared snapshots. It next runs P3G snapshot validation, P3H reports
for baseline and candidate, P3I/P3J source and graph comparisons when their
report schemas support them, P3K exact patch reproduction in its disposable
baseline-package copy, and P3F local evidence-record validation.

Each invoked stage has a log and event record with hashes. Failures preserve the
partial destination and write `pipeline-failure.json` where possible; failure
recording errors do not replace the original exception. Successful completion
of these local checks is labeled `partial`. Build, correctness, GUI,
performance, and promotion gates remain `not_run`. `pipeline-result.json`
references the stage events; the P3F record validates a bounded set of local
artifacts and is not a full candidate-acceptance decision.

The P3F record's `provenance.approved_intake_sha256` hashes the exact intake
bytes parsed by the runner. Those bytes are retained as `approved-intake.json`
and included in the P3F artifact list, making the recorded approval input
locally integrity-checkable alongside the copied manifests.

## Evidence and limits

The focused synthetic suite passed 13 tests and Python syntax compilation passed
on 2026-10-20:

- `target/headstart/evidence/run-candidate-pipeline-tests-20261020.log`
- `target/headstart/evidence/run-candidate-pipeline-pycompile-20261020.log`

The stage-runner tests mock downstream validators. The prepared Quad V2 source
pair was later processed by the real local intake chain at M272, recorded
below. No new candidate revision or patch provenance was invented, and no
build, correctness, GUI, performance, or promotion result is claimed.
Candidate acceptance remains gated by the approved workload and acceptance
inputs in `P3_PATCH_VALIDATION_DESIGN.md`.

## M202 evidence correction

The P3F record now binds `provenance.approved_intake_sha256` to the exact intake bytes parsed by the runner. Those bytes are retained as `approved-intake.json` and included in the P3F artifact list. The focused 13-test pipeline suite and Python syntax compilation passed; see `target/headstart/evidence/run-candidate-pipeline-intake-hash-m202-20261020.log` and `target/headstart/evidence/run-candidate-pipeline-intake-hash-pycompile-m202-20261020.log`.

## M220 — Special-file rejection (2026-10-20)

JSON inputs are opened nonblocking with regular-file and no-follow checks. FIFO
intake, source-manifest, and patch-plan paths are rejected before any read or
candidate-directory creation. The synthetic pipeline suite passed 16/16, and
Python syntax compilation and scoped diff checks passed. The suite does not
exercise a full prepared-candidate run; all downstream candidate gates remain
unrun.

## M224 — Descriptor-pinned source copying (2026-10-20)

Source copying keeps each opened directory descriptor alive through recursive
descent and opens child entries relative to that descriptor with no-follow
flags. Regular files are checked through their opened descriptors before
copying. A regression swaps a directory path after its descriptor is opened
and verifies that an outside file is not copied; the post-copy inventory also
rejects the changed source closure. Final focused evidence is recorded in
[`run-candidate-pipeline-m224-20261020.log`](../../target/headstart/evidence/run-candidate-pipeline-m224-20261020.log):
20 tests passed, both Python files compiled, and the scoped no-index whitespace
check passed for the untracked code, test, note, and evidence files. No full
candidate intake was run.

## M226 — No-follow patch and plan copying (2026-10-20)

The P3 runner now copies the hash-pinned source manifest, patch plan, and
approved patch files through its existing no-follow regular-file copier.
This closes a race where a checked path could be replaced by a symlink before
`copyfile` opened it; a regression swaps the approved patch for a symlink at
that boundary. The focused runner suite passed 21/21, Python syntax
compilation and scoped whitespace checks passed. No full candidate intake or
downstream candidate gates ran. Evidence:
`target/headstart/evidence/run-candidate-pipeline-local-input-copy-followup-20261020.log`.

## M272 — Prepared Quad V2 local intake (2026-10-20)

The local runner processed source candidate `quad-stable-sort-v2-20261020-a`
into destination `quad-stable-sort-v2-chain-m272-20261020`, using intake
`target/headstart/evidence/quad-v2-intake-m272-20261020.json`. P3E plan
validation and binding, P3G, both P3H reports, P3I/J, P3K exact patch
reproduction, and P3F evidence validation each passed with exit status 0. The
pipeline status is `partial`. Build, correctness, GUI, performance, and
promotion remain `not_run`; this validates only local metadata and exact patch
reproduction and does not accept or promote the candidate. Evidence:
`target/headstart/evidence/quad-v2-intake-m272-20261020.log` and
`target/headstart/candidates/quad-stable-sort-v2-chain-m272-20261020/evidence/pipeline-result.json`.

## M274 — Paired Quad V2 build gate (2026-10-20)

After M272 intake, the copied baseline and candidate each passed
`./scripts/cargo-headstart check --workspace --all-targets --locked --offline`
and `./scripts/cargo-headstart build --workspace --all-targets --locked --offline`.
Each invocation used its own `baseline/target` or `candidate/target` directory.
The build gate is passed for this copied pair. Correctness tests were not run;
GUI, performance, and promotion remain open. The M272 `pipeline-result.json`
was not rewritten; follow-up evidence with event and raw-log hashes is in
`target/headstart/candidates/quad-stable-sort-v2-chain-m272-20261020/evidence/m274-build-gate-summary.json`.

## M276 — Quad V2 production correctness (2026-10-20)

An isolated external harness bound to the immutable M272 candidate GPUI source
passed 10/10 locked/offline test-target tests. The production regression runs
`Scene::finish` on 1,024 dense, out-of-order Quads and compares all payload
fields with stable stock-sort output. The first unfiltered command passed its
test targets but failed afterward in unrelated doctests because host rustdoc
1.97.1 could not consume the Headstart-built GPUI artifact. The successful
`test --tests` run excludes doctests. M272 source/manifests remain unchanged;
GUI, performance, and promotion remain open. Hash-bound follow-up evidence is
in `target/headstart/candidates/quad-stable-sort-v2-chain-m272-20261020/evidence/m276-build-correctness-gate-summary.json`.

## M344 — Pipeline failure exit-status evidence

If a local validator fails, the runner now records exit status 1 in the P3F
record for the top-level runner command. A focused regression verifies the
pipeline is failed, the record reports exit status 1, and the record itself
still validates. The runner suite passed 24/24 and `py_compile` passed. No
candidate command ran or was promoted.
Logs: `target/headstart/evidence/run-candidate-pipeline-exit-status-audit-20261021.log`
and `target/headstart/evidence/run-candidate-pipeline-exit-status-pycompile-20261021.log`.

## M626 — Bound local pipeline steps

Each local pipeline subprocess has a 600-second timeout. A timed-out step records exit status 124 and retains partial stdout in the stage log. The focused timeout regression passed. No candidate was run for this change.
