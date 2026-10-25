# M1267 — Stage 10 bound dependency-reuse JSONL input

The P2B dependency-reuse verifier previously read, decoded, and split the
entire Cargo JSONL log without a size limit. It now reads at most 64 MiB plus
one detection byte and rejects larger logs before parsing or writing a report.
The accepted bytes remain the source for `log_sha256`.

Focused verifier tests passed 3/3, including exact-limit hash preservation and
oversized-input rejection without report creation. This is local verifier
resource hardening only; no Cargo build or candidate acceptance is implied.
