# M838: Require integer schema versions in performance evidence

The M276 performance evidence validator compared summary and run-index schema
versions with integer `1` using equality alone. Python considers JSON `true`
equal to integer `1`, so malformed identity records could pass those checks.

Both checks now require the value's exact type to be `int` before comparing it
with version 1. Focused regressions verify that boolean summary and run-index
versions are rejected. This remains a local measurement-integrity check; it
does not establish native performance acceptance or authorize promotion.
