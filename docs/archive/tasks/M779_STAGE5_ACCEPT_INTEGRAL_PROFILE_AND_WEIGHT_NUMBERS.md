# M779 — Stage 5 accept integral JSON numbers for profile and weight fields

Theme deserialization now accepts mathematically integral JSON decimal values
for profile version bounds, typography font weights, and text-style weights,
matching the integer fields in the published schema. Fractional and
out-of-range values remain rejected.

The focused schema regression and scoped `git diff --check` passed.
`rustfmt --check` still reports pre-existing test-formatting differences in
the dirty schema file; those unrelated hunks were preserved. Broader native
visual theme acceptance remains open.
