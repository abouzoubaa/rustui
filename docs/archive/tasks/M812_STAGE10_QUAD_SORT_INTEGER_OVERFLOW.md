# M812: quad-sort evidence integer overflow

## Scope

Keep oversized JSON timing integers inside the quad-sort validator's ordinary
invalid-evidence path. Python's `math.isfinite` can raise `OverflowError` when
passed an integer too large to convert to a float.

## Change

The numeric comparison helper now catches that conversion overflow and rejects
the evidence as invalid instead of allowing a traceback to escape validation.

## Evidence

- `python3 scripts/tests/test_validate_quad_sort_performance.py -k out_of_range`
  — 1 focused regression passed.

Candidate provenance, build/correctness, GUI/performance acceptance, review,
approval, and promotion remain open.
