# M1264 — Stage 10: bound P3H Cargo metadata output

The dependency-boundary checker now drains Cargo metadata stdout and stderr
concurrently and retains at most 32 MiB combined. If Cargo exceeds that bound,
the checker terminates it and reports a bounded-output error. Successful
stdout still passes directly to the JSON parser, and failing stderr remains
available for the diagnostic.

The real-child regression writes concurrently to stdout and stderr beyond the
33,554,432-byte cap, then verifies capture returns with a bounded-output error,
the child is killed and reaped, and the pipe readers do not deadlock. The
2026-10-24 run of `python3 -m unittest scripts.tests.test_verify_dependency_boundary -v`
passed (8/8). M1265 separately covers the candidate-source reporter's live-child
overflow path.

This closes only the local metadata-capture resource gap. Candidate execution,
correctness evidence, GUI/performance acceptance, independent review,
approval, and promotion remain open.
