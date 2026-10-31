# M973 — Stage 10 / reject evidence JSON numbers outside finite float range

## Change

Candidate evidence JSON parsing now rejects numeric tokens that Python would
convert to non-finite floats, including numbers inside permissive provenance
fields. P3D visual-evidence parsing applies the same finite-float check to
both visual manifests and referenced artifact JSON.

## Focused evidence

`test_rejects_json_numbers_outside_finite_float_range` passed (1/1).
`test_rejects_float_overflow_numbers_in_visual_and_provenance` passed (1/1),
covering `1e10000` in both parsing paths.

## Remaining gates

This closes the parsing-integrity defect only. Candidate execution, correctness
and build evidence, review, approval, and promotion remain open.
