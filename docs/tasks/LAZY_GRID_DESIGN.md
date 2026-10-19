# LazyGrid design proposal

**Status: Proposed. This document describes a design, not implemented behavior.**

## Goal

Provide a virtualized grid whose cells are arbitrary RustUI `View` compositions.
Only cells in the row range requested by GPUI are constructed and lowered. The
grid must preserve per-cell component state across reordering and scrolling,
drop state for removed cells, and keep cell actions attached to the containing
RustUI component owner.

This proposal uses GPUI's pinned `uniform_list` for vertical virtualization.
It does not add a second scheduling, state, or rendering runtime and does not
modify vendored GPUI code.

## Current constraints

- `View::into_element` receives a borrowed `RenderContext`. That context borrows
  the current `Window`, `App`, and `RetainedEntities` and cannot be captured by
  GPUI's deferred callback.
- A root `ComponentHost` begins and finishes retention during its synchronous
  render. Its finish step drops unseen retained entries and child components.
- GPUI `uniform_list` stores a `'static` callback with the signature
  `Fn(Range<usize>, &mut Window, &mut App) -> Vec<R>`. GPUI calls it for the
  requested visible range and any range it needs to measure or lay out.
- The existing `List` and `KeyedList` adapters produce string rows. They do not
  provide RustUI cell composition.

Consequently, the callback cannot borrow the root `RenderContext` or share the
root host's frame-level retention pass. It must use a fresh context and
independently owned retention state.

## Candidate public API

The concrete syntax is open for review. The API must carry stable IDs in the
complete data set and take a factory, not prebuilt cell views:

```rust,ignore
LazyGrid::new(
    "photo-grid",
    items, // Rc<[LazyGridItem<T>]>, each item has a stable Identity and value
    NonZeroU16::new(3).unwrap(),
    |index, item| {
        VStack::new()
            .child(Image(item.image.clone()))
            .child(Text(format!("{}: {}", index, item.title)))
    },
)
.height(480.0)
.row_height(144.0)
.column_gap(8.0)
.row_gap(8.0)
```

Here `items` is an immutable, reference-counted snapshot so the deferred
callback can own the current data without cloning every item. `LazyGridItem`
must reject blank IDs and duplicate IDs across the full collection. The
required grid identity must also be unique in its current RustUI retention
scope. Exact names and builders can change during implementation review.

The cell factory has the conceptual type
`Fn(usize, &T) -> V`, where `V: View + 'static`. It is invoked only for cells
in rows requested by GPUI, including any measurement or overscan rows GPUI
requests. It must not be mapped over the full item collection when the grid is
constructed or synchronously lowered. ID validation may scan the full ID set;
it must not call the factory. A large-data acceptance probe must confirm that
factory calls remain limited to requested rows rather than the full data set.

## Deferred rendering and ownership

1. During the containing owner's normal `into_element`, create or retrieve a
   stable per-grid GPUI state entity through the owner's existing retained
   entity mechanism. Refresh it with the current item snapshot, cell factory,
   layout options, and render-context seed before returning the GPUI list
   element.
2. The seed is owned data only. It carries the root `ComponentUpdater`, root
   `EntityId` and owner `TypeId`, the root owner's weak task-owner handle, the
   current environment values, control font size and button theme, and the
   current localization snapshot. It never stores a `RenderContext`,
   `Window`, or `App` borrow.
3. The `uniform_list` closure captures a `WeakEntity` for the grid state and the
   current immutable data snapshot. For each requested range, it upgrades the
   weak entity and uses `WeakEntity::update` with the callback's `App` to make
   one fresh `RenderContext`, using the callback's `Window` and `App` and the
   grid entity's `RetainedEntities`. If the weak entity is gone, return no
   elements. Do not capture a strong entity or any frame-borrowed reference in
   the callback.
4. Map each requested row to its row-major cell indexes. For each cell, call
   the factory inside its stable cell scope, then lower the returned view with
   the fresh context. The cell scope must participate in both RustUI retention
   keys and native element IDs. Use the stable item ID, not the current index,
   so reorder preserves state. Same child IDs in different cells remain
   distinct because their retention paths include the cell scope.
5. Keep a dedicated `RetainedEntities` store on the per-grid state entity. Do
   not run `begin_frame`/`finish_frame` for the whole grid per callback: GPUI
   may request disjoint ranges in separate callback invocations, and treating
   each invocation as a full frame would drop offscreen cell state.
6. During synchronous reconciliation, prune all row scopes whose IDs are no
   longer in the full current item snapshot, before constructing the callback.
   This also drops retained child component entities, subscriptions, and their
   release hooks. Offscreen rows still present in the data stay retained. When
   the grid itself disappears, the containing owner's finish step drops the
   per-grid entity; the weak callback then cannot keep it alive.

The retained cell state is therefore bounded by the number of distinct cells
in the current data set, not by the total number of items ever visited. It can
grow with the current data set because keeping arbitrary cell-local state while
scrolling away and back requires retaining that state. Removing data prunes
that state. A future bounded cache/eviction policy would intentionally reset
offscreen component state and needs a separate API decision.

### Owner actions and tasks

The seed preserves the root's action updater and weak task-owner handle. Cell
buttons and bindings can therefore dispatch to the current containing owner;
`OwnerActionReader` and `OwnerTaskReader` must still observe the same root owner
type and entity as ordinary synchronous children. Weak root ownership makes
late actions harmless after the root is released. Cell `Component` views still
create their own normal `ComponentHost` owners for local state and lifecycle.

### Context refresh

Refresh the seed on every parent render. The root host already observes locale
invalidation; the next root render supplies the new locale, direction,
messages, environment, theme, and font size. Each deferred callback receives
only the seed from the GPUI element created for that render. Lowered cell
components receive the current environment and localization context. When an
offscreen cell returns, its next lowering refreshes its retained component
environment before rendering. No callback may keep a prior frame's borrowed
context or use a stale snapshot after a newer parent render.

Text element IDs need a stable per-cell prefix in addition to the root owner
ID. `RenderContext` currently derives its text prefix from the root ID and
starts the slot counter at zero; independently created cell contexts would
otherwise generate colliding IDs. Add an internal cell-scoping operation that
derives the prefix from the stable cell ID and resets the local text slot for
that cell. It must not change `owner_id`, `owner_type`, or action/task targets.

## Layout and sizing contract

- The grid has a fixed, positive column count and lays cells out row-major. The
  last row may be partial. This phase virtualizes rows vertically; it does not
  virtualize columns horizontally.
- The viewport height is finite and positive. Every virtual row has one
  uniform, finite, positive height supplied by the caller. This is required so
  GPUI can calculate offsets without lowering unseen rows. Cell content that
  exceeds that height follows the documented overflow/clipping behavior; it
  must not silently alter later row offsets.
- Columns use equal available widths after applying the configured column
  gaps. Rows use the declared height and configured row gap. Cell composition
  may be arbitrary RustUI content within those constraints.
- The list element has a stable native ID derived from the required grid ID.
  Search, selection, horizontal virtualization, variable-height rows, custom
  tracks/spans, and a scroll-position persistence guarantee across data
  replacement are outside the initial contract.

## Implementation work after design review

1. Add the public `LazyGridItem`/`LazyGrid` API and exports in `layout/` and
   `prelude.rs`; document the API contract and validate IDs, dimensions, and
   column count at construction.
2. Add an owned render-context seed and per-cell text-ID scoping in `view.rs`.
   The seed must expose only crate-internal cloning and fresh-context creation.
3. Add per-grid retention scope pruning in `runtime/component_host.rs` or the
   retained entity helper. Pruning must drop both retained entities and
   component hosts under removed cell scopes without pruning current offscreen
   cells.
4. Implement the uniform-list adapter in `layout/lazy_grid.rs`. Build rows
   only from the callback range, assign stable cell scopes, refresh context
   before callback rendering, and return empty if the grid weak owner expired.
5. Add a Playground story with stateful child cells, reorder, insertion,
   removal, and a large source data set. Keep direct GPUI dependencies inside
   framework implementation crates.

## Focused verification plan

- Pure tests: reject empty/duplicate IDs across the full data set; reject zero
  columns and invalid viewport/row dimensions; map callback row ranges to
  row-major cell indexes, including empty and partial final rows.
- Retention tests: preserve a cell's child state through reorder and
  offscreen/onscreen rendering; prune removed IDs before any later callback;
  release cell owners when the grid disappears; keep same child IDs isolated
  across cells and across sibling grids.
- Context tests: cell text IDs are unique and stable across reorder, while
  owner action and task targets retain the root owner identity and type.
- A callback-count test with a large collection verifies the factory is called
  only for the requested visible/measurement ranges, never eagerly for every
  item. Do not assert a hard-coded overscan count unless the pinned GPUI source
  makes that exact count part of its contract.
- Integration verification: insert, reorder, remove, and scroll cells in the
  Playground; verify retained local state, root-owner actions, localization and
  environment refresh, native accessibility output, and no state after owner
  removal. Build success alone is not GUI acceptance.

## Evidence reviewed

- `crates/rustui/src/view.rs`: `RenderContext`, `RetentionScope`, `KeyedView`,
  `ComponentView`, and owner readers.
- `crates/rustui/src/runtime/component_host.rs`: root retention begin/finish,
  retained component ownership, and release lifecycle.
- `crates/rustui/src/components/data/list.rs` and
  `crates/rustui/src/components/data/keyed_list.rs`: current string-only list
  delegates and stable selection behavior.
- `vendor/gpui-pre-0.3.9/src/elements/uniform_list.rs`: uniform row list and
  `'static` range callback receiving only `Window` and `App`.
- `docs/tasks/C2_GRID_ZSTACK.md` and `docs/tasks/R3B_KEYED_LIST.md`: existing
  fixed-grid scope and stable-ID list contract.
