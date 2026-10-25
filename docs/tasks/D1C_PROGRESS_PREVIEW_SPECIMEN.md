# D1c: Progress preview specimen

The `progress` preview inspector exposes the public Progress settings:

- `label`: accessible operation name.
- `value`: finite percentage from 0 to 100.
- `indeterminate`: boolean mode, defaulting to `false`.

The compiled factory forwards `indeterminate` through
`Progress(...).indeterminate(...)`. The focused regression
`progress_preview_preserves_f64_descriptor_and_bounds_feedback` checks the
boolean schema default and descriptor round-trip alongside the existing value
bounds and f64 preservation.

## Verification

- Focused locked/offline Playground regression passed (1 test).
- GUI and native accessibility review were not run.
