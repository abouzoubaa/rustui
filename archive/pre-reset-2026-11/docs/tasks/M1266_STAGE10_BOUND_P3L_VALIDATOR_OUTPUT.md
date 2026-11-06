# M1266 — Stage 10 bound P3L validator output

`run_step` previously retained complete validator stdout and stderr in memory
until each command exited. It now drains output incrementally with a combined
64 MiB ceiling, terminates the validator process group on overflow, and records
the failure in the bounded command log. P3H stderr remains separate from its
machine-readable stdout; overflowed or timed-out P3H output is not saved as a
report. Timeout events retain their existing failed status and bounded partial
log.

Focused pipeline checks passed 6/6, covering timeout evidence, exact-limit
success, live combined-stream overflow and termination, P3H report/log
separation, pinned descriptor context, and refusal to overwrite a symlinked
P3H report. Candidate execution, correctness, GUI/performance acceptance,
independent review, approval, and promotion remain open.
