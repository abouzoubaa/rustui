# M1225 — Bound M287 GUI artifact reads

## Change

The M287 evidence aggregator reads screenshot and accessibility-tree files into
memory to verify their hashes and inspect PNG structure. These files now have a
64 MiB per-artifact read limit, matching the existing command-log resource
ceiling. The bound is applied before hashing or parsing either artifact.

## Focused evidence

`test_m287_artifact_read_is_bounded` creates a hash-bound screenshot larger
than a patched small test limit and confirms aggregation rejects it.

```sh
python -B -m unittest \
  scripts.tests.test_aggregate_candidate_evidence.AggregateCandidateEvidenceTests.test_m287_artifact_read_is_bounded
```

Result: passed (1/1). Scoped `git diff --check` passed.

## Remaining gates

This change bounds M287 evidence aggregation only. It does not establish GUI or
performance acceptance of a candidate; independent review, approval, and
promotion remain separate gates.
