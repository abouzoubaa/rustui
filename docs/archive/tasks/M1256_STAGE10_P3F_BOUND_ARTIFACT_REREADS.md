# M1256 — Stage 10: bound P3F artifact rereads

P3F validates evidence artifacts, after which the aggregator rereads them to
check pinned identities and obtain the approved-intake document. An artifact
could grow between those passes, bypassing the aggregate validation budget and
causing an unbounded allocation on the second read.

The aggregator now enforces the shared 4 GiB artifact budget during that
identity-checking reread and reuses the approved-intake bytes from the same
pass. A focused mutation regression grows an artifact after P3F validation and
confirms the second pass rejects it at the configured byte limit.

The focused aggregate-evidence suite passed **71 tests**; Python bytecode
compilation passed. This is local evidence-read integrity only. No candidate
was executed; candidate correctness, GUI/performance acceptance, independent
review, approval, and promotion remain open.
