# Stage 9 / D1c LazyGrid preview specimen

The canonical compiled-preview gallery exposes an opt-in specimen for the
implemented public RustUI `LazyGrid` API. Its descriptor contains bounded
`columns` (1–4) and `spacing` (0–32) values. Six fixed `LazyGridItem`s, item
identities, viewport and row sizes, and the cell factory remain compiled-local;
the same spacing value controls the native row and column gaps. No closures or
runtime state are serialized.

The focused regression verifies the schema bounds, empty event/action maps,
default descriptor values, and import of three columns with 24.5-point gaps.
The locked/offline focused test passed 1/1 and the Playground all-targets check
passed. Evidence:

- `target/headstart/evidence/d1c-lazy-grid-specimen-test-20261022.log`
- `target/headstart/evidence/d1c-lazy-grid-specimen-check-20261022.log`

No GUI acceptance was run. Native preview layout, scrolling, keyboard and
screen-reader behavior, localization/RTL, and cross-platform acceptance remain
open. A whole-file rustfmt check reports one unrelated formatting hunk in an
existing Checkbox test; the LazyGrid code itself had no rustfmt diff.
