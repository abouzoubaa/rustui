# M1234 — Require explicit pass status in P3M follow-up events

P3M follow-up verification no longer treats a missing event `status` as
`passed`. The regression removes the status while refreshing the bound event
and summary hashes, then confirms aggregation rejects the report. A valid
linked M274/M276 follow-up compatibility check also passed; focused tests
passed (2/2).

This hardens evidence integrity only. Benchmark execution, performance
acceptance criteria, GUI evidence, independent review, approval, and promotion
remain open.
