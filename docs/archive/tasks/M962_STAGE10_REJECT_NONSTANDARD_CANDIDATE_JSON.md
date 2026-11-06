# M962 — Stage 10 / reject non-standard JSON in candidate manifests

P3K candidate-manifest parsing now rejects Python's non-standard `NaN`,
`Infinity`, and `-Infinity` constants. The focused patch-application suite
passed 37/37. This hardens manifest parsing only; candidate correctness/build,
GUI/performance, review, approval, and promotion remain open.
