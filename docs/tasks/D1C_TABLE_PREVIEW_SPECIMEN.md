# M308 — Stage 9 / D1c public Table preview specimen

## Scope

The compiled-preview gallery exposes an opt-in specimen using the public
RustUI `Table` API. It uses fixed Task/Status columns and three fixed rows:
Design/Complete, Implement/In progress, and Review/Queued. The descriptor
stores only `selected_id`, restricted to `none`, `row-0`, `row-1`, or `row-2`;
the table fixture remains compiled into the gallery factory. Event and action
maps are empty.

Row selection uses the existing generation/revision-stamped property feedback
bridge. The focused regression covers the fixture/schema, valid selection,
stale callback rejection, invalid row rejection, and descriptor round-trip.

## Verification

- Focused specimen regression passed **1/1**:
  `target/headstart/evidence/m308-table-preview-focused-test-20261020.log`.
- Locked/offline Playground all-targets check passed:
  `target/headstart/evidence/m308-table-preview-all-targets-check-20261020.log`.
- Scoped whitespace check passed for
  `apps/rustui-playground/src/stories/compiled_preview_gallery.rs`.

This is source and compile evidence only. Native table appearance/semantics,
selection, keyboard/focus, accessibility/VoiceOver, localization/RTL, visual
comparison, and other-platform acceptance remain open.

## M311 — Native observation

On macOS 27, the compiled Table specimen visibly rendered Task and Status
columns with Design/Complete, Implement/In progress, and Review/Queued rows.
Its accessibility tree reported row and cell roles, including both headers
and all six data cells. Clicking Implement changed `selected_id` from `none`
to `row-1`; the preview stamp advanced from generation 4/revision 0 to
generation 4/revision 1. AX did not report a selected-row or focus state.
This is partial native acceptance. VoiceOver, keyboard/focus,
localization/RTL, visual comparison, and other-platform acceptance remain
open. Evidence: `target/headstart/evidence/m311-table-native-observation-20261020.json`
and `target/headstart/evidence/m309-preview-controls-native-build.log`.
