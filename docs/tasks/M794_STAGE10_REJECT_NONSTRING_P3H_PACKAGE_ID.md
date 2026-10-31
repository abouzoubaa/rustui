# M794 — Stage 10 reject non-string P3H package IDs

The P3I report consumer validates package identity fields used for inventory
comparison, but previously ignored each P3H package's Cargo `id`. That allowed
malformed reports such as `packages[0].id: true` to pass validation. P3H package
IDs are strings; they are required report data even though comparison uses
root-independent normalized identities instead of the IDs themselves.

The consumer now rejects missing, non-string, or blank package IDs. The
focused regression passed:
`python3 scripts/tests/test_compare_candidate_sources.py
CompareCandidateSourcesTests.test_boolean_package_id_is_rejected` (1 test).
