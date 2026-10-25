# D1c DatePicker preview specimen

## Scope and contract

The opt-in `date-picker-specimen` catalog entry renders the public RustUI
`DatePicker` API through the existing compiled preview host. Its descriptor has
an editable bounded `label`, an editable bounded `accessibility_label`, and an
editable `date` string. The accessible-name property defaults to `Start date`,
so it demonstrates a stable name distinct from the visible `Choose a date`
placeholder. An empty accessible name omits the override and falls back to the
visible label. An empty date means no selection; a selected date uses canonical
`YYYY-MM-DD`, with years 0001–9999.
The event catalog and descriptor action map remain empty under the selected
signal-only policy. Calendar changes use the existing controlled binding path.

## Implementation

`apps/rustui-playground/src/stories/compiled_preview_gallery.rs` registers the
schema and compiled factory, adds the specimen launcher, and supplies an optional
`NaiveDate` binding. Accepted selection and clearing feedback update the resolved
serializable property using the current preview stamp. An older generation or a
different mounted component cannot accept that callback. Export includes the
latest accepted value, and import restores the controlled selection. Malformed,
noncanonical, and out-of-range dates are rejected before replacing the mounted
preview. The factory safely handles an absent or malformed optional date.

This does not change the DatePicker API, its upstream integration, pinned
revisions, or vendor sources.

## Validation

- Focused locked/offline isolated-target regression passed: 1 test, 0 failures.
  It covers empty defaults, empty events/actions, selection, clearing, descriptor
  export/import, malformed dates, and callbacks after remount/component changes.
- Locked/offline Playground all-targets check passed (exit 0). Existing
  `THEME_CATALOG_LEN` and `table_binding` warnings and the pinned `block 0.1.6`
  future-incompatibility notice remain.
- Rust formatting and scoped whitespace checks passed (exit 0).
- No GUI or native accessibility acceptance run.

Evidence is retained in `target/headstart/evidence/`:

- `d1c-datepicker-preview-tests-20261020.log`
- `d1c-datepicker-preview-check-20261020.log`
- `d1c-datepicker-preview-tests-initial-failed-20261020.log` preserves the initial
  compile failure from an omitted factory-context field; that field was added
  before the successful focused regression.

The isolated target is `target/headstart/d1c-avatar-preview-check`, reusing the
prior specimen dependency cache while avoiding the shared default build target.
