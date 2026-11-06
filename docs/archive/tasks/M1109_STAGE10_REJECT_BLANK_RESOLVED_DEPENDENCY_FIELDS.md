# M1109 — Stage 10 P3H reject blank dependency metadata

## Contract

P3H source reports must validate normalized resolved dependency edges. Optional
Cargo dependency-kind and target fields may be `null`, but present string
values must not be blank.

## Change

Metadata validation now rejects empty or whitespace-only dependency-kind and
target strings rather than reporting the resolved edge as valid.

## Evidence

`python3 scripts/tests/test_report_candidate_sources.py` passed (21 tests),
including `test_rejects_blank_dependency_kind_metadata`. `git diff --check`
reported no whitespace errors.

This improves P3H source-report validation; no candidate was run or accepted.
