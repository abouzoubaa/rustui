# M1223 — Stage 5: reconcile resolved list rows

`ListView` initialized its retained GPUI delegate with locale-resolved labels,
but rerender reconciliation compared those labels to untranslated source items
and replaced the delegate rows with the untranslated items. Lists with localized
suffixes therefore lost the suffix after a render update.

Reconciliation now compares and installs the same resolved labels used to
initialize the delegate. Focused regressions check both the change predicate
and retained delegate behavior: unchanged resolved rows remain stored across a
rerender, then a locale change replaces them with the newly resolved labels.
The retained-delegate regression passed with
`./scripts/cargo-headstart test -p rustui --lib retained_delegate_keeps_resolved_rows_across_rerender_and_locale_change --locked --offline`
(1 passed). Native visual and live locale-switch acceptance remain open.

## Native acceptance run sheet — prepared 2026-10-24, not performed

Do not run until Stage 6 explicitly reports cleanup and release of the shared
Playground validation slot and the earlier Playground processes have exited.
Use the existing single disposable Playground validation app; do not create a
second app for this milestone.

1. Clear search (`playground-search`), select the `slider` catalog row, and
   confirm the selected row and canvas story are both Slider.
2. In English, confirm `slider · Partial`. Capture the row, selection, and
   canvas story; record the native accessibility name and selected value.
3. Click `Français` (`lab-locale-fr`) without restarting. Confirm the live
   label is `slider · Partiel`, selection remains on Slider, and the canvas
   still shows Slider. Capture and record the native accessibility name and
   selected value.
4. Search `Partiel`. Confirm `slider · Partiel` is visible; select that result
   and confirm Slider remains active. Record every visible catalog key/label,
   selected key, canvas story, and native name/selected value.
5. Keep the `Partiel` query and click `العربية` (`lab-locale-ar`). Confirm
   `slider` remains in the results because coverage filtering searches all
   registered catalogs, while its active-locale label is `slider · جزئي`.
   Confirm selection and the Slider story remain unchanged; capture and record
   the native name/selected value.
6. Replace the query with `جزئي`, confirm `slider · جزئي` remains visible and
   selected, then clear search and confirm selection remains on Slider.
   Capture each filter state and record visible rows, selected key, story, and
   native name/selected value.
7. In each unfiltered locale, inspect the native accessibility tree for the
   Slider row. Record the actual role, full accessible name, and selected
   value. If the tree does not expose a value, leave that check incomplete.

Expected row labels are `slider · Partial` (English), `slider · Partiel`
(French), and `slider · جزئي` (Arabic). These are expectations, not observed
native results.

## Current source/test oracle boundary

The focused source tests cover these narrower claims:

- `localized_suffixes_preserve_item_prefixes_and_resolve_by_active_locale`
  checks `slider · Partial`, `slider · Partiel`, and `slider · جزئي` from the
  suffix resolver. It does not exercise a Playground window.
- `retained_delegate_keeps_resolved_rows_across_rerender_and_locale_change`
  checks that resolved English rows survive an unchanged delegate update and
  that French resolved rows replace them. It does not exercise a live locale
  button, Arabic, or native accessibility.
- `localized_coverage_label_filters_and_selects_catalog_row` checks that
  French `Partiel` and Arabic `جزئي` find and select the original `button`
  catalog item. It does not assert that `slider` appears, enumerate the full
  result set, or preserve Slider selection through live locale changes.
- `filtered_catalog_indices` searches coverage labels across registered
  catalogs. Therefore, while Arabic is active, `Partiel` may still include
  `slider`; its visible suffix should remain the active-locale Arabic label
  `slider · جزئي`.

The native run must record, for each locale/query state: visible catalog row
keys and labels, selected catalog key, active canvas story, and the matching
native accessibility node's role, name, and selected value. The current
renderer attaches the full label as the row accessibility label and to its
child Label. It does not explicitly set `aria-selected` on that row in RustUI;
record the value exposed by the native tree rather than inferring it from the
highlight or owner binding. If name or selected state is absent from the tree,
leave that check incomplete. Screenshots document visible state but do not
substitute for accessibility-tree observations.

### Evidence fields

For every captured state, record the active locale, exact query, all visible
catalog row keys and labels, selected catalog key, active canvas story, and
screenshot path. For each unfiltered locale and each required filter state,
also record the native row role, full accessible name, and selected value.

The source tests do not specify the complete filtered result set. Record the
actual visible rows and assert the required `slider` row; do not claim a
locale-exclusive filter or infer an expected full result list from the
single-item unit assertions.

After the run, update
`target/headstart/evidence/m1223-stage5-list-locale-native-acceptance-20261024.json`
with the current HEAD, actual per-check outcomes, observed accessibility
results, and screenshot paths. Keep native checks `not_performed` until
observed; mark native acceptance passed only when every check is evidenced.
The focused unit oracle remains unit-only evidence.

After the GUI run, quit that validation app normally with Cmd-Q; make one
bounded process check that its executable exited; remove only that run's
disposable validation app bundle and any standalone GUI test executable it
created; then verify no test app remains running. Preserve source,
screenshots/evidence, and shared Cargo caches. Do not stop another worktree's
process or remove another worktree's bundle.

At preparation time, all native acceptance checks remain `not_performed`; no
app was launched, no build was run, and no source code or patch was changed.
