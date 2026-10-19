# F1: Nested ownership and keyed composition

Status: F1a–F1d are implemented, including live props and render-driven scoped
environment propagation. Milestone 10 final workspace all-target check exited 0
in 2.30 seconds. Tests and GUI acceptance have not run. Fine-grained observation
and cross-owner operations remain design work.

## Implemented descriptions (F1a)

`View::body` describes a temporary tree. `AnyView` erases a description;
`Keyed(key, view)` adds stable retention and native identity scopes. Neither owns
independent component state. `Option<V>` uses `gpui::Empty` for absence and adds no
stack gap. Stack `.children(iter)` eagerly composes supplied rows, without
virtualization or an automatic scope. Duplicate keys fail in the current logical
scope, even for stateless rows.

`ScopedElement` forwards the child's existing LayoutId, prepaint and paint without
adding a layout box. Typed local IDs compose through GPUI ancestors. Tabs use
paired `Tabs/Tab` boundaries; navigation uses `Navigation/Page(full route prefix)`.
Macro grammar remains constructor calls and method chains, not general if/for.

## Implemented ownership (F1b)

`Component(key, initializer) -> ComponentView<C>` explicitly creates a retained
GPUI `ComponentHost<C>` child. The initializer is `Fn() -> C + 'static`, run only
when creating/recreating a slot. Matching scope/key/type retains the entity;
changing type creates a new host and native entity identity. Duplicate child keys
in one retention scope fail even across concrete types. A separate child map
avoids collisions with control entries.

Each host provides its own action/binding updater, retained controls, environment
snapshot and lifecycle. Actions notify that GPUI host; `on_mount` runs on creation
and `on_unmount` follows actual GPUI entity release. Parent removal drops the
registry reference; other GPUI references may delay destruction. Native entity
lowering provides the identity/rendering boundary without an added layout box.
Environment is inherited on creation and refreshed on parent lowering by F1d. Initializer captures do not update live
children and are not a props API.

Existing R2 sweep rules apply: declared inactive tabs and covered declared pages
retain children; removed tabs, popped pages and conditionally absent visible
children release their references. Inactive contents reconcile when rendered again.
Root values and child values belong to their respective hosts.

Action/binding type mismatches now report panics instead of silent no-ops; the
read-only Binding probe still returns Option. Lenses remain owner-relative and
cannot detect crossing between two instances of the same concrete type. Parent
actions/bindings must not be passed across the child boundary as a substitute for
an explicit owner-target API.

## Implemented live props (F1c)

`Component(...).props(value, reconcile)` stores typed incoming props with the
retained child slot. `P: Clone + PartialEq + 'static`; the reconciler is a stable
`fn(&mut C, &P)`. Initial props apply before mount. Later unequal props apply and
notify the existing GPUI child; equal props do not run the callback or notify.
Only fields explicitly written by the callback change. The function must stay
stable for the slot because callback changes alone do not trigger reconciliation.

A plain Component clears the previous-props baseline without undoing projected
values. Reattaching props reapplies them. A props-type change also reapplies while
preserving C; a C-type change replaces the host. Inactive retained children receive
the latest props on their next lowering; pruned children lose the metadata.

## Remaining decisions
- Define fine-grained environment observation and any cached-child invalidation.
  F1d now provides typed scoped overrides/readers and inherited snapshot refresh
  when parents render, without a second observation runtime.
- Define parent-target callbacks/bindings if needed, preserving correct owner
  lifetime and avoiding silent instance retargeting.
- Keep arbitrary if/for macro grammar as a separate parser decision; heterogeneous
  branches can currently use explicit AnyView.
- Run requested acceptance for child creation, reorder, removal/restoration, type
  replacement, duplicate keys, local action/binding isolation, native focus,
  inactive tabs/pages, mount/release and subscription cleanup. Compilation alone
  does not establish those runtime properties.

No advanced GPU or native composition work belongs to F1. GPUI Fast remains P4,
after workload/baseline evidence and the isolated versioned patch/fallback process,
with separate phase approval.
