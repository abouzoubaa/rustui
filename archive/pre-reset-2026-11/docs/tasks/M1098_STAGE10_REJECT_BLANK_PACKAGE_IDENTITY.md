# M1098 — Stage 10: reject blank Cargo package identity fields

The source comparator now rejects whitespace-only Cargo package names and
versions in otherwise well-formed source reports.

The focused regression passed for both fields. No full comparison or candidate
run was performed; candidate evidence review, approval, and promotion remain
open.
