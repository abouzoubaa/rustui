# M837 Stage 10 P3F root-directory record path rejection

The P3F command accepted `--root` itself as the record path syntactically. Its
relative path then had no path components, so the descriptor-relative loader
raised an uncaught `IndexError` instead of returning the validator's JSON
`invalid` report.

The CLI now rejects a record path that identifies the root directory before
loading it. The focused regression asserts a nonzero exit, an empty stderr, and
a structured invalid report.

Verification: `python3 scripts/tests/test_validate_candidate_evidence.py`.
This checks malformed path handling only; it does not establish candidate
acceptance, provenance, or any human review gate.
