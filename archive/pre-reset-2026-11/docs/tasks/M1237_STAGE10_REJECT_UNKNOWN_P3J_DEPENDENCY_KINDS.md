# M1237 — Reject unsupported dependency kinds in P3J comparisons

P3J now uses Cargo's allowed dependency-kind values (`null`, `build`, and
`dev`) when comparing candidate sources and rejects unsupported labels or
types. The new malformed-kind regression plus existing kind-only and
target-only comparison cases passed (3/3); the validator and test module also
passed `py_compile`.

This validates comparison input shape; it does not establish candidate
equivalence or acceptance. Candidate execution, GUI/performance evidence,
independent review, approval, and promotion remain open.
