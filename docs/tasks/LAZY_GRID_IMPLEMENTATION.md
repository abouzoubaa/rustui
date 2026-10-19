# LazyGrid implementation (M54)

## Stage 8 virtualization audit (2026-10-19)

The current implementation already uses pinned GPUI `uniform_list` for vertical row virtualization. GPUI computes a visible row range from the scroll offset and viewport, then calls RustUI's row factory for that range; `map_row_cells` invokes the cell factory only for items in those rows. The public contract has fixed column count, row height, and viewport height, so no new layout API is needed. `GridState` receives the latest snapshot and generation during parent rendering, and cells are keyed by stable item `Identity` rather than index. The Playground gallery exercises 2,400 items with reverse, prepend, removal, restoration, and scroll actions. A focused unit check now covers reordered IDs and updated values in requested rows. This audit made no virtualization implementation change. GUI acceptance remains the M54 evidence recorded below; the new unit check awaits the shared Headstart Cargo validation run.

`LazyGrid::new(id, Rc<[LazyGridItem<T>]>, columns, viewport_height, row_height, factory)` virtualizes rows with pinned GPUI `uniform_list`. `columns` is `NonZeroU16`; heights must be finite and positive. `.column_gap()` and `.row_gap()` accept finite nonnegative values; the total column-gap span and row-height-plus-gap sum must also stay finite. Each item has a stable nonblank `Identity`; duplicate IDs anywhere in the complete snapshot are rejected before the lazy callback. The factory has shape `Fn(index, &T) -> V` and is called only for rows GPUI requests, including measurement ranges. Equal-width columns are laid out in logical row order, reversed for RTL. Columns are not virtualized. Cell content above the declared row height is clipped; each native list row occupies `row_height + row_gap`, including padding after the last row. The sum must remain finite.

A retained GPUI entity per grid owns a current context seed and a dedicated `RetainedEntities` store. The seed clones owner updater, ID/type, weak task owner, environment, theme, font, and locale. It has no borrowed `Window`, `App`, or frame context. A weak grid entity in the list callback builds a fresh context from the callback's window and app. A generation check rejects stale callbacks after parent rerender. Text IDs include the per-grid entity ID and stable cell ID, keeping them separate across sibling grids and stable across item reorder.

On each callback cell, the retained store reconciles only that cell: unseen conditional descendants are released, while other current offscreen cells remain retained. Parent reconciliation scans the full snapshot IDs and prunes state for removed cells. Removing the whole grid releases its retained entity; the weak callback cannot retain it. Memory for cell-local state grows with distinct visited cells still in the current data set, plus the caller's immutable item snapshot. There is no cache eviction for current offscreen cells because that would reset their local state.

Unit checks cover full-snapshot duplicate IDs, dimensions including row-height-plus-gap and total column-gap overflow, row mapping, and factory calls through the same row mapper used by rendering. The factory count test checks only a requested row; it does not assert native overscan behavior. Native build, large-data callback counts, owner action/task behavior, localization refresh, accessibility, and GUI scrolling remain for root integration verification; source implementation alone does not establish those behaviors.

## Explicit native row width correction

The initial M54 native review exposed cells 0 through 8 to accessibility and
reported nine mounts while the viewport was visually blank. Those observations
do not establish painting or M54 acceptance.

Each native row now sets `w_full()` and an explicit horizontal flex direction
before adding equal-width, zero-basis cells. RTL keeps its row-reverse mapping.
Pinned `uniform_list` supplies definite available viewport width to visible
rows (`vendor/gpui-pre-0.3.9/src/elements/uniform_list.rs:505-511`), while its
measurement prototype uses MaxContent width (`uniform_list.rs:675-682`). These
rows are nested layout roots; they do not receive the separate window-root
stretch operation. Giving the row an explicit percentage width lets native
layout resolve viewport width before distributing it among clipped cells,
instead of relying on an automatic shrink-wrapped row width.

A focused check inspects the same native row builder used by rendering for
100% width, fixed row extent, column gap, and LTR/RTL directions. The corrected
build received bounded native acceptance in M54. The accepted review used
`target/headstart/verification/M54 M55 Paint Review.app`, SHA-256
`316b268732146400e3f6efecb788317a509f3f11c8b48bff6dd764afd30bea7f`, with
evidence in `target/headstart/milestone54-lazy-grid-gui.json`. The earlier
binary `05fabb23fc5780056a01adeca40eab671a2704a740ba428638aab531d027ab9a`
exposed cells 0–8 to accessibility while painting a blank viewport; it was not
accepted, and the explicit row-width correction followed it.

The accepted run visibly displayed three columns and cells 0–8. For 2,400
items, the initial sample reported 15 factory calls and 9 mounts, which include
GPUI measurement and repeated render requests and are not an overscan guarantee
or benchmark. The tested cell counter advanced 0→1, and its Root + action
advanced the root owner. Scrolling to cells 24–35 and returning preserved the
counter with one mount and no release. Reversing and restoring order preserved
it; prepending an item moved item-0000 to index 1 without resetting state.
Removing item-0000 released its tested owner, restoring it created a fresh
zero counter, and hiding the grid released all 32 visited owners. An Arabic
remount placed item 0 in the rightmost column and localized Close; switching
live to French updated the label while preserving the counter and observed
mount/release counts.

The shared M54–M55 paint test log reports 161 passed and 0 failed across 12
binaries (`target/headstart/milestone54-55-paint-tests.log`). Remaining limits:
native descendant element IDs are positional; keyboard and screen-reader
coverage is pending. Factory counts do not establish performance. Retained
memory grows with visited cells in the current dataset and has no viewport-size
eviction policy. Theme/custom-environment propagation, background-task
propagation, and stale detached-callback invocation after removal were not
separately exercised natively. Source/style checks, accessibility entries, and
this bounded GUI run do not establish those untested behaviors.
