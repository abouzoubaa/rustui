# M306 — Stage 10 / P3M follow-up gate evidence aggregation

## Scope

P3M extends the read-only P3L candidate evidence report with optional,
hash-bound M274 build and M276 correctness follow-up summaries. The repeatable
`--followup-summary` option accepts those summaries under the candidate root.

An M274 summary can add only `build=passed` after validating candidate and
manifest identity, the exact four baseline/candidate check/build events, their
successful statuses, and their event/log hashes. An M276 summary requires the
supplied M274 summary, verifies its path and digest link, and adds only
`correctness=passed` after checking its success event and raw-log digest. M276
cannot be supplied without M274. Duplicate summaries, identity conflicts,
broken links, unsafe/missing artifacts, and digest drift fail closed.

This reconciles local evidence integrity and gate-state consistency. It does
not execute the candidate pipeline, accept a candidate, run GUI or performance
gates, or promote a patch. Promotion remains `not_run`. Calls without a
follow-up summary retain the original M297/P3L behavior.

## Invocation

```sh
python3 scripts/aggregate_candidate_evidence.py \
  target/headstart/candidates/<candidate-id>/evidence/pipeline-result.json \
  --root target/headstart/candidates/<candidate-id> \
  --followup-summary evidence/m274-build-gate-summary.json \
  --followup-summary evidence/m276-build-correctness-gate-summary.json
```

The paths are relative to the candidate root. The output names each supplied
summary by milestone, path, and SHA-256; only independently validated gates are
carried into `gate_states`. Any invalid follow-up makes the report `invalid`
and leaves all reported gates `not_run`.

## Verification

- Focused tests: **17/17 passed**, including valid linked records, candidate
  and manifest mismatches, broken M274 linkage, event/log digest drift, and
  missing M274. Evidence: `target/headstart/evidence/m306-p3m-tests-20261020.log`.
- Syntax validation: `py_compile` passed for the aggregator and focused test.
  Evidence: `target/headstart/evidence/m306-p3m-pycompile-20261020.log`.
- Read-only M272 reconciliation: **consistent**, 8 original events,
  `build=passed`, `correctness=passed`, and GUI, performance, and promotion
  `not_run`. Evidence:
  `target/headstart/evidence/m306-p3m-linked-followup-smoke-20261020.json`.

These checks establish local record consistency only. Candidate acceptance,
GUI/performance acceptance, and promotion remain separate gates.

## M317 — M276 correctness source identity binding

The M276 follow-up now requires `source_identity` to name the baseline and
candidate scene paths and SHA-256 digests. The aggregator verifies each path
is beneath the corresponding manifest source root, checks the digest against
the snapshot file inventory, and hashes the current file through the existing
descriptor-relative no-follow reader. Missing identities, path escapes,
manifest mismatches, or source drift invalidate the report and leave all gates
`not_run`. This prevents a valid old correctness event from being reused after
the candidate source changes. It does not add build, GUI, performance, or
promotion evidence.

Focused regressions cover a source changed after the recorded correctness run
and a scene path outside the paired source root. The full focused suite passed
19/19; syntax validation passed. The M272 read-only reconciliation remains
`consistent` with build and correctness passed and GUI, performance, and
promotion `not_run`. Evidence:
`target/headstart/evidence/p3m-m276-source-identity-tests-20261020.log`,
`target/headstart/evidence/p3m-m276-source-identity-m272-smoke-20261020.json`,
and `target/headstart/evidence/p3m-m276-source-identity-pycompile-20261020.log`.

## M337 candidate — Bind M274 event records to their summary slots

M274 event records are bound to their summary slots using the canonical `step`
strings emitted by the baseline and candidate build records. Reusing one
successful candidate event across all four slots fails closed. The initial
implementation compared `step` directly to the slot key, which rejected the
recorded M272 baseline and candidate events; M418 corrects that mapping while
retaining exact per-slot equality.

Each M274 summary event must also record an integer `exit_status` matching its hash-bound event file. A regression confirms that a summary claiming exit status 1 for a successful event fails closed and leaves all gates `not_run`; the focused suite now passes 22 tests and `py_compile` passes. This remains local evidence consistency only.

## M418 candidate — Bind M274 follow-up events to side identities and summary slots

Baseline events must carry the paired `candidate_id`; candidate events must
carry the candidate `manifest_sha256`. The aggregator rejects omitted or
transplanted side identities. It maps each summary slot to its canonical event
`step` string and requires exact equality, preserving fail-closed behavior for
wrong-slot event reuse. Focused regressions cover valid asymmetric records,
missing/transplanted identities, and event reuse across slots. The focused
suite passed 26/26; Python syntax validation passed. The historical M272
follow-up reconciles as `consistent` across eight events: build and correctness
passed; GUI, performance, and promotion remain `not_run`. This establishes
local evidence consistency only, not candidate acceptance. Evidence:
`target/headstart/evidence/p3m-event-side-identity-m418-tests-20261021.log`,
`target/headstart/evidence/p3m-event-side-identity-m418-pycompile-20261021.log`,
and `target/headstart/evidence/p3m-event-side-identity-m418-m272-smoke-20261021.json`.

## Follow-up audit — M276 event status consistency (2026-10-21)

An M276 event with an explicit `status` must record `passed`. Previously, a
hash-bound event with `exit_status=0` and a matching raw-log digest could carry
`status=failed`, `not_run`, or `null` and still add `correctness=passed`. The
aggregator now rejects those contradictions and leaves every reported gate
`not_run`. Omitted status remains supported for the historical M276 event
format; its milestone, integer zero exit status, log hash, summary identity,
linked M274 summary, and current scene identities are still required.

The regression reproduced all three contradictory values before the guard.
The focused suite then passed **27/27** and `py_compile` passed for the
aggregator and test module. Evidence:
`target/headstart/evidence/p3m-m276-event-status-audit-20261021-red.log`,
`target/headstart/evidence/p3m-m276-event-status-audit-20261021-tests.log`, and
`target/headstart/evidence/p3m-m276-event-status-audit-20261021-pycompile.log`.
This audit ran synthetic evidence checks only; candidate acceptance, GUI,
performance, and promotion remain separate gates.
