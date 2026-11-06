# M300 — Stage 9 / D1c DataGrid preview specimen

## Status

Implemented. The focused regression passed; the locked/offline Playground
all-targets check passed. Native GUI acceptance was not run.

## Scope

Add an opt-in DataGrid specimen to the canonical compiled-preview gallery, using the public `components::data::data_grid::DataGrid` and `components::data::table::TableView` APIs.

The descriptor exposes only these bounded scalar properties:

- `label`: required accessible name, 1–80 characters; reject values whose trimmed form is empty during live feedback and import.
- `selected_id`: one of `none`, `row-0`, `row-1`, or `row-2`; default `none`.

The compiled factory constructs one fixed table with exactly two columns and three rows. Column headings, cell values, row identities, and table shape are fixture data owned by the specimen; the descriptor must not serialize arbitrary rows, columns, or executable behavior. Map `selected_id` to the table's controlled selection and use the existing generation/revision-stamped property-feedback path when selection changes. Preview event and action maps remain empty: selection is property feedback, not an event.

Add a visible `Preview DataGrid specimen` mount control and use the existing preview host for property editing and descriptor import/export. Validate the complete descriptor before mounting. Reject a blank label or unknown selection value without replacing or mutating the currently mounted preview.

## Acceptance

- Focused regression covers defaults, nonblank label bounds, the restricted selection enum, exact fixed fixture shape/content, empty event/action maps, descriptor round-trip, accepted and stale generation/revision-stamped selection feedback, and rejection of blank labels or invalid selections while preserving the mounted preview.
- Run the focused regression and locked/offline Playground all-targets check with an isolated Headstart target directory. Record evidence paths only after each check actually runs; inspect the scoped diff and formatting.
- Native accessibility-tree role/name and cell/header exposure, keyboard and focus behavior, screen-reader output, localization, and platform behavior remain open acceptance gates. Source tests and successful compilation do not close them.

## M303 native observation

On macOS 27, the native accessibility tree exposed a named `Quarterly revenue`
table with a header row and Q1–Q3 data rows, each with Quarter and Revenue
cells. Clicking Q2 updated stamped `selected_id` feedback to `row-1` at
revision 1; Down moved it to `row-2` at revision 2. The selected Q3 row showed
selection feedback, but AX did not report row selection or focus. This is
partial native acceptance. VoiceOver, native row focus/selection semantics,
broader keyboard behavior, localization/RTL, visual comparison, and other
platforms remain open. Evidence:
`target/headstart/evidence/m303-datagrid-native-build.log` and
`target/headstart/evidence/m303-datagrid-native-observation-20261020.json`.

## Boundaries

Keep the change within the compiled-preview specimen and its required regression/documentation. Do not change upstream dependency revisions, vendor code, or the shared Playground story registry as part of this task.

## Validation completed

- Focused regression: `target/headstart/evidence/m300-focused-test.log`
- Playground all-targets check: `target/headstart/evidence/m300-m301-playground-all-targets-check.log`
- Isolated target directory: `target/headstart/candidates/m300-validation/target`

M303 cleanup follow-up: the disposable validation bundle and standalone executable have been removed, and no validation process remains. Pre-existing Playground PID 20036 was untouched. Observation evidence: `target/headstart/evidence/m303-datagrid-native-observation-20261020.json`.
