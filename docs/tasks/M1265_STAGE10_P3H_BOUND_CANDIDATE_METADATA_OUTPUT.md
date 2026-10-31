# M1265 — Stage 10: bound P3H candidate-source metadata output

The candidate-source reporter drains Cargo metadata stdout and stderr
concurrently for both standalone and descriptor-pinned reports. It retains at
most 32 MiB combined, terminates Cargo after overflow, and routes the bounded
result through the existing process diagnostics and JSON validation. The
pinned path continues to pass its workspace descriptor; the standalone path
continues to use its isolated working directory and environment.

The final JSON written to stdout has a separate 32 MiB UTF-8 byte limit,
including its trailing newline. Both report entry paths use the shared emitter.
If a report exceeds this limit, the reporter emits a compact schema-versioned
invalid JSON report and exits nonzero instead of writing a truncated report.

The focused reporter suite passed 40/40 tests. Regressions accept metadata
output exactly at the configured limit and verify live-child termination on
combined stdout/stderr overflow with a patched 16-byte limit, both directly
through the capture helper and through standalone and descriptor-pinned report
entry paths. Report regressions accept deterministic JSON exactly at the output
limit and verify bounded machine-readable failure through both report entry
paths when serialized JSON exceeds it.

These bounds close the reporter's metadata-capture and final JSON output
resource gaps only. Candidate execution, correctness evidence, GUI/performance
acceptance, independent review, approval, and promotion remain open.
