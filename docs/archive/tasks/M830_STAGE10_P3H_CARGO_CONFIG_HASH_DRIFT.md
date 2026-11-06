# M830 — Stage 10 P3H candidate Cargo config hash stability

P3H recorded candidate Cargo config hashes before invoking Cargo but did not
verify that the config chain remained the same during metadata inspection. A
config edit during the subprocess could therefore produce a valid report whose
hashes did not describe the config Cargo observed. P3H now rechecks path
containment and hashes after metadata returns, rejecting changed configuration.

The focused regression passed:
`python3 scripts/tests/test_report_candidate_sources.py ReportCandidateSourcesTests.test_rejects_cargo_config_hash_drift_during_metadata`
(1 test). This is local source-report integrity evidence only; candidate
acceptance, build/correctness, GUI/performance, review, approval, and promotion
remain open.
