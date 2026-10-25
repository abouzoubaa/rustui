# M1236 — Reject unknown P3H Cargo dependency kinds

P3H metadata validation now accepts only Cargo's `null` (normal), `dev`, and
`build` dependency-kind values. Arbitrary nonempty labels are rejected. The
focused regression passed (1/1):

```sh
python3 -m unittest \
  scripts.tests.test_report_candidate_sources.ReportCandidateSourcesTests.test_rejects_unknown_dependency_kind
```

This tightens metadata validation only. Candidate execution, full graph
equivalence, GUI/performance evidence, independent review, approval, and
promotion remain separate gates.
