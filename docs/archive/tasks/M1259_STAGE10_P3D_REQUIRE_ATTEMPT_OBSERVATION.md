# M1259 — Stage 10: require observations for attempted P3D states

The P3D evidence validator now requires an observation record for `failed`
and `inconclusive` states even when no screenshot was produced. `not_run`
states remain exempt and continue to be reported as incomplete.

The focused regression exercised both attempted statuses without screenshots,
then supplied observations and verified that the evidence remains incomplete
rather than being accepted as captured. It passed 1/1.

This tightens local evidence-record completeness only. It does not establish
candidate execution, current-source visual acceptance, correctness,
performance, independent review, approval, or promotion.
