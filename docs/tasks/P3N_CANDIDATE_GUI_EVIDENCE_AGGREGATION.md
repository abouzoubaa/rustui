# P3N — Candidate-bound paired GUI evidence aggregation

## Contract

The existing P3M `--followup-summary` path now accepts an additive M287 GUI
summary with `schema_version: 1`. Existing P3L, M274, and M276 input records
retain their existing schemas. The M287 summary is read-only evidence; it does
not launch an application, run a candidate, accept a patch, or promote it.

M287 requires the pipeline result's exact `candidate_id`, `manifest_sha256`,
and `source_hash_manifest_sha256`. It also links the supplied M274 build summary
by candidate-root-relative path and SHA-256. Each baseline/candidate source tree
digest must match that side's entry in the run's hash-bound
`source-hash-manifest.json`. This prevents the disposable overlay comparison
under M283 from being reported as GUI acceptance for the unchanged M272
candidate.

The summary has exactly these fields:

```json
{
  "schema_version": 1,
  "milestone": "M287",
  "candidate_id": "<pipeline candidate_id>",
  "manifest_sha256": "<pipeline manifest_sha256>",
  "source_hash_manifest_sha256": "<pipeline source_hash_manifest_sha256>",
  "build_summary_path": "evidence/m274-build-gate-summary.json",
  "build_summary_sha256": "<M274 summary SHA-256>",
  "gate_states": {
    "visual": "passed",
    "accessibility": "passed",
    "gui": "passed"
  },
  "snapshots": {
    "baseline": {
      "source_tree_sha256": "<source-hash-manifest baseline tree_sha256>",
      "screenshot_state": "passed",
      "screenshot": {"path": "evidence/gui/baseline.png", "sha256": "<SHA-256>"},
      "accessibility_state": "passed",
      "accessibility_tree": {"path": "evidence/gui/baseline-ax.json", "sha256": "<SHA-256>"}
    },
    "candidate": {
      "source_tree_sha256": "<source-hash-manifest candidate tree_sha256>",
      "screenshot_state": "passed",
      "screenshot": {"path": "evidence/gui/candidate.png", "sha256": "<SHA-256>"},
      "accessibility_state": "passed",
      "accessibility_tree": {"path": "evidence/gui/candidate-ax.json", "sha256": "<SHA-256>"}
    }
  }
}
```

All artifact paths are safe relative paths beneath the candidate root and are
read with the existing descriptor-relative no-follow reader. `not_run`,
`failed`, and `inconclusive` states are retained. A `passed` visual comparison
requires both screenshot captures to pass; a `passed` accessibility comparison
requires both AX captures to pass. `gui` is derived: it passes only when both
comparisons pass, and otherwise remains `not_run`, `failed`, or `inconclusive`
according to the recorded comparison states. Artifact references may be null
for non-passed captures; any supplied reference is still hash-checked. The
aggregate report retains side tree hashes, states, and screenshot/AX paths and
digests under the M287 follow-up record. Baseline and candidate screenshot paths
must differ, and baseline and candidate AX-tree paths must differ. The file
contents may be identical when each side uses its own path.

This is local record consistency only. The aggregator does not inspect pixels
or interpret AX semantics; those observations remain review inputs. The M283
overlay screenshots have no retained AX dump and are not bound to the M272
source-manifest digest, so they do not satisfy this contract.

## Invocation

```sh
python3 scripts/aggregate_candidate_evidence.py \
  target/headstart/candidates/<candidate-id>/evidence/pipeline-result.json \
  --root target/headstart/candidates/<candidate-id> \
  --followup-summary evidence/m274-build-gate-summary.json \
  --followup-summary evidence/m287-gui-gate-summary.json
```

Without M287, prior P3L/P3M behavior and accepted summary schemas are
unchanged. Supplying an M287 record without its linked M274 record, with a
transplanted candidate/source manifest identity, or with a missing/mismatched
artifact returns `invalid` and leaves reported gates `not_run`.

## Verification

Focused synthetic tests cover a valid pair, explicit `not_run`, identity and
tree-digest transplants, artifact drift, missing captures, and the required
M274 link. They do not run a candidate or establish native GUI acceptance.

- Focused P3L/P3M/P3N plus pipeline tests: **58/58 passed**.
- `py_compile` passed for the aggregator, pipeline, and both focused test
  modules.
- M514 paired-path regressions and the full focused aggregation module passed
  **35/35**; `py_compile` passed for the aggregator and focused test module.
- Read-only aggregation of existing M272 M274/M276 summaries remained
  `consistent`; build/correctness are passed, GUI/performance/promotion remain
  `not_run`. No M287 summary was applied to M272 because the retained M283/M287
  evidence comes from overlay copies and lacks AX artifact files.

## M524 distinct artifact files

Baseline and candidate screenshot or accessibility-tree references must resolve to distinct regular files, in addition to using distinct relative paths. The validator compares device and inode captured from the same safely opened descriptors used to read and hash the artifacts. Separate copies with identical bytes remain valid. Focused aggregation tests passed 37/37, including hard-link aliases for both artifact kinds. Evidence: `target/headstart/evidence/m524-artifact-file-identity-tests-20261021.log`.

## M528 cross-role artifact identity

All non-null M287 screenshot and accessibility-tree artifacts across baseline
and candidate must now identify distinct files, even when the duplicate is
between different artifact roles. Regression cases reject same-snapshot path
reuse and cross-side cross-role hard links. The focused aggregation suite
passed 39/39 and `py_compile` passed. This local record-integrity check does
not prove captured GUI or accessibility content is correct. Evidence:
`target/headstart/evidence/m528-p3n-cross-role-artifact-identity-tests-20261021.log`.
