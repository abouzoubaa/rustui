# M946 — Stage 10 / reject non-standard P3D JSON constants

Python's JSON decoder accepts `NaN`, `Infinity`, and `-Infinity` by default.
P3D now rejects these non-standard numeric constants in both the primary visual
evidence record and its referenced build/plan provenance JSON, keeping those
inputs within the JSON contract.

The focused regression rejects `NaN` in the visual record and `Infinity` in a
hash-bound provenance artifact. The regression passed, and `py_compile` passed
for the validator and its test module.

This is evidence-parser integrity only. It does not establish candidate
correctness, build, GUI, performance, review, approval, or promotion.
