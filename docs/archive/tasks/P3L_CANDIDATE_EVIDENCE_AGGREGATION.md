# M297 — Stage 10 / P3L candidate evidence aggregation

## Scope

`scripts/aggregate_candidate_evidence.py` reads an existing P3 pipeline result,
its hash-bound event records and logs, the candidate manifests, and the P3F
evidence record. It checks path and digest provenance, verifies that event-file
contents match the events embedded in the pipeline result, and reconciles the
four P3F gate states with the pipeline result.

The P3F record's observed `exit_status` must agree with the pipeline outcome: `partial` requires 0 and `failed` requires 1.
The JSON report uses `consistent` or `invalid` to describe record consistency.
Gate values are copied only when the records agree. Missing or invalid evidence
leaves every reported gate at `not_run`; promotion is always reported as
`not_run`. The tool does not run commands, edit evidence, authenticate approval,
interpret validation events as candidate acceptance, or make promotion
decisions. `consistent` means only that the supplied local records agree.

## Invocation

```sh
python3 scripts/aggregate_candidate_evidence.py \
  target/headstart/candidates/<candidate-id>/evidence/pipeline-result.json \
  --root target/headstart/candidates/<candidate-id>
```

The P3F record defaults to `evidence/P3F-record.json`. The `--p3f-record` option
can select another record beneath the same root. Duplicate JSON keys, unsafe
paths, symlinked record aliases or artifacts, hash mismatches, manifest
conflicts, contradictory gate states, and invalid P3F evidence fail closed with
an `invalid` report. Event JSON uses type-sensitive value comparison, so a JSON
boolean cannot be mistaken for an integer during reconciliation.

Evidence reads start from a pinned root directory descriptor and walk each path
component with descriptor-relative opens, `O_NOFOLLOW`, and `fstat` checks. The
final file must be a regular file. On platforms without descriptor-relative
`os.open` and `os.stat`, `os.stat(..., follow_symlinks=False)`, or `O_NOFOLLOW`,
reads fail closed with an invalid report instead of falling back to pathname
reads.

## Verification

- Focused tests: **10/10 passed**. Coverage includes consistent regular-file
  reads, gate disagreement, promotion claims, event hash/content drift,
  symlinked files and parent directories, external record aliases,
  type-sensitive event comparison, and missing P3F evidence. Log:
  `target/headstart/evidence/m297-p3l-tests.log`.
- Syntax validation: `py_compile` passed for the aggregator and its test module.
  Log: `target/headstart/evidence/m297-p3l-pycompile.log`.
- Read-only M272 smoke: **consistent**, with 8 events. Build, correctness, GUI,
  performance, and promotion all remain `not_run`. Output:
  `target/headstart/evidence/m297-p3l-m272-smoke.json`.
- Whitespace check: passed for the aggregator, its test module, and this task
  note using `git diff --no-index --check`.

This report is not an acceptance gate. Build, correctness, GUI, performance,
and promotion remain independently reviewed states; absent evidence remains
`not_run`.

## M326 — Failed command event log binding

Every pipeline event that records a command must now include a hash-bound log,
regardless of whether the command passed or failed. Previously, a failed
command event could omit its log and still produce a `consistent` aggregation.
The aggregator now rejects that incomplete event and leaves all reported gates
`not_run`. This tightens local evidence integrity without treating logs or
validator outcomes as candidate acceptance.

- Focused tests: **20/20 passed**, including a regression for a failed command
  event with a hash-bound event record but no log. Evidence:
  `target/headstart/evidence/m326-p3l-tests-20261021.log`.
- Syntax validation: `py_compile` passed. Evidence:
  `target/headstart/evidence/m326-p3l-pycompile-20261021.log`.
- Read-only M272 reconciliation: **consistent**, 8 events; build, correctness,
  GUI, performance, and promotion remain `not_run`. Evidence:
  `target/headstart/evidence/m326-p3l-m272-smoke-20261021.json`.

These checks establish local record consistency only. Candidate acceptance,
GUI/performance acceptance, and promotion remain separate gates.

## M624 — Bound pipeline metadata records

P3L reads metadata records with a 1 MiB cap and rejects oversized records before decoding or parsing. The focused oversized-record regression passed 1/1. This protects local aggregation input only; it does not run or approve a candidate.

## M633 — Bound pipeline event records

P3L applies the same 1 MiB cap to pipeline event JSON before decoding. The focused regression updates the event digest and confirms oversized event records are rejected: `python3 scripts/tests/test_aggregate_candidate_evidence.py AggregateCandidateEvidenceTests.test_pipeline_event_record_size_is_bounded` (**1 passed**). This protects local aggregation input only; candidate execution and approval remain separate gates.
