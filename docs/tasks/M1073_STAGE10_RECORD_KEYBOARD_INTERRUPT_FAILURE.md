# M1073 — Stage 10 / record candidate pipeline interruption failures

The candidate pipeline now records failure evidence when it is interrupted by
`KeyboardInterrupt`, then re-raises the interruption. Previously, candidate
creation could be followed by an unrecorded partial run because the handler
caught only `Exception`. A focused regression confirmed both failure-record
creation and propagation of the original interruption (1/1). No candidate
validator, build, or promotion was run.
