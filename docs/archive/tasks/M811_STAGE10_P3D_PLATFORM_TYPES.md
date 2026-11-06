# M811: P3D platform metadata types

## Scope

Require stable platform, appearance, and display identifiers in P3D visual
evidence. JSON permits nonfinite values such as `NaN` in Python's decoder;
the old shared numeric-or-string check could therefore accept malformed
platform metadata.

## Change

The evidence validator now requires these three fields to be nonblank strings.
Scale factors retain their separate finite numeric validation.

## Evidence

- `python3 scripts/tests/test_validate_p3d_visual_evidence.py -k platform` — 1
  focused regression passed.

Native visual capture, controlled performance evidence, review, approval, and
promotion remain open.
