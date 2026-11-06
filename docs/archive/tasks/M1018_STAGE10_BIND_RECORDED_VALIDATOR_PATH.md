# M1018 — bind Stage 10 evidence to the recorded validator path

P3M now checks a pipeline command against the validator beside the active
aggregator. A command naming an unrelated file with the same basename no
longer produces a `consistent` evidence report. The focused regression
`test_command_must_match_event_step_validator` passed 1/1; the existing
consistent-record regression remains the corresponding valid-path coverage.

This hardens provenance validation only. No candidate was run; candidate
build/correctness, GUI/performance evidence, review, approval, and promotion
remain open.
