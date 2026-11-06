# M1197 — Stage 10 validate comparison candidate IDs

## Scope

Make the P3I comparison reader enforce the same candidate-ID contract as its
report producer.

## Change

`compare_candidate_sources.py` now rejects candidate IDs that do not match the
producer's bounded safe-identifier format, including path-like values, slash
components, `..`, and overlength values.

## Verification

Focused regression command:

```sh
python3 -m unittest scripts.tests.test_compare_candidate_sources.CompareCandidateSourcesTests.test_unsafe_candidate_id_is_rejected
```

Result: 1 test passed. Candidate execution, independent review, approval,
promotion, and GUI/performance acceptance remain open.
