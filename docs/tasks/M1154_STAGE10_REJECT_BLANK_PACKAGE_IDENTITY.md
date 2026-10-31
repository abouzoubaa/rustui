# M1154 — Stage 10 P3H reject blank package identity fields

## Contract

P3H source reports must validate Cargo package identities before using them in
the normalized dependency graph. Package names and versions must be nonempty
strings; empty and whitespace-only values are invalid.

## Change

The P3H source reporter now rejects empty or whitespace-only `name` and
`version` values instead of emitting a valid report with an incomplete graph
package identity.

## Evidence

`python3 -m unittest
scripts.tests.test_report_candidate_sources.ReportCandidateSourcesTests.test_rejects_blank_package_identity_fields`
passed (1 test), covering empty and whitespace-only names and versions. The
valid-graph reporter smoke test
`test_reports_candidate_path_and_vendored_registry_sources` passed (1 test).
`git diff --check` reported no whitespace errors.

This closes a P3H metadata validation gap; no candidate was run or accepted.
